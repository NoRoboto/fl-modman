# Machete FL27 Immersive camera angles

- Author: Machete; camera modules by nesa24 / digitalfoxx, nesalib by juce (file headers). Archive
  `FL27_Immersive_camera_angles.rar` (Ally `~/Games/FL27-src/mods`). Public URL not recorded [U].
- Version: `BroadCastCam.lua` reports `version = "2.2"`.
- Type: Lua module. FL27: **claimed** (made for FL27 by name). It was installed on the Ally, but its four
  `lua.module` lines are now commented out there (`;lua.module = …`); the reason is not recorded [U].

## Original instructions

No readme. The install that was done (`rog-ally-fl27/sider_add_mods.py`): copy `modules/` into `SiderAddons/modules`
and add `lua.module` lines `BroadCastCam.lua`, `FanViewCam.lua`, `StadiumCam.lua`, `VerticalCam.lua` after the
base ones.

## Mapping

- One put per file; `modules/backup/BroadCastCam.ini` (the author's backup copy) is left out.
- `lib/nesalib.lua` overwrites the base library (same SHA-256 as the file now on the Ally, which may itself come from this archive; whether it differs from
  the stock FL27 file is [U]; the base file is kept in
  `originals/` for undo). FL27's base already loads `lib\nesalib.lua` before every new module, as the cameras need.
- The `.ini` files are rewritten by the modules at runtime; they match `drift_ignore`, so in-game tweaks are kept.
- Conflicts: same file names as `realistic-epl-camera-2026` (enable one). FL27's base `camera.lua` may drive the same
  camera memory [U] (install-rules Theorem 8): test with the base camera settings first.

## Archive layout and import

Listed with `unrar lb`: `Immersive camera angles/modules/{BroadCastCam.lua,.ini, FanViewCam.lua, StadiumCam.lua,.ini,
VerticalCam.lua,.ini, lib/nesalib.lua, backup/BroadCastCam.ini}`.

```sh
fl-modman import FL27_Immersive_camera_angles.rar --manifest catalog/machete-immersive-camera/mod.toml
```

Detection descends the wrapper to `modules/` (Lua); the payload lands at `mods/<id>/modules/`. `--manifest` uses this
`mod.toml` (the generated one also puts `backup/`).
