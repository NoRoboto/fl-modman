# PES 2021 Commentary English Callnames v20

- Author: predator002. Posted 2025-09-03. Two archive parts (modsfire), names not given.
- URL: https://pes-files.com/pes-2021-commentary-english-callnames-v20/
- Type: livecpk. FL27: **claimed for an older FL** ("PES 2021 and Football Life 2025"; needs Sider 7.3.3, which FL27
  most likely ships, install-rules F10 [U]). It adds English player call names, so it only matters with the English
  commentary slot, and it shadows `mariano-closs-v5-eng` there (list order decides).

## Original instructions (brief)

> "Delete all old files related to call names!!!" Extract the CPK contents to a temporary folder using CRI Packed
> File Maker. Create a `preds-root` folder inside `Sider/LiveCPK` and copy into it. Add `cpk.root = ".\livecpk\preds-root"`.

## Mapping

- The extracted tree → `@livecpk/preds-root/` + `cpk_root`.
- "Delete old call-name files": older call-name packs are separate mods; disable them (an older `preds-root`
  installed by hand is part of the base, and this mod overrides it file by file).
- The CPK extraction is a manual step before import (a CRI cpk unpacker; fl-modman does not unpack `.cpk`).

## Archive layout and import

Extract both parts, then unpack the `.cpk` into a folder named `preds-root`:

```sh
fl-modman import preds-root --root . --type livecpk --manifest catalog/english-callnames-v20/mod.toml
```

The payload lands at `mods/<id>/preds-root/`; `--manifest` uses this `mod.toml` (the generated one is the same).
