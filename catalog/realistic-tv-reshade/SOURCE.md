# PES 2021 Realistic TV ReShade preset

- Author: matservant (post by G. Leroy, 2024-01-15). Archive `PES_2021_Realistic_TV_Reshade_by_matservant.zip`.
- URL: https://www.pesmodding.com/2024/01/pes-2021-realistic-tv-broadcast-reshade.html
- Type: loose file in the game root. FL27: **unknown** (made for PES 2021; a ReShade preset is engine-generic, but
  the effects it names must exist in the installed `reshade-shaders/`).

## Original instructions (brief)

> "Install Reshade in your PES folder." Copy "PES 2021 Realistic TV Reshade by matservant.ini" into your PES folder.
> Select the preset in the ReShade menu.

## Mapping

The preset `.ini` → `@game/`. ReShade itself is a prerequisite outside this manifest: its installer is refused
(EXCLUDED.md), and `ReShade.ini` / `ReShade.log` are user paths. On the Ally ReShade is already present
(`dxgi.dll`, `reshade-shaders/`). Pick the preset in the ReShade menu; that choice is written to `ReShade.ini`.

## Archive layout and import

The `.ini` at the top of the zip [U]:

```sh
fl-modman import PES_2021_Realistic_TV_Reshade_by_matservant.zip --root . --type loose --manifest catalog/realistic-tv-reshade/mod.toml
```

The payload lands at `mods/<id>/game/`; `--manifest` uses this `mod.toml` (the generated one, `game/` → `@game/`, is equivalent).
