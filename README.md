# Game2Unit

Wrapper around [App2Unit](https://github.com/Vladimir-csp/app2unit)
that extracts the unit properties from the launcher environment.

## Usage

```
game2unit [launcher] ...[app2unit option] -- <game program> ...[game argument]
```

As a wrapper, all arguments given to `game2unit` (except `launcher`) will be passed down to `app2unit`.

If given a `launcher`, `game2unit` will extract some properties from the environment and give them to `app2unit`.
See the [supported launchers](#supported-launchers) for setup instructions and details about generated properties.

## Installation

0. Install the runtime dependencies:
   - [App2Unit](https://github.com/Vladimir-csp/app2unit)
   - [Nushell](https://www.nushell.sh/)
1. Download this repository
2. Link [`game2unit`](./game2unit) to your `PATH`:
   ```shell
   ln -sr game2unit ~/.local/bin/
   ```

## Supported launchers
