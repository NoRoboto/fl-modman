# Machete FL27 Immersive indicators (player cursors, target, card)

- Author: Machete (archive `FL27_Immersive_indicators.rar`, kept in the Ally's `~/Games/FL27-src/mods`). Public URL
  not recorded; a web search found no page [U].
- Type: livecpk (three roots). FL27: **verified** (Ally: three roots registered after `Mariano Closs V5` by
  `rog-ally-fl27/sider_add_mods.py`, folders `livecpk/{NoBallX,FanuIndics,FanuIndics2}`).

## Original instructions

No readme in the archive. The install that was done: copy the three folders into `livecpk` and add
`cpk.root` lines `NoBallX`, `FanuIndics`, `FanuIndics2`, in that order.

## Mapping

Three folder puts and three `cpk_root` lines, in the installed order. Sider's lookup is first-wins, so for the
`d2_target` model that all three ship, `NoBallX` wins (the "no ball X" target), as on the Ally.
`FanuIndics` also holds the player cursors `d2_cursor_a/b/c` and `d2_card` (the "Machete cursor" part).

## Archive layout and import

Listed with `unrar lb`: `indicators/{NoBallX,FanuIndics,FanuIndics2}/Asset/model/game2d/…` (`.fmdl`, `.ftex`).

```sh
fl-modman import FL27_Immersive_indicators.rar --root indicators --type livecpk --manifest catalog/machete-immersive-indicators/mod.toml
```

The payload lands at `mods/<id>/indicators/`; `--manifest` uses this `mod.toml` (the generated one has one root named `indicators`).
