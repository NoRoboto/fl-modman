# PES 2021 Boots & Gloves Pack (boot-root) October 2026

- Authors: Fallons, DN7 Kits, leom 3219, dogg260 (post by G. Leroy, 2026-10-26). Archive `boot-root_october._2026.rar`.
  90+ boots, 25+ goalkeeper gloves.
- URL: https://www.pesmodding.com/2026/10/pes-2021-boots-gloves-pack-boot-root.html
- Type: livecpk. FL27: **claimed** ("PES 2021 and SP Football Life"). A commenter on the April 2026 pack notes that
  the boot assignment is made on PES 2021 player ids, which FL sometimes changes, so some players may show other
  boots (https://www.pesmodding.com/2026/04/pes-2021-boots-gloves-add-on-pack-april.html; seen in a search snippet,
  page not re-read [U]).

## Original instructions (brief)

> Extract and copy the "boot-root" folder to your Sider 7/livecpk folder. Add `cpk.root = ".\livecpk\boot-root"`.

## Mapping

Folder put + `cpk_root`. A newer monthly pack is the same manifest with a new payload; keep one of them enabled.

## Archive layout and import

`boot-root/…` [V, page]:

```sh
fl-modman import boot-root_october._2026.rar --root boot-root --type livecpk --manifest catalog/boot-root-october-2026/mod.toml
```

The payload lands at `mods/<id>/boot-root/`; `--manifest` uses this `mod.toml` (the generated one lacks only the version).
