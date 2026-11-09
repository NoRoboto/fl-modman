# Cloth FX 1.1-rc

- Author: Kris1982 (based on the approach of cloth-render v42 by Alex10). Archive `Cloth_FX_1.1-rc.zip`.
- Video: https://youtu.be/y8AvOylMSQk
- Download: https://drive.google.com/file/d/1r1BiybuoowlzEvUyw3snizja_7lCDiUE/view
- Type: Lua module + native DLL (`content/cloth-render/cloth_fx.dll`) + cloth meshes. FL27: **verified** (imported
  and applied with fl-modman on the Ally, LAB 2026-11-08 and stable 2026-11-09).

## Original instructions (brief)

> Copy the "modules" and "content" folders into your SiderAddons folder. In sider.ini:
> `luajit.ext.enabled = 1`, `lua.module = "cloth-fx.lua"`. Do not load cloth-render.lua /
> cloth-render-fl.lua at the same time. Settings are saved to content\cloth-render\cloth-fx.ini.

## Mapping

- `modules/cloth-fx.lua` to `@modules/`, one `lua_module` line.
- `content/cloth-render/` (DLL, ini, meshes) to `@sider/content/cloth-render/`.
- `README_Cloth_FX.txt` is not installed.
- The module runs native code (the DLL). The overlay rewrites `cloth-fx.ini`.

## Import

```sh
fl-modman import Cloth_FX_1.1-rc.zip --root . --type loose --manifest catalog/cloth-fx-1-1-rc/mod.toml
```
