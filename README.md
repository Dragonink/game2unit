# Game2Unit
Launch games as systemd units.

## Usage
```
game2unit [<launcher_args>...]
```
Game2Unit is basically a wrapper around an external *systemd unit launcher*.
> [!IMPORTANT]
> All arguments given to Game2Unit will be passed down to the *systemd unit launcher*.

Moreover, additional arguments retrieved from a [source](#sources) are passed to the launcher as well.

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
<LAUNCHER> [-a <app_name>] [-d <unit_description>] [-p <key>=<value>] [-t scope] <program> [<program_args>...]
```
Launcher option | Description
--:|:--
**`-a <app_name>`** | Application name substring of unit ID
**`-d <unit_description>`** | Unit description
**`-p <key>=<value>`** | Additional unit property
**`-t scope`** | Force unit type to be a scope

## Sources
Game2Unit retrieves additional systemd unit properties (like the unit name and description) from a source environment.

### Steam
Prepend `game2unit` to the *Launch Options* of your game (or shortcut), for example:
```
game2unit -- %command%
```
