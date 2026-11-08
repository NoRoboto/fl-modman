# Real Faces FL26/27 (SmokePatch facepack)

- Author: SmokePatch. Released 2025-10-15, update 4 on 2026-05-12.
- URL: https://www.pessmokepatch.com/2025/10/faces2627.html
- Type: `download/*.cpk` replacement. FL27: **claimed** ("SP Football Life 2026 / 2027" on the page). On the Ally,
  FL27 ships `download/01_smkdb_fa1..8,faA.cpk` as 6,272-byte placeholders, and the FL26 `fa1..7` cpks were copied
  over them; in-game face display after that is not recorded.

## Original instructions (brief)

> Extract the CPK files and copy them into the Football Life download folder, replacing the existing ones.
> Update: extract `01_smkdb_fa7.cpk` and replace the existing file in `game/download`.
> (CPK 1–6 main pack, 7 accumulative updates, 8 next season.)

## Mapping

One folder put into `@download/`. Each cpk replaces the base placeholder of the same name; fl-modman keeps the
placeholder in `originals/`, so disabling the mod restores it. An update is the same manifest with only `fa7`.
The names of cpks 1–6 and 8 are shown only in a picture on the page; `01_smkdb_fa<N>.cpk` is inferred from the FL27
base file names [U]. A cpk whose name is not in the base becomes a new file that the game most likely never loads (it is not
listed in `spFileList.bin`, whose format is [U]), so check the plan lists only replacements.

## Archive layout and import

Multi-part archive; the cpks at its top level [U]. Extract the parts first (one folder with the cpks), then:

```sh
fl-modman import <folder-with-the-cpks> --manifest catalog/smokepatch-real-faces-fl2627/mod.toml
```

Detection sees cpk names present in the base (download type); the payload lands at `mods/<id>/download/` with the
base's spelling, and `--manifest` uses this `mod.toml` (the generated one is the same).
