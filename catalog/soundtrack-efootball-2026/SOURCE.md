# eFootball 2026 Soundtrack for PES 2021 (Football Life variant)

- Author: baygonstroberi. Posted 2026-01-27. 25 songs.
- URL: https://pes-files.com/efootball-2026-soundtrack-for-pes-2021/
- Type: `download/*.cpk` replacement. FL27: **claimed** ("SP Football Life" variant; the page says any PES 2015–2021
  with AWB audio). FL27 ships `download/SMK_Addmusic.cpk` [L, Ally].

## Original instructions (brief)

> Football Life: extract "eFutBol26 BGM.cpk" into the `download` folder, delete "SMK_Addmusic.cpk", rename
> "eFutBol26 BGM.cpk" to "SMK_Addmusic.cpk".
> Sider: extract "Soundtrack eFootball 2026" into LiveCPK and add `cpk.root = ".\livecpk\Soundtrack eFootball 2026"`.

## Mapping

"Delete + rename" is one put onto the base file's name; the base `SMK_Addmusic.cpk` is kept in `originals/` for undo.
The Sider variant is a separate mod (livecpk folder + `cpk_root`), not in this catalog.
The page spells the file both "eFutBol26 BGM.cpk" and "eFootBol26 BGM.cpk" [U]: use the name in the archive.

## Archive layout and import

The cpk somewhere in the archive [U]:

```sh
fl-modman import <archive> --root <folder holding the cpk> --type download --manifest catalog/soundtrack-efootball-2026/mod.toml
```

The payload lands at `mods/<id>/download/eFutBol26 BGM.cpk`; `--manifest` uses this `mod.toml` (the generated one
would add the cpk under its own name, which the game never reads).
