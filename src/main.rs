use log::{info, error};
use ini::Ini;
use tokio::{
    process::Command,
    io::BufReader,
    prelude::*,
    io::AsyncBufReadExt
};
use futures::{future::FutureExt, pin_mut, select};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
use chrono::Utc;

#[derive(Clone)]
struct Service {
    pool: r2d2::Pool<SqliteConnectionManager>,
    name: String,
    run: uuid::Uuid,
    exec: String,
    vars: Vec<(String, String)>,
}

fn new_event(conn: &mut Connection, svc: &Service, pipe: &str, data: &str) -> Result<usize, String> {
    conn.execute(
        "INSERT INTO event (service, run, timestamp, pipe, data)
            VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&svc.name, &svc.run.to_hyphenated().to_string(), &Utc::now().to_rfc3339(), pipe, data]
    ).map_err(|e| format!("insert failed: {:?}", e))
}

fn extract_envvars<I, K, V>(iter: I, re: &regex::Regex) -> Vec<(String, String)>
where I: Iterator<Item = (K, V)>,
      K: AsRef<str>,
      V: AsRef<str> {
    iter.filter_map(|(k, v)| {
        re.captures(k.as_ref()).map(|caps| (caps[1].to_string(), v.as_ref().to_string()))
    }).collect()
}

async fn pipe_events<R>(svc: &Service, pipe: BufReader<R>, pipe_name: &str)
where R: AsyncRead + Unpin {
    let mut conn = svc.pool.get().map_err(|e| format!("failed to get connection: {:?}", e)).unwrap();
    let mut lines = pipe.lines();
    while let Some(s) = lines.next_line().await.expect("failed to get line") {
        new_event(&mut conn, &svc, pipe_name, &s).expect("insert failed");
    }
}

async fn run_svc(svc: &Service) {
    info!("spawning `{}`...", &svc.name);

    let re = regex::Regex::new(&format!(r"(?:EVA__{}__)([\w|_]+)", &svc.name)).unwrap();
    let vars: Vec<(String, String)> = extract_envvars(std::env::vars(), &re);

    let mut conn = svc.pool.get().map_err(|e| format!("failed to get connection: {:?}", e)).unwrap();
    let mut cmd = Command::new(&svc.exec)
        //.env_clear()
        .env("LD_PRELOAD", "./libstub.so")
        .env("EVA_SERVICE", &svc.name)
        .envs(svc.vars.clone())
        .envs(vars)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect(&format!("`{}` failed to exec", &svc.name));

    new_event(&mut conn, &svc, "START", "").expect("insert failed");

    let stdout = cmd.stdout.take().expect("no stdout");
    let stderr = cmd.stderr.take().expect("no stderr");

    let pipe_stdout = pipe_events(svc, BufReader::new(stdout), "STDOUT").fuse();
    let pipe_stderr = pipe_events(svc, BufReader::new(stderr), "STDERR").fuse();
    let cmd_fused = cmd.fuse();

    pin_mut!(pipe_stdout, pipe_stderr, cmd_fused);

    let mut maybe_exit_status: Option<std::process::ExitStatus> = None;
    loop {
        select! {
            () = pipe_stdout => {},
            () = pipe_stderr => {},
            ret = cmd_fused => {
                info!("service {} shut down", &svc.name);
                maybe_exit_status = Some(ret.map_err(|e| format!("capturing exit status failed: {:?}", e)).unwrap());
            },
            complete => break,
        };
    }

    use std::os::unix::process::ExitStatusExt;
    let exit_status = maybe_exit_status.expect("no exit status?!");
    let status_code = match exit_status.code() {
        Some(code) => format!("Code({})", code),
        None => format!("Signal({})", exit_status.signal().unwrap_or(-1))
    };

    new_event(&mut conn, &svc, "EXIT_STATUS", &status_code).expect("insert failed");
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let env = env_logger::Env::default()
        .filter_or("EVA_LOG_LEVEL", "eva=debug");
    env_logger::init_from_env(env);

    let manager = SqliteConnectionManager::file("eva.db");
    let pool = r2d2::Pool::new(manager)
        .map_err(|e| format!("failed to create conn pool: {:?}", e))?;

    let conn = pool.get()
        .map_err(|e| format!("failed to get connection: {:?}", e))?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS event (
            id INTEGER PRIMARY KEY,
            service VARCHAR NOT NULL,
            run VARCHAR NOT NULL,
            timestamp TEXT NOT NULL,
            pipe VARCHAR NOT NULL,
            data TEXT NOT NULL
        )", params![],
    ).map_err(|e| format!("create table failed: {}", e))?;

    let re = regex::Regex::new(r"(?:env__)([\w|_]+)").unwrap();
    let i = Ini::load_from_file("eva.ini")
        .map_err(|e| format!("{:?}", e))?;

    let mut services = Vec::new();
    for (sec, prop) in i.iter() {
        if sec.is_none() {
            error!("empty section title, skipping");
            continue;
        }

        let svc_name = sec.unwrap().to_string();
        info!("loading service `{}`...", &svc_name);

        if let Some(exec) = prop.get("exec") {
            let svc = Service {
                name: svc_name.to_string(),
                pool: pool.clone(),
                run: uuid::Uuid::new_v4(),
                exec: exec.to_string(),
                vars: extract_envvars(prop.iter(), &re)
            };
            services.push(svc);
            info!("   ok!");
        } else {
            error!("   skipped, no executable!");
        }
    }

    services.iter().for_each(|the_svc| {
        // TODO: cloning this is nasty, figure out a way to appease the compiler
        //       or make `new_event` not require a service reference
        let svc = the_svc.clone();
        let mut conn = svc.pool.get().unwrap();
        tokio::spawn(async move {
            use tokio::net::UnixListener;
            use tokio::stream::StreamExt;

            let eva_sockfile = format!("/home/aszkid/dev/eva/eva_server.{}.sock", &svc.name);
            match std::fs::remove_file(&eva_sockfile) {
                Err(e) => {
                    if e.kind() != std::io::ErrorKind::NotFound {
                        error!("failed to remove sockfile: {:?}", e);
                        return;
                    }
                }
                _ => {}
            }   
            
            let mut listener = UnixListener::bind(&eva_sockfile).unwrap();
            while let Some(stream) = listener.next().await {
                match stream {
                    Ok(mut stream) => {
                        let mut data = String::new();
                        stream.read_to_string(&mut data).await.unwrap();
                        new_event(&mut conn, &svc, "SYSLOG", &data).expect("insert failed");
                    },
                    Err(e) => {
                        println!("error! {:?}", e);
                    }
                }
            }
        });
    });

    let futs = services.iter().map(|svc| {
        run_svc(svc)
    });
    futures::future::join_all(futs).await;

    Ok(())
}
