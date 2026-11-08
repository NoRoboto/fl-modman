# Offside VAR 1.0-rc

- Author: Kris1982. Archive `FL27_Pes21_Offside_VAR_1.0-rc.zip` (2026-11-07).
- Video: https://www.youtube.com/watch?v=RPtgnXa21W4
- Download: https://drive.google.com/file/d/1AKIX_WpryLa0ORHJ-I4nGa37LppZ9RJR/view
- Type: Lua module + livecpk model + ReShade add-on and effect.

## Original instructions (brief)

> Unzip into the game folder. In sider.ini: `luajit.ext.enabled = 1`, `cpk.root = ".\livecpk\root"`,
> `lua.module = "offside.lua"` after `lib\nesalib.lua`. Needs CommonLib.lua + nesalib.lua (FL27 ships both)
> and ReShade 6.x with add-on support for the banner. Enable "OffsideBanner" in ReShade once.

## Mapping

- The wall model goes to its own root `livecpk\Offside VAR` instead of the base `livecpk\root`, so it is
  undoable and does not write into the base folder. Same effect: livecpk serves the path.
- `offside.lua` + `offside.ini` to `@modules/`, one `lua_module` line (FL27 loads both libs first).
- `offside_banner.addon64` and `reshade-shaders/` next to the exe (`ReShade.ini` searches `.\reshade-shaders\**`).
- `README_Offside_VAR.txt` is not installed.
