# Graphic Menu FC Bayern München 2027

- Post by G. Leroy, 2026-08-07 (menu author not named on the page). Archive `Menu_Bayern.rar`.
- URL: https://www.pesmodding.com/2026/08/graphic-menu-fc-bayern-munchen-2027-sp.html
- Type: livecpk + Lua module. FL27: **claimed for FL26** ("PES 2021 & SP Football Life 2026").

## Original instructions (brief)

> Install CommonLib first, first in your Sider. Copy the "content, livecpk, and modules" folders into your Sider 7
> folder. Add `cpk.root = ".\livecpk\Menu Bayern"`, `cpk.root = ".\livecpk\UIColors"`, `lua.module = "UIColors.lua"`.

## Mapping

Same shape as `menu-barcelona-2027` (see its SOURCE.md for the line placement, the `UIColors` conflict between
menus and the `config.ini` user-path rule).

## Archive layout and import

`content/`, `livecpk/{Menu Bayern,UIColors}/`, `modules/UIColors.lua` at the top [U]:

```sh
fl-modman import Menu_Bayern.rar --root . --type loose --manifest catalog/menu-bayern-2027/mod.toml
```

`--manifest` uses this `mod.toml` instead of the generated one.
