# Eva

A lightweight process monitor that produces JSON logs.

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