# Game2Unit
Launch games as systemd user units.

## Usage
Just prepend `game2unit` to the usual command that launches your game:
```
game2unit <program> [<program args>...]
```

> [!IMPORTANT]
> If the command involves settings environment variables like this:
> ```sh
> FOO=foo BAR=bar game --game-option
> ```
>
> You will need to put the variables before `game2unit`:
> ```sh
> FOO=foo BAR=bar game2unit game --game-option
> ```
> or use the [`env`](https://www.man7.org/linux/man-pages/man1/env.1.html) command:
> ```sh
> game2unit env FOO=foo BAR=bar game --game-option
> ```

Created systemd units will be put under [`app.slice`](https://www.freedesktop.org/software/systemd/man/latest/systemd.special.html#app.slice) by default.
That can be set using the `GAME2UNIT_SLICE` environment variable.

> [!TIP]
> If you are using [UWSM](https://github.com/Vladimir-csp/uwsm),
> you should add the following to your configuration:
> ```env
> # ~/.config/uwsm/env
> export GAME2UNIT_SLICE='app-graphical.slice'
> ```

## Acknowledgements
- This project originally started as a wrapper around [App2Unit](https://github.com/Vladimir-csp/app2unit) (❤️) before being rewritten to talk directly to systemd via D-Bus.
  That's the reason why the name "Game2Unit" is (heavily) inspired by "App2Unit".
- Thanks to [runapp](https://github.com/c4rlo/runapp) for helping me understand [systemd-run](https://www.freedesktop.org/software/systemd/man/latest/systemd-run.html).
