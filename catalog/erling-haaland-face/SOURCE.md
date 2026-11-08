# Erling Haaland (short hair) face

- Converted from eFootball 2026 by FaceZone Pes (post by G. Leroy, 2026-10-25). Archive `Erling_Haaland.rar`.
- URL: https://www.pesmodding.com/2026/10/pes-2021fl-2026-erling-haaland-short.html
- Type: livecpk (shared `Faces` root). FL27: **claimed** ("PES 2021 & SP Football Life"; FL26 in the title).
  The player id is not given; FL may use other ids than PES 2021 for some players [U].

## Original instructions (brief)

> Sider 7 version: find the folder with the new face ("usually identified by numbers"), copy it to
> `...\livecpk\Faces\Asset\model\character\face\real`, and add `cpk.root = ".\livecpk\faces"` "if not already done".
> (The CPK version needs CriPackedFileMaker + DpFileList Generator: not used.)

## Mapping

- Every numbered face folder → `@livecpk/Faces/Asset/model/character/face/real/`, without naming the id.
- `cpk_root` is the shared `.\livecpk\Faces`: other face mods add the same value, which goes into `sider.ini` once
  (values are compared case-insensitively). Two face mods with the same id conflict file by file; the lower wins.
- The page writes `faces`; the folder is `Faces`. Wine resolves either case, and the manifest keeps one spelling.

## Archive layout and import

The numbered face folder at the top of the archive [U]. Import it verbatim:

```sh
fl-modman import Erling_Haaland.rar --root . --type loose --manifest catalog/erling-haaland-face/mod.toml
```

`--type loose` only copies the tree verbatim to `mods/<id>/game/`; `--manifest` uses this `mod.toml` (the generated
one would put the folder in the game root). If the archive wraps the face folder, pass that wrapper
as `--root`.
