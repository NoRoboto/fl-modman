# Next Gen Graphics 2026

- Author: Moiduran2 (post by G. Leroy, 2026-04-03). Download labelled "Graphics" (Mega); archive name not given.
- URL: https://www.pesmodding.com/2026/04/pes-2021-next-gen-graphics-2026-by.html
- Type: livecpk. FL27: **verified** (imported with fl-modman on the Ally, stable and LAB workspaces, 2026-11-02;
  this manifest is the one the import produced there).

## Original instructions (brief)

> Copy the "Next Gen Graphics" folder into your Sider 7 livecpk folder (FL26: `SiderAddons/livecpk`). Add
> `cpk.root = ".\livecpk\Next Gen Graphics"`. Copy the "AddOn" folder into the stadium folder you want to use
> (Konami stadiums and stadiums with ID 009).

## Mapping

- The livecpk folder and its root: mapped.
- `AddOn/`: not mapped. It is written into another stadium's folder (a cross-ownership write, install-rules R7);
  for a Sider stadium, add it to that stadium's own mod instead.

## Archive layout and import

`Next Gen Graphics/Asset/model/{bg,light}/…` [L, Ally] and `AddOn/` [V, page]:

```sh
fl-modman import <archive> --root "Next Gen Graphics" --manifest catalog/next-gen-graphics/mod.toml
```

The payload lands at `mods/<id>/Next Gen Graphics/`; `--manifest` uses this `mod.toml` (the generated one is the same).
