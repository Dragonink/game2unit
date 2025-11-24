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

### [Steam](https://store.steampowered.com/)
**Setup instructions:**  
Set the [*Launch Options*](https://help.steampowered.com/en/faqs/view/7D01-D2DD-D75E-2955) of your games to something like:
```
game2unit steam -- %command%
```

**Generated `app2unit` options:**  
```
-a "steam_${STEAM_COMPAT_APP_ID}"
-d $game_name
```

### [Heroic Launcher](https://heroicgameslauncher.com/)
**Setup instructions:**  
Add a *Wrapper command* (in *Settings*, *Game Defaults*) with the following parameters:
- **Name**: `game2unit`
- **Arguments**: `heroic --`

**Generated `app2unit` options (Legendary):**  
```
-a "heroic_legendary_${HEROIC_APP_NAME}"
-d $game_name
```

### Minecraft — [Prism Launcher](https://prismlauncher.org/)
**Setup instructions:**  
Set the [*Wrapper command*](https://prismlauncher.org/wiki/help-pages/custom-commands/) to something like:
```
game2unit minecraft-prism --
```

**Generated `app2unit` options:**  
```
-a "minecraft_prism_${INST_ID}"
-d "Minecraft (${INST_NAME})"
```
