use log::{info, debug, error};
use ini::Ini;
use std::collections::HashMap;
use tokio::{
    process::Command,
    io::BufReader,
    prelude::*,
};

struct Service {
    exec: String,
    vars: Vec<(String, String)>,
}

async fn run_svc(svc_name: &str, svc: &Service) {
    info!("spawning `{}`...", svc_name);

    let re = regex::Regex::new(&format!(r"(?:EVA__{}__)([\w|_]+)", svc_name)).unwrap();
    let vars: Vec<(String, String)> = std::env::vars().filter_map(|(k,v)| {
        re.captures(&k).map(|caps| (caps[1].to_string(), v.to_string()))
    }).collect();

    let mut cmd = Command::new(&svc.exec)
        //.env_clear()
        .envs(svc.vars.clone())
        .envs(vars)
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect(&format!("`{}` failed to exec", svc_name));

    let stdout = cmd.stdout.take().expect("no stdout");
    let mut lines = BufReader::new(stdout).lines();

    while let Some(s) = lines.next_line().await.expect("failed to get line") {
        info!("{} > {:?}", svc_name, s);
    }

    let exit = cmd.await.expect("failed to wait for process");
    info!("final {:?}", exit);
}

fn extract_envvars(props: &ini::ini::Properties, re: &regex::Regex) -> Vec<(String, String)> {
    props.iter().filter_map(|(k, v)| {
        re.captures(&k).map(|caps| (caps[1].to_string(), v.to_string()))
    }).collect()
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let env = env_logger::Env::default()
        .filter_or("EVA_LOG_LEVEL", "eva=debug");
    env_logger::init_from_env(env);

    let re = regex::Regex::new(r"(?:env__)([\w|_]+)").unwrap();
    let i = Ini::load_from_file("eva.ini")
        .map_err(|e| format!("{:?}", e))?;

    let mut services = HashMap::new();
    for (sec, prop) in i.iter() {
        if sec.is_none() {
            error!("empty section title, skipping");
            continue;
        }

        let svc_name = sec.unwrap().to_string();
        info!("loading service `{}`...", &svc_name);

        if let Some(exec) = prop.get("exec") {
            let mut svc = Service {
                exec: exec.to_string(),
                vars: extract_envvars(prop, &re)
            };
            
            services.insert(svc_name, svc);
            info!("   ok!");
        } else {
            error!("   skipped, no executable!");
        }
    }

    let mut futs = Vec::new();
    for (svc, exec) in services.iter() {
        futs.push(run_svc(svc, exec));
    }

    futures::future::join_all(futs).await;

    Ok(())
}
