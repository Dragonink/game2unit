# Game2Unit
Launch games as systemd units.

## Usage
```
game2unit [<launcher_args>...]
```
Game2Unit is basically a wrapper around an external *systemd unit launcher*.
> [!IMPORTANT]
> All arguments given to Game2Unit will be passed down to the *systemd unit launcher*.

### *systemd unit launcher* configuration
The *systemd unit launcher* command is configured from the following sources, by decreasing priority:
1. `GAME2UNIT_LAUNCHER` runtime environment variable
2. `GAME2UNIT_DEFAULT_LAUNCHER` **compile-time** environment variable
3. Defaults to [`app2unit`](https://github.com/Vladimir-csp/app2unit)

For example, if you don't have App2Unit but you use [UWSM](https://github.com/Vladimir-csp/uwsm), you can add this to your environment variables:
```env
export GAME2UNIT_LAUNCHER='uwsm app'
```

> [!WARNING]
> Both environment variables accept a command with arguments,
> but this command will **not** be executed through any shell.

The *systemd unit launcher* is expected to accept the following argument syntax:
```
<LAUNCHER> <program> [<program_args>...]
```
