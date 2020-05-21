# Eva

_Note: this is just a toy project. Use at your own peril._

Lightweight, batteries-included process monitor.

## Features

- Capture `stdout` and `stderr` output
- Capture `syslog` calls
- Structured log SQLite database

## Planned features

- Restart policies
- Full text search
- Email alerts

## Maybe features?

- HTTP API for starting/stopping services
- Service dependencies (A depends on B)

## Building

1. Run `gcc -shared -fPIC src/stub.c -o libstub.so`
2. Run `cargo build`
3. Profit!

## Service definition
In `eva.ini`:
```
[SERVICE_NAME]
exec=/path/to/executable
env_foo=value
keepalive=true
...
```

## Forwarding environment variables
You have two options:

1. Run `EVA__SERVICE_NAME__VAR__=VAL ./eva`
2. Set `env__VAR=VAL` in `eva.ini`

Note that (1) overrides (2).

## Todo

- [ ] Also store PID (look at [procinfo](https://docs.rs/crate/procinfo/0.4.2))
- [ ] Live process memory usage, etc.
- [ ] Email alerts (SendGrid, MailChimp, ... ?)
