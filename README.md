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

## Acknowledgements
- This project originally started as a wrapper around [App2Unit](https://github.com/Vladimir-csp/app2unit) (❤️) before being rewritten to talk directly to systemd via D-Bus.
  That's the reason why the name "Game2Unit" is (heavily) inspired by "App2Unit".
