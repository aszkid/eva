# Eva

Lightweight, batteries-included process monitor.

## Building

1. Run `gcc -shared src/stub.c -o libstub.so`
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