# New Football Life 2026 Menu Mod + Intro (MenuC1987)

- Author: Carlos1987 (post by G. Leroy, 2025-10-21). Archive `MenuC1987.rar` (MediaFire).
- URL: https://www.pesmodding.com/2025/10/new-football-life-2026-menu-mod-intro.html
- Type: livecpk + Lua module. FL27: **claimed for FL26** ("SP Football Life 2026").

## Original instructions (brief)

> Copy the "content", "livecpk", and "modules" folders into your sider folder. Add
> `cpk.root = ".\livecpk\MenuC1987"`, `cpk.root = ".\livecpk\UIColors"`, `lua.module = "UIColors.lua"`.
> (Author's reply to a user who still saw the default menu: put the two `cpk.root` lines above all others.)

## Mapping

Same shape as `menu-barcelona-2027`. "Above all others" is what fl-modman does for every new `cpk.root`.

## Archive layout and import

`content/`, `livecpk/{MenuC1987,UIColors}/`, `modules/UIColors.lua` at the top [U]:

```sh
fl-modman import MenuC1987.rar --root . --type loose --manifest catalog/menu-c1987-fl26/mod.toml
```

`--manifest` uses this `mod.toml` instead of the generated one.
