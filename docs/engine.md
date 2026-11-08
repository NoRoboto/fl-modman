# Engine: core + CLI (slices 1 and 2)

Linux only, FL27 only, one install, one ordered mod list where the **lower mod wins** (`grilling.md`). The CLI is the
engine behind the future GUI and a dev/test tool, not a polished user interface.

## On-disk layout

`fl-modman init <game>` creates the workspace next to the game (`<game>/../fl-modman`, or `-w <dir>`):

| Path | Contents |
|---|---|
| `state.toml` | `game`, `prefix`, `base` (snapshot file), `mods` (ordered list), `disabled` (set). |
| `base-<id>.json` | Base snapshot: path → BLAKE3 + size, `sider.ini`/`map_teams.txt` text, exe SHA-256. Taken as-is at init; hand-installed mods are frozen into it (Q8). `accept-drift` writes a new one. |
| `mods/<id>/` | Payload + `mod.toml` (`docs/mod-toml.md`). The id is the folder name. Immutable once imported. |
| `mods/<id>/index.json` | Payload index: path → BLAKE3 + size, written at import (Q20a). Plan, conflicts and the GUI read it and never rehash payloads; a mod without one gets it built once on first load. |
| `originals/<path>` | A base file, kept once just before a mod first replaces it. Undo copies it back. |
| `history/NNNN.toml` | A generation: `status` (`in-progress`, `complete`, `rolled-back`), `note`, `base`, `mods` (the mods in effect), `files` (paths whose expected content differs from the base). Base + `files` = the full expected tree. |
| `hashes.json` | Hash cache: path → size, mtime, BLAKE3. Files are rehashed only when size or mtime change; rewritten only when it changed. |
| `staging/` | Import scratch space. Deleted after a successful import. |
| `settings.dat.bak-<ms>`, `GRAPHICS000000.bak-<ms>` | A copy of each file a `settings set` changes, taken before it writes. |

## Commands

```sh
fl-modman detect                         # candidates: game folders (FL_2027.exe) and prefixes with SPGAME/Football Life
fl-modman init <game> [--prefix <pfx>]   # workspace + base snapshot
export FL_MODMAN=<workspace>             # or pass -w <workspace> to every command below
fl-modman import <archive|dir> [--type livecpk|lua|stadium|download|loose] [--team <id>[,<slot>]] [--root <path>]
                 [--manifest <mod.toml>]  # e.g. catalog/<id>/mod.toml instead of a generated one
fl-modman list | enable <id> | disable <id> | move <id> <index> | remove <id>
fl-modman usage                          # disk usage per mod, totals, reclaimable, filesystem warning
fl-modman remove --disabled [--yes]      # every disabled, unapplied mod + staging/ leftovers
fl-modman apply [--yes]                  # without --yes: print the plan only
fl-modman history | rollback <N> [--yes]
fl-modman recover --finish | --rollback  # after an interrupted apply
fl-modman accept-drift | restore
fl-modman check                          # rehash every mod payload against its index.json (read-only)
fl-modman settings get | settings set key=value ...
fl-modman doctor                         # Proton launch options (never edits Steam files)
```

**Import.** zip and 7z in process; rar via the first of `7z`, `7zz`, `unrar`, `bsdtar` that succeeds. Root detection
descends single wrapper folders, ignores readme/image files, and stops at the first level that looks like one of the
five types (Q7), case-insensitively, the most confident kind winning: `download/*.cpk` names present in the base →
download; `.lua` files → lua module (every top-level `.lua` becomes a `lua.module`); a `stadium*` entry, or livecpk
folders next to a Stadium Server `AddOn/` → stadium (needs `--team`; the slot defaults to the team's base `map_teams`
row); a livecpk top folder (`targets/fl27.toml` `livecpk_dirs`) → livecpk; a `.dll`/`.asi` or a file named like a
game-root file → loose. Otherwise it fails with the candidate roots for `--root`. `--team` on anything but a generated
stadium manifest is an error. `--manifest` copies the given `mod.toml` instead of generating one; it must load against
the payload (every `put.from` matches a file). The mod is built in `staging/` and moved to `mods/` only once its
`mod.toml` loads; it is enabled at the bottom of the list.

**Apply.** Refuses while an `in-progress` generation exists (every command does), while the game runs (a process
whose name or argv[0] file name is `FL_2027.exe`), and on drift. Then: check every source exists with the size the
plan expects (a stat, not a hash; a full rehash is `check`), write
`history/NNNN.toml` as `in-progress`, apply deletes → puts → documents file by file, mark `complete`. Every write is
temp file + rename, never in place (the LAB instance shares hardlinks with the stable install). `.cpk .acb .awb .ftex
.dll` are hardlinked when on one filesystem; everything else is copied; never symlinks. Folders emptied by a delete
are removed.

**Drift.** Every path of the last complete generation's expected tree, and every path the plan would touch, must
match the disk. Plans stat only the paths fl-modman knows (base, last generation, every mod's puts, the documents),
never walking folders: files added by hand outside them are not seen (and never deleted). `FL_MODMAN_TIMING=1`
prints the time of each phase to stderr. `drift_ignore` globs (Sider module `.ini` files rewritten at runtime) are never drift and keep their
runtime content while the wanted content does not change. `accept-drift` takes the drifted files as the new base
(their stale `originals/` are dropped) and records an `accept-drift` generation; the next apply re-applies the mods. A
new exe hash disables exe-guarded mods with a warning. `restore` re-applies the last complete generation; it fails
for changed base files that were never kept (use `accept-drift`).

**Undo.** `rollback N` re-applies generation N's mods through apply (with the drift check) and makes them the list,
the other mods disabled. Generations made before an `accept-drift` cannot be rolled back to.

**Settings.** `{prefix}/drive_c/users/*/Documents/SPGAME/Football Life/settings.dat` (`crates/fl-core/src/codec/settings_dat.rs`):
`resolution=WxH fullscreen fps_game=30|60 fps_replay vsync=off|on1|on2 hdr sound_buffer xinput p2p udp_port auto_udp
voice_chat` (flags are `on|off`). Magic `WECF` is checked; every other byte is kept. The in-game rendering options are
`2027/save/GRAPHICS000000` next to it (`crates/fl-core/src/codec/graphics.rs`, 19 bytes, checked): `preset=low|medium|high|custom
render_scale=50..100 texture_filtering ambient_occlusion surroundings grass spectators=low|medium|high
shadows lens effects goal_nets=low|high depth_of_field motion_blur bloom postprocessing cloth=on|off`. Like the game, a preset writes
its whole row and any other field sets `custom`; bytes 0–2 are never written. `get` prints settings.dat keys, then
these (or says the file is missing: the game writes it once its rendering menu was opened); `set` takes keys of both,
checks every pair before writing either file, and refuses render keys while the file is missing.

**Usage.** `Workspace::usage` (`workspace/usage.rs`) walks the workspace with `symlink_metadata` only: no hashing,
symlinks never followed. Per mod: files and bytes under `mods/<id>/`, and **shared** bytes: a payload file whose
(device, inode) equals the file at its tree path in the game, i.e. the hardlink apply made. Only applied mods are
checked (one extra `lstat` per put). `nlink > 1` was not used: it would also count links fl-modman did not make, and a
payload another mod overrides. `only_here = bytes - shared` is therefore what the workspace adds on top of the
game: files outside `hardlink_ext` (copied on purpose: the game or a Sider module may rewrite `.ini`/`.lua` in
place), payload files a lower mod overrides (not placed at all), and every unapplied mod. Totals for `mods/`, `originals/`, `history/`, `staging/` and the whole folder;
**orphans** are real folders under `mods/` that are not in the list. A symlinked or missing mod folder, a mod whose
`mod.toml` does not load (shared = 0) and a missing game folder (`same_fs` assumed) become `notes`, not errors.
**Reclaimable** = mods that are disabled and not in the last complete generation (the `remove` rule,
`Workspace::removable`), orphans and `staging/`. `same_fs` compares the `st_dev` of the workspace and the game: on
different filesystems apply copies (hardlinks need one filesystem; btrfs subvolumes count as different), so every
applied byte is stored twice. Usage works during an interrupted apply (the CLI allows it before the pending check).

**Remove.** Mod ids must be one plain path component. `remove <id>` and `remove --disabled`
(`Workspace::remove_disabled`) are refused while the game runs or an apply is interrupted. `--disabled` without
`--yes` prints the list (mods, orphans, staging); with `--yes` it deletes exactly the ids it printed: each is checked
again and kept with a reason if it no longer qualifies, a failure keeps that item and the rest go on, and the
command exits non-zero if anything was kept. A symlink (`mods/<id>`, `staging`) loses only the link. `staging/` is
not wiped, and an orphan folder is neither listed nor deleted (a note says so), while anything in it changed in
the last 10 minutes (`QUIET`: an import may be running).

## Code

Hexagonal, in `crates/fl-core/src` (grilling Q20b). The CLI (`fl-cli`) and the GUI (`fl-gui`) are thin adapters over
it. The public API stays flat (`fl_core::{build, plan, manifest, import, settings, ...}` re-export the layers).

| Layer | Files | Contents |
|---|---|---|
| `domain/` | `paths target base build plan manifest hash` | Pure model: no filesystem, no processes. `targets/fl27.toml`, tree manifests and the base snapshot type, `mod.toml` → `Package` (`Package::new` validates), build (desired tree, documents, conflicts), plan. |
| `kinds/` | `livecpk lua stadium download loose` | One `ModKind` per mod type: `detect` (confidence that an extracted root is this kind), `contribute` (payload layout, puts, `cpk_root`/`lua_module`/`map_teams`), `describe` (label, `--type` value). `KINDS` is a static registry; no runtime loading. |
| `codec/` | `sider_ini map_teams settings_dat graphics` | Byte-exact codecs (`render(parse(x)) == x`; binary files write only documented bytes). |
| `ports.rs` | | `Host`: the game-running check and archive extraction, the two things a Windows build would replace. |
| `adapters/` | `linux fs detect` | The Linux `Host` (`/proc`, zip/7z, system rar tools), the filesystem edge (atomic writes, hardlink-or-copy, walk, hashing: plain functions, `std::fs` is already portable), game discovery. |
| `workspace/` | `mod apply scan index import settings usage` | State, history and generations; apply/recover/restore/rollback/accept-drift with the drift check; cached scans and the base snapshot; `index.json` and package loading; import; `settings.dat` + `GRAPHICS000000` files; disk usage. |

**Add a mod kind:** write `kinds/<kind>.rs` with a unit struct implementing `ModKind` (`detect` returns 0 or a
confidence: download 50, lua 40, stadium 30, livecpk 20, loose 10; `contribute` usually calls `folder`), add
`<kind>::<Kind>` to the `registry!` list in `kinds/mod.rs`, and add an import test. Build, plan and apply need no change.

```sh
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check
```

Tests run on synthetic trees copied into the cargo tmpdir (never a real game): codec round trips, fixture plans,
conflicts, T5/T7 property tests (seeded xorshift), import of each type (wrappers, ambiguity, zip, 7z, a Stadium Server folder, `--team` misuse, `--manifest`), apply →
disable all → rollback byte-identical round trip, crash recovery, drift refuse/restore/accept, hardlink vs copy, the
running-game check, a synthetic 612-byte `settings.dat` and 19-byte `GRAPHICS000000` (the spec's preset rows byte for byte), discovery on a synthetic home, the payload index (plan unchanged by a same-size payload edit that `check` reports,
lazy index migration), and one CLI end-to-end run. `cargo test --release -p fl-core --test bench -- --ignored
--nocapture` times plan on a synthetic 16,000-file / 1 GiB mod (`FL_BENCH_FILES`, `FL_BENCH_KIB`).

## Removed earlier (grilling Q13)

before/after constraints, capabilities, params/`when`, seeds, `[opaque]`, `[[delete]]`, refusal kinds, multiple
targets/instances/loadouts, `manifest check`. Slice 2 also dropped the slice-1 `scan`/`plan --json` commands
(`apply` without `--yes` prints the plan) and `loadout.toml`.

## Known issues (found on device 2026-11-02)
- `import --manifest` still demands `--type` when detection is unsure; it should take the kind from the manifest.
- `import --root <wrapper> --manifest catalog/boot-root-october-2026/mod.toml` fails with `from "boot-root/" matches no file`; the payload layout under `--root` and the catalog manifests disagree. `catalog/boot-root-october-2026/SOURCE.md` also omits the archive wrapper folder `boot-root october. 2026/`.
- Failed imports leave the extracted archive in `staging/`; they should clean up (it saved the user once, but it is accidental).
