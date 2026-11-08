# Mariano Closs V5 (Latin American Spanish commentary, English slot)

- Author / URL: not recorded. The folder was taken from the local FL26 install
  (`Football Life 2026/SiderAddons/livecpk/Mariano Closs V5`); no public download page was found.
- Version: V5 (folder name).
- Type: livecpk. FL27: **verified** (Ally: `livecpk/Mariano Closs V5/eng/sound/{acb,awb,config}` + `cpk.root`).

## Original instructions

None recorded. As found on FL26: the root holds `ags/sound/{acb,awb,config}`, `mex/menu/…`, `mex/string/all.str`,
`spa/string/all.str`, registered with `cpk.root = ".\livecpk\Mariano Closs V5"` (install-rules row 13).

## Mapping

- `ags/sound/` is moved to `eng/sound/`, so Closs replaces the English commentary slot: pick English commentary in
  the game settings. The `ags` slot needs `Data/dt41_ags_all.cpk`, which the Ally setup parked as unused.
- Not mapped: `mex/` and `spa/string/all.str` (menu strings; they shadow menu mods' `all.str`). Another slot is
  another mod (grilling Q13): copy this manifest and change `eng` to the slot.

## Archive layout and import

`Mariano Closs V5/ags/sound/…` plus `mex/`, `spa/`:

```sh
fl-modman import <archive-or-folder> --root "Mariano Closs V5" --type livecpk --manifest catalog/mariano-closs-v5-eng/mod.toml
```

The payload lands at `mods/<id>/Mariano Closs V5/`; `--manifest` uses this `mod.toml` (the generated one maps `ags` as-is).
