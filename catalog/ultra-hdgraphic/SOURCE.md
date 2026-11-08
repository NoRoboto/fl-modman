# ultra_hdgraphic (stadium graphics)

- Author / URL / version: not recorded. The folder comes from the local FL26 install
  (`Football Life 2026/SiderAddons/livecpk/ultra_hdgraphic`, no readme); a web search found no page.
- Type: livecpk. FL27: **unknown**. Imported into the Ally LAB workspace with fl-modman (2026-11-02); the in-game
  result is not recorded. It ran on FL26 [L].

## Original instructions

None. On FL26 it was registered as `cpk.root = ".\livecpk\ultra_hdgraphic"`.

## Mapping

Folder put + `cpk_root`. It replaces per-stadium models under `Asset/model/bg/stNNN/` (`light`, `pitch`,
`sourceimages`, `effect`, `#Win`), so it may overlap `next-gen-graphics` (which also ships `Asset/model/bg/`) [U]: list order decides
(the lower mod wins), and the plan reports the shadowed files.

## Archive layout and import

`ultra_hdgraphic/Asset/model/bg/stNNN/…` [L]:

```sh
fl-modman import <folder-or-archive> --root ultra_hdgraphic --manifest catalog/ultra-hdgraphic/mod.toml
```

The payload lands at `mods/<id>/ultra_hdgraphic/`; `--manifest` uses this `mod.toml` (the generated one is the same).
