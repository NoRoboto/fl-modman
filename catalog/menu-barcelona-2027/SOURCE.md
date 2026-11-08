# Graphic Menu FC Barcelona 2027

- Author: Diyaad (post by G. Leroy, 2026-07-29). Archive `Menu_Barcelona.rar` (sharemods).
- URL: https://www.pesmodding.com/2026/07/graphic-menu-fc-barcelona-2027-sp.html
- Type: livecpk + Lua module. FL27: **claimed for FL26** ("PES 2021 & SP Football Life 2026").

## Original instructions (brief)

> Needs CommonLib, "the first entry in your Sider". Copy the "content, livecpk, and modules" folders to your Sider 7
> folder. Add `cpk.root = ".\livecpk\Menu Barcelona"`, `cpk.root = ".\livecpk\UIColors"`, `lua.module = "UIColors.lua"`.

## Mapping

- The three folders → `@livecpk/`, `@modules/`, `@sider/content/` (Sider folder = `SiderAddons`).
- New `cpk.root` lines go before the base ones, which is what menus need ("above everything").
- `lua.module = "UIColors.lua"` goes after the base modules, so after FL27's `lib\CommonLib.lua` (already first).
- Conflicts: `UIColors` files and `UIColors.lua` are shared with every Diyaad / Carlos1987 menu
  (`menu-bayern-2027`, `menu-c1987-fl26`): enable one menu. Menus override the FL Switcher theme
  (`SMK_Addtheme.cpk`) by shadowing.
- A put cannot write `SiderAddons/content/*/config.ini` (a user path); if `content/` ships one, loading this manifest
  (`apply`) stops with that path; drop the file from the payload [U whether it ships one].

## Archive layout and import

`content/`, `livecpk/{Menu Barcelona,UIColors}/`, `modules/UIColors.lua` at the top of the archive [U whether
there is a wrapper folder; if so pass it as `--root`]:

```sh
fl-modman import Menu_Barcelona.rar --root . --type loose --manifest catalog/menu-barcelona-2027/mod.toml
```

`--type loose` copies the tree verbatim to `mods/<id>/game/`; `--manifest` uses this `mod.toml` instead of the generated
one. Checked end to end on a synthetic archive of this shape (`apply` plan: 2 roots, 1 module, the puts).
