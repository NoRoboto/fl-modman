# Referee Kit Server Season 2025/26

- Author: RezvFX (post by G. Leroy, 2025-12-10). Download "Ref KitServer" (modsfire); archive name not given.
- URL: https://www.pesmodding.com/2025/12/referee-kit-server-season-202526-pes.html
- Type: livecpk + Lua module. FL27: **claimed** ("PES 2021 & SP Football Life PC Games"; needs Sider 6.5+ and
  CommonLib, both in the FL27 base).

## Original instructions (brief)

> "Sider Version 6.5+ requires & CommonLib.lua". Copy the `content`, `livecpk`, and `modules` folders into your
> Sider directory. Add `cpk.root = ".\livecpk\RefColor"` and `lua.module = "RefKitServer.lua"`.

## Mapping

- Three folder puts into `SiderAddons/` + one root + one module (after the base modules, so after CommonLib).
- FL27's base already loads `common\referee.lua` (SmokePatch referee kits). Two referee modules may both
  answer the same Sider event, and Lua module order decides [U]; test with only one first.
- `content/*/config.ini` is a user path: see `menu-barcelona-2027/SOURCE.md`.

## Archive layout and import

`content/`, `livecpk/RefColor/`, `modules/RefKitServer.lua` at the top [U]:

```sh
fl-modman import <archive> --root . --type loose --manifest catalog/referee-kit-server-2526/mod.toml
```

`--manifest` uses this `mod.toml` instead of the generated one.
