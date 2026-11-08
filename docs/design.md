# fl-modman: design

Date: 2026-11-02. Status: design, pre-implementation.
Evidence base: `research/install-rules.md` (cited **IR §n**, facts **F1–F10**, corpus rows **#n**, refusals **R1–R7**,
theorems **T1–T8**) and `research/state-of-the-art.md` (cited **SotA §n**). Product decisions: `docs/grilling.md`.
Tags: **[U]** = unverified, must be checked before the dependent feature ships.

## 1. Goals, non-goals, success criteria

**Goals**

- G1. Declarative state: the tree a user sees is `Build(Σ₀, E, ≺, θ)` (IR §4.5): an immutable base, plus an ordered
  set of enabled manifests, plus parameters. Every mutation recomputes the tree from these inputs, so disable and
  rollback are exact (T5). No tool in SotA §1 works this way. All of them edit `sider.ini` and copy files in place,
  which T3/T4 show cannot be uninstalled correctly in general.
- G2. One product on Windows 10/11 and Linux (desktop, Steam Deck, ROG Ally under Bazzite, run as a non-Steam game).
  On a plain Windows PC it should behave like a normal mod manager (README decision).
- G3. Conflicts are shown before anything is written. A terraform-style plan explains every write, every conflict
  winner and every refusal.
- G4. Mods are adopted, not lost. An install that already holds hand-installed mods, like ours on the Ally, is
  imported into manifests without reinstalling the game.

**Non-goals** (each with its reason)

| Non-goal | Reason |
|---|---|
| Downloading or hosting mods, scraping Sharemods/Modsfire | Hosts are low-trust (SotA §1) and redistribution rights are unclear. The user supplies archives. |
| A VFS or API hooking (MO2 USVFS) | Not available natively on Linux (SotA §2). Sider's ordered roots already give priority semantics (F1). |
| Packing or editing CPK contents | Out of scope. The unit of work is a file, a document entry, or a captured diff. |
| Decky plugin, in-game menu, macOS | README decision. Sider Mod Menu already covers the in-game case (SotA §1). |
| Managing save data beyond one-shot seeds | Saves are 𝒫_user, and the game rewrites them (R5). |

**Success criteria** (measurable; each one is checked by an automated test or a scripted run on the Ally)

| # | Criterion | Measure |
|---|---|---|
| S1 | Exact reversibility | For random sequences of enable/disable/reorder actions (property test, ≥10k cases on fixture trees), the realized tree hash equals `Build` of the final `(E, ≺)`. With E = ∅, it equals Σ₀ byte for byte on 𝒫_base ∪ 𝒫_mgd. |
| S2 | User data untouched | Hashes of the 𝒫_user globs are identical before and after every apply and rollback, apart from confirmed seeds, which are journaled. |
| S3 | Crash safety | Kill -9 at any journal step, then restart, leaves the tree equal to either generation n or n+1, never a mix (fault-injection test). |
| S4 | Speed on the Ally | A no-op plan takes ≤ 2 s on FL27 with 10 mods (stat-based drift check). Apply time is ≤ 1.2× a plain copy of the changed bytes. |
| S5 | Gamepad-only operation | The full flow (pick instance, toggle, reorder, plan, apply, rollback) can be completed in Gaming Mode on the Ally with no keyboard or touch. |
| S6 | Migration completeness | After adoption, every path and document entry that differs from the FL27 reference is owned by exactly one manifest or by 𝒫_user (§10). |
| S7 | Cross-platform | The same release passes the S1–S3 suites on Windows 11 (NTFS) and Linux (btrfs and ext4). |

## 2. Architecture and stack

```
fl-modman-core (Rust lib)   model, codecs, Build, planner, deployer, discovery, capture
   ├─ fl-modman (CLI)       every operation, plus `--json` for machine use
   └─ fl-modman-gui (Tauri) thin shell; calls core commands over Tauri IPC; never holds state
```

All state lives on disk (§3). The CLI and the GUI are peers over the same core API. Because `--json` is a stable
contract, a different GUI could also drive the core over a process boundary.

| Criterion | Rust + Tauri 2 (chosen) | Go + Wails 2 | .NET 8 + Avalonia 11 (runner-up) |
|---|---|---|---|
| Windows artifact | one .exe. Needs WebView2, which ships with Win11 and comes via a bootstrapper on Win10 | same as Tauri | single-file self-contained, larger [U size] |
| Linux artifact | AppImage that bundles WebKitGTK (large) | same | AppImage. Skia rendering, no webview |
| Headless CLI | small static binary, no runtime. Works over SSH on the Ally | small static binary | Native AOT possible [U for the full feature set] |
| Gamescope / handheld rendering | WebKitGTK under gamescope has known rendering issues **[U, spike in Phase 0]** | same risk | lowest risk |
| Gamepad | `gilrs` in the core, events forwarded to the UI. Independent of webview Gamepad API support | weaker libraries | SDL through Silk.NET |
| FS primitives (reflink, hardlink, `MoveFileExW`, VDF, registry) | mature crates | adequate | adequate |
| Contributor access | Rust core is a barrier. The UI is TS/HTML, which is easy | easiest language | C#, matching the existing PES tools (.NET 4.5, SotA §1) |
| Fit for the formal model | exhaustive enums for primitives and σ, `proptest` | weaker typing | records and pattern matching, FsCheck |

**Decision: Rust core and CLI, Tauri GUI.** The reasons:

- The headless CLI is a first-class deliverable: we operate the Ally over SSH, and scripting `plan`/`apply` matters.
  Rust gives it with no runtime.
- The correctness-critical part (Build, journaled deploy) benefits from exhaustive matching and property testing.
- A web UI makes a 10-foot, focus-driven layout cheap to iterate.

**Runner-up: .NET + Avalonia.** It wins on rendering risk and on familiarity for the community. It loses on CLI
footprint. **Exit criterion:** if the Phase 0 Tauri spike fails S5 in Gaming Mode on the Ally, the GUI becomes
Avalonia, driving the Rust core over the `--json` contract. The core does not change.

## 3. Data model

Naming. IR §6.1 calls the game description a "profile". Here that is a **target**, because "profile" is the
user-facing word for a mod set. The terms used throughout:

| Term | What it is | Stored as |
|---|---|---|
| Target | A game build family: roots, document registry, 𝒫_user globs, and an exe-hash → version table, plus a sider.dll-hash → Sider version table | `targets/fl27.toml`, shipped with the app (IR §6.1 schema) |
| Instance | One concrete install: game path, Wine prefix (Linux), target, base snapshot id. **Stable and LAB are two instances.** | `<store>/instances/<name>/instance.toml` |
| Loadout | The user's intent for an instance: E, ≺ (an ordered list) and θ (parameters). An instance can hold several and has one active. | `[loadout.<name>]` tables in `instance.toml`. Text, diffable (packwiz model, SotA §2) |
| Package | One mod version: `mod.toml`, a file index (path → blob), and provenance | `<store>/pkgs/<id>@<ver>+<b3:8>/` |
| Base snapshot | Σ₀: a full-tree hash manifest plus the parsed base documents | `<store>/bases/<snap>.toml`. Blobs are pinned in the CAS |
| Generation / lock | The realized result: every managed path → (blob, deploy method), every rendered document → hash, the inputs (snapshot, package hashes, ≺, θ) | `<store>/instances/<name>/gens/NNNN.lock.toml`. `lock.toml` points at the current one |

**Store.** The store is a content-addressed store (BLAKE3, read-only blobs: `blobs/b3/ab/abcd…`). Its default
location is `<parent of game dir>/.fl-modman/`: one store per volume, shared by every instance on that volume, so
`~/Games/FL27` and `~/Games/FL27-LAB` deduplicate. The reasons:

- hardlinks and reflinks need the same volume;
- renames into the game tree must be atomic, which also requires the same volume;
- the game folder itself stays clean.

Original archives are kept in `archives/` (optional, on by default). A corrupted blob can then be re-extracted, and
provenance can be checked.

Hashes: SHA-256 only for exe/dll identity, to match `exe_sha256` (IR §6.2) and published hash lists. BLAKE3
everywhere else (speed on large CPKs).

**Hash cache.** `(dev, inode, size, mtime_ns) → b3`. Drift checks only re-hash files whose metadata changed (S4). The
LAB was made with `cp -al`, so its assets share inodes with stable, and its first capture is almost free.

**Generations.** These follow the Nix model. Apply and rollback each create a new generation. Rollback to k means
"rebuild from gen k's lock". It is always possible because the base and the blobs are immutable. GC keeps the last 10
generations, plus pinned ones, plus every installed package.

## 4. The immutable base

**Capture (clean install).**

1. Hash the full tree into `bases/<snap>.toml`, excluding 𝒫_user.
2. Parse every registered document (IR Def. 1) and store its entries and trivia.
3. **Vault** = pin the base blob of every path that some package PUTs or DELs. Pinning happens lazily, at the first
   plan that touches the path and before the commit.

The vault is not a separate copy. It is the CAS holding the base bytes, ingested as a reflink, a hardlink or a
copy (§5.4). A hardlink is safe here because deploys always rename and never write in place, so the original inode is
retired, not mutated.

**Adoption (install already modded, as on the Ally).** Σ₀ must be the *pristine* base, or disable cannot restore the
replaced originals (T3). The flow:

1. **Reference.** Get a clean reference for the exact base version. The preferred source is to run the FL27 base
   installer (plus updates) into a throwaway tree with the capture machinery (§7). The result is an exact Σ₀
   including the originals. The fallback is a published reference hash list for that version (hashes only, no game
   files). That fallback does not supply the bytes of replaced files.
2. **Classify** every path against the reference:

   | Class | Meaning |
   |---|---|
   | clean | the hash matches the reference |
   | replaced | the hash differs |
   | added | not in the reference |
   | removed | in the reference, missing on disk |
   | user | matches the 𝒫_user globs |

   Every registered document is diffed at entry level against the reference document.
3. **Assign.** The user (helped by §8 heuristics and §10 sketches) assigns each non-clean path and entry to a new
   manifest. Unassigned leftovers go into one `local-<instance>` manifest, so nothing is silently lost (IR §5.4,
   "hand edits are imported").
4. **Taint.** If no reference bytes exist for a replaced file, Σ₀ records the current bytes and marks the path
   `tainted`. The owning manifest can still be disabled, but the plan warns that disabling it restores the modded
   bytes, not the original.

**Game updates (rebase).** A new exe hash or a changed `version.txt` puts the instance into `base-outdated`. Every
`apply` is refused (R1 is re-evaluated against the new base) until `rebase` runs. The recommended order is `unapply`
(deploy Σ₀), then run the updater, then `rebase`.

If the updater ran over a deployed tree, `rebase` does a three-way merge per path, using Σ₀, the current lock L and
the disk:

- disk = L(p): the update did not touch p, so Σ₀′(p) = Σ₀(p);
- disk = Σ₀(p): p is unchanged;
- otherwise the update wrote p, so Σ₀′(p) = disk.

Registered documents get the same merge at entry level: Σ₀′(D) = Σ₀(D) plus the entries in (disk − render(L)). Then
guards are re-checked and a new generation is planned. A manifest whose guard now fails is disabled and listed. It is
never dropped silently.

## 5. Build and deploy

### 5.1 Target computation

Build follows IR Def. 5:

1. **Check.**
   - Guards on Σ₀.
   - `requires` within E.
   - `supersedes`: an older version in E is disabled automatically.
   - Capabilities and conflicts.
   - No unresolved `[opaque]`.
   - No `reviewed = false` draft (§8).
2. **Files.** Each path takes the value of its ≺-maximal provider.
3. **Documents.** DELs, then ENTRYs, rendered by the per-document σ:
   - **Ord:** Kahn's algorithm over the hard constraints plus the base chain. ⊤ before all non-⊤. Tie-break by
     `default_side`, then ≺, then key (IR §4.6).
   - **First:** emit by descending ≺.
   - **Last:** emit by ascending ≺.
   - **Multi:** base rows, then rows by ascending ≺, with a multiplicity warning.
4. **Never** write 𝒫_user (seeds excepted).

Output: a target map `path → blob | rendered-bytes | absent`.

Codecs keep trivia (comments, blank lines, CRLF, BOM, encoding) by the lens law ρ∘π = id. `sider.ini` roots are
always written as relative `.\…` paths, never as drive letters. Absolute and `Z:\` paths misbehave under Wine
(SotA §3).

### 5.2 Conflicts, resolution, refusal

| Kind (IR Def. 6) | Handling |
|---|---|
| Value (file, or a First/Last key) | Resolved by ≺ (T6) and shown. `overrides = ["<id>"]` in a manifest marks the conflict as intended (an update over its base pack), so it is not flagged. The build is refused if the overriding manifest ranks lower in ≺ than the one it overrides. |
| Shadow (same virtual path from two roots; F1, F2) | Computed from the root order and the file indexes. Resolved by ≺ through the root order (T6) and shown, because it is invisible at the file level. |
| Multiplicity (Multi document, same key) | Warning. `exclusive = true` on the entry turns it into DEL + ENTRY. |
| Explicit constraint overriding ≺ | Reported as author intent (T6 caveat). |

**Refusals.** Priority cannot fix these, so the plan stops and names the cause:

- R1 guard failure;
- R2 order cycle (the cycle is printed);
- R3 capability clash, declared conflict or missing requirement (the fix is offered: disable one side);
- R4 uncaptured opaque installer;
- R5 a write into 𝒫_user that is not a confirmed seed;
- R6 a meta-level op outside the single `platform` manifest;
- R7 a write into another mod's folder that it did not declare as an extension point.

Operational refusals added here:

- **drift**: the disk differs from the lock outside 𝒫_user. The user must adopt the change into `local-<instance>`
  or discard it;
- **game running**: the FL exe, `sider.exe` or a Wine process with that image is alive;
- **base-outdated** (§4);
- **not writable**: on Windows, Program Files triggers an offer to relaunch elevated, once. No service is installed.

### 5.3 Plan output

The output below is illustrative:

```
$ fl-modman plan stable
instance stable  target fl27 1.00 [exe sha256 3f2a…]  base snap 9c1e…  gen 7 → 8
  + put   SiderAddons/livecpk/Mariano Closs V5/eng/**   412 files 1.1 GiB  reflink   mariano-closs-v5
  ~ put   Data/dt41_spa_all.cpk                          replaces base (vaulted)   com269-spa [captured]
  - del   SiderAddons/modules/BroadCastCam.lua                                   camera-epl disabled
  ~ doc   SiderAddons/sider.ini [cpk.root]   + ".\livecpk\Mariano Closs V5" @2 (front, prio 40)
  ~ doc   content/stadiums/map_teams.txt     + 108, 002, Camp Nou 2026 …   WARN base also maps 108
  ! shadow  <virtual> …/string/all.str   winner machete-cursor (prio 30) over mariano-closs-v5 (prio 20)
  = user paths: 0 writes   seeds: 0
Plan: 413 put, 1 del, 2 docs, 1 conflict (resolved), 0 refusals.
```

### 5.4 Atomic apply and rollback

1. **Lock.** Take the instance lock file. Check the preconditions (§5.2).
2. **Diff.** Diff the target map against the current lock, which gives the minimal PUT/DEL set (IR §5.4 note).
3. **Stage.** Write every new file into `<store>/instances/<name>/staging/<gen>/` and fsync.
4. **Journal.** Write `journal/<gen>.json` (target, staged path, displaced-to path for each op) and fsync.
5. **Commit.** For each op, rename the current file to `trash/<gen>/…`, then rename the staged file into place
   (`MoveFileExW` with `REPLACE_EXISTING | WRITE_THROUGH` on Windows). A DEL is just the first rename.
6. **Finish.** Write `gens/<gen>.lock.toml`, repoint `lock.toml`, and mark the journal committed. The trash is kept
   until the next apply, as a cheap undo.
7. **Recover.** At startup, an uncommitted journal is reversed by replaying the renames backwards (S3).

There is no whole-tree swap, because the game root also holds 𝒫_user files. Hence the per-file journal.

**Deploy method per file**, chosen at stage time:

1. **reflink**, when the filesystem supports it (btrfs/XFS on Linux; ReFS Dev Drive later). Copy-on-write, no
   aliasing.
2. **hardlink**, only for the target's `immutable_assets` extensions (cpk, fpk, dds, ftex, acb, awb, model files).
   This is the same policy as our `cp -al` LAB: assets are linked; text, lua, ini, dll and exe are copied.
3. **copy** for everything else, and as the fallback.

Two rules make this safe. Files are **never written in place**: always temp plus rename. This matters doubly
because the LAB shares inodes with stable. Store blobs are read-only, and `check` re-hashes linked blobs. A corrupted
blob is re-extracted from its archive.

**Symlinks are never used under the game root.** Behaviour under Wine/Sider is untested (SotA §4 Q7), and on
Windows symlinks need Developer Mode or admin.

## 6. Game discovery

Each candidate is a directory containing a target's `exe` and `SiderAddons/`. The version is identified by
`sha256(exe)` against the target table, plus `version_file` (F8) as a cross-check. A known layout with an unknown
hash gives an "unverified" instance: everything works except manifests with `exe_sha256` guards (R1).

| Platform | Sources, in order |
|---|---|
| Windows | (1) Uninstall keys under HKLM/HKCU (both WOW64 views) whose DisplayName matches "Football Life" **[U whether the SmokePatch installer writes one]**. (2) Steam `libraryfolders.vdf` → `steamapps/common/eFootball PES 2021` (AppID 1259970, SotA §3) for vanilla PES 2021 targets. (3) A shallow scan (depth 2) of fixed-drive roots and `%USERPROFILE%\Games`. (4) A user-picked folder. |
| Linux | (1) Steam `shortcuts.vdf` (binary VDF; non-Steam games), resolved to the exe path and the `compatdata/<id>/pfx` prefix. (2) Steam `libraryfolders.vdf` → compatdata for AppID 1259970. (3) Lutris game YAML (`exe`, `prefix`). (4) Heroic `GamesConfig/*.json` and sideload library (`winePrefix`). (5) A scan of `~/Games` at depth 2 (this finds `~/Games/FL27` and `~/Games/FL27-LAB`). (6) A user-picked folder plus prefix (the umu/manual case, which is ours). |

The prefix is needed only for 𝒫_user resolution: `{DOCUMENTS}` maps to
`<pfx>/drive_c/users/steamuser/Documents` for saves (F10), and for capture (§7). Discovery is read-only and every
result is confirmed by the user before an instance is created.

## 7. Opaque installers (capture)

This turns R4 into Lemma 1. The installer runs against a throwaway realization of **Σ₀**, never against the built
tree, so its diff does not depend on E (A1).

1. **Tree.**
   - Linux: `fuse-overlayfs`, with a read-only lower layer = the realized Σ₀ and an upper layer = the capture dir.
     No copy is needed, and deletions appear as whiteouts **[U: Wine on fuse-overlayfs, and its availability on
     Bazzite]**.
   - Fallback: a reflink clone.
   - Last resort: a full copy, after showing the size and time estimate.
   - Hardlink clones are forbidden, because an in-place write would corrupt base and store blobs.
2. **Prefix.** Linux uses a fresh prefix cloned from a template, never the game prefix. The capture tree is mapped
   as drive `X:`, and the UI tells the user to pick `X:\` in the installer, which avoids `Z:\` (SotA §3). Windows
   runs the installer natively against the capture tree. Registry changes are exported before and after
   (`HKCU\Software`, `HKLM\Software\WOW6432Node`) and only **reported**, never replayed.
3. **Diff → manifest.** Changed or new paths become PUTs; removed paths become DELs. Writes matching 𝒫_user become
   `[[seed]]` entries. Writes outside the game tree and the prefix's Documents are listed and dropped, with a
   warning. The diff is checked against the target's document registry: a changed `sider.ini` becomes entry-level
   ENTRY/DEL, not a whole-file PUT.
4. **Provenance.** The generated `mod.toml` gets:

   ```toml
   [provenance]
   kind = "captured"
   installer_sha256 = "…"
   base_snapshot = "…"
   tool = "fl-modman 0.x"
   date = "…"
   deterministic = true   # optional second run produced the identical diff
   ```

   The plan labels the package `[captured]`. A base change (rebase) marks every capture `stale`. Stale captures are
   refused (R4) until they are re-captured.

## 8. Mod packaging

**`mod.toml`** is the IR §6.2 schema, unchanged, plus five fields:

| Field | Purpose |
|---|---|
| `overrides = [ids]` | marks a value conflict as intended (§5.2) |
| `[provenance]` | `authored` / `imported` / `captured` (§7) |
| `reviewed = bool` | drafts from the import helper start `false` and are refused until the user accepts them |
| `[launch] wine_dll_overrides` | advisory text only, for example ReShade's `dxgi=n,b` under Proton. Steam launch options cannot be set reliably, so the app shows the line to copy |
| `[match] archive_b3 = [...]` | lets a published manifest attach automatically to a known archive (Phase 3 index) |

Placement uses logical roots (`@livecpk`, `@stadiums`, …). Server roots that differ between builds are bound
through `binds` (IR §8.6).

**README import helper** (suggests, never applies). It scans the archive for:

- the folder layout (`livecpk/`, `modules/`, `content/`, `download/`, `Data/`);
- `.exe`/`.bat` files, which flag the package `[opaque]`;
- README/txt lines matching `cpk.root = "…"`, `lua.module = "…"`, `^\d+,\s*\d{3},` (map_teams rows), and the phrases
  "at the very top", "above", "below", "before", "after X";
- readme path aliases (`content/stadium-server` → `@stadiums`, `content/kit-server` → `@kits`, F8).

It emits a draft `mod.toml` in which every inferred op carries a `# from README.txt:12` comment and a confidence
level. Ambiguous order phrases become explicit `before`/`after` constraints that the user must confirm. The draft is
`reviewed = false`.

## 9. GUI (handheld and PC)

Screens, all reachable within two inputs:

| Screen | Contents |
|---|---|
| **Instances** | Stable and LAB cards with target, version, generation, drift status, and an "Open loadout" action. |
| **Loadout** | Ordered mod list. Each row shows a toggle, its priority position, badges (`captured`, `conflict`, `stale`, `guard-fail`) and a params chip. Rows are reordered with move-up/move-down. No drag is needed. |
| **Conflicts** | Grouped by address. Shows the winner, the losers, the reason (≺, a constraint, or `overrides`), and refusals with a one-press fix ("disable X"). |
| **Plan / Apply** | The §5.3 diff, grouped by mod and collapsible. Apply uses hold-to-confirm, then shows progress and the result. |
| **History** | Generations with a summary diff. Rollback is hold-to-confirm. |
| **Mod detail** | Provenance, files, document entries, capabilities, params, README. |
| **Settings editor** | The `settings.dat` editor (§10). |

**Gamepad map** (also mirrored on the keyboard, so Steam Input's keyboard template works):

| Input | Action |
|---|---|
| D-pad / left stick | move focus |
| A | toggle / confirm |
| B | back |
| X | detail |
| Y | plan |
| LB / RB | switch tabs |
| LT / RT | page |
| A on a row's handle | enter "move" mode; the D-pad then moves the row; A drops it |
| Start | apply (hold) |
| View | menu |

Layout rules:

- Focus is always visible.
- Targets are ≥ 48 px and text ≥ 16 px at 1280×800 (Deck) and scale up for the Ally's 1080p panel.
- No hover-only affordances. Touch uses the same targets.
- On Linux, gamepad input is read in the core through `gilrs`, so it does not depend on WebKitGTK's Gamepad API.
- In Gaming Mode the AppImage is added as a non-Steam game **without** a compatibility tool, because it is native.

## 10. Migration of the Ally's hand-installed mods

Sketch notation from IR §7 (`P` put, `D` del, `E` entry, `cr` = cpk.root, `lm` = lua.module). The reference Σ₀ comes
from a captured clean FL27 install (§4). Each line is one manifest.

| Manifest | Sketch |
|---|---|
| `smk-facepack-27` | `P download/01_smkdb_fa*.cpk` (replaces base placeholders with the same names, so no `spFileList.bin` edit is needed). |
| `smk-facepack-27-u4` | `P` the update-4 subset of `01_smkdb_fa*.cpk`. `requires smk-facepack-27`, `overrides = ["smk-facepack-27"]`. |
| `com269-spa` | `[opaque] com269_spa.exe` captured → `P Data/dt41_spa_all.cpk`, `P Data/dt43_spa_all.cpk` (the first replaces a base slot, F8). Provides `commentary:spa`. |
| `mariano-closs-v5` | param `slot = "eng"`. `P` sound tree → `@livecpk/Mariano Closs V5/{slot}/…`. `E cr:".\livecpk\Mariano Closs V5"`. Provides `commentary:eng`. Shadow check on `*/string/all.str` (#13). |
| `camp-nou-2026` | `P "Camp Nou 2026/" → @stadiums/…`. `E stadiums.map_teams:(108, 002, …)`. `exclusive` is pending the base row for 108 **[U]**. |
| `machete-cursor` | `P → @livecpk/<cursor root>/`. `E cr:… (front)`. |
| `machete-indicator` | `P → @livecpk/<indicator root>/`. `E cr:… (front)`. |
| `camera-epl` | `P` 4× `*.lua → @modules/`. `E lm:"lib\nesalib.lua"`, then `E lm:<cam> after lib\nesalib.lua` for each module. `seed *.ini` (F7). Provides `broadcast-camera` (exclusive). Kept **in the store, disabled**: this replaces the commented-out lines. |
| `reshade` | param `dll = "dxgi"`. `P dxgi.dll → @game/{dll}.dll`. `P reshade-shaders/`. `seed ReShade.ini` and the preset `.ini` (the overlay rewrites both). `[launch] wine_dll_overrides = "dxgi=n,b"`. |
| *(not a manifest)* `settings.dat` | 𝒫_user, written by the game and by `Settings.exe`, which crashes under Proton (SotA §3). It is handled by a typed editor using the WECF codec (magic "WECF", u32@8 flags, u32@12 width, u32@16 height). Each edit takes a backup and a journal entry, with its own undo outside generations **[U location of the file]**. |
| *(𝒫_user globs for target fl27)* | `{DOCUMENTS}/SPGAME/Football Life/2027/save/**`, `settings.dat`, camera `*.ini` written by modules, `SiderAddons/*.log` and `*.cache`, `ReShade.log`. |

## 11. Phases, risks, open questions

| Phase | Scope | Acceptance |
|---|---|---|
| **0. Spikes** (days) | (a) A Tauri hello-world on the Ally in Gaming Mode with `gilrs` input. (b) Wine reading a `sider.ini` with LF vs CRLF. (c) Wine on fuse-overlayfs. (d) Confirm the Ally `/home` is btrfs (reflink). | Each spike is a yes/no recorded in `docs/spikes.md`. (a) failing triggers the Avalonia exit (§2). |
| **1. MVP: CLI, Linux, FL27** | Targets `fl27`. Instance add (manual path). Base capture and adoption with reference. CAS store. PUT/DEL/ENTRY on `cr`, `lm`, `map_teams`. Build, plan, journaled apply, rollback, drift check, seeds, `settings.dat` editor. | S1, S2, S3, S4, S6 on the Ally. Every §10 manifest except `com269-spa` (which stays in `local-stable` as tainted until Phase 3) round-trips: disable all → tree = Σ₀, enable all → the game boots with each mod observable. The LAB instance works alongside, with shared blobs. |
| **2. GUI + Windows** | Tauri GUI (§9). Windows build (NTFS hardlink/copy, `MoveFileExW`, elevation). Discovery (§6). Loadouts, plus "promote LAB loadout to stable" (same base required, otherwise rebase first). | S5 on the Ally, S7 on Windows 11. Fresh Windows PC: discover → add a mod → apply → rollback with no manual file work. |
| **3. Capture + import** | Opaque capture (§7). README import helper (§8). Rebase three-way merge. Targets `fl26` and `pes2021`. | `com269-spa` captured deterministically (two runs give the same diff). FL27 update → rebase keeps every manifest or lists those it disabled. |
| **4. Ecosystem** | A manifest index matched by `archive_b3` (hashes and TOML only, no mod files). `spFileList.bin` codec. `platform` manifest for Sider upgrades. | Index lookup attaches a manifest to a known archive. A CPK-method facepack (#2) installs via filelist ENTRY. |

**Risks and open questions**

| Item | Impact | Plan |
|---|---|---|
| `spFileList.bin` codec and packed-CPK precedence (IR §8.1, F8) | CPK-method mods (#2) cannot be ENTRY'd | MVP only allows PUT replacements of existing names (the facepack case). Mods that need a new list entry are offered the livecpk variant or refused. The codec comes in Phase 4. |
| Sider 7.3.3 vs 7.4.1 on FL27 (F10, IR §8.4–8.5) | No `save.folder`, so R5 seeds stay irreversible. A Sider upgrade may break `SPcolor.lua` | Identify Sider by sider.dll hash. Guards use that version. A Sider upgrade is only possible as a guarded `platform` manifest (R6). |
| Stadium root naming `content/stadiums` vs `content/stadium-server` (F8, IR §8.6) | Wrong root makes a stadium inert | `binds` on the stadium-server capability. Import aliases (§8). Camp Nou confirms `content/stadiums` on FL27. |
| Duplicate scalar keys in `sider.ini` (IR §8.2) | Unknown winner for `sider.scalar` | Keep `last` [U]. Refuse manifests that ENTRY a scalar the base already duplicates, until this is verified. |
| No clean FL27 reference | Adoption ends up tainted (§4) | Capture the base installer once. Keep the reference snapshot per version. |
| WebKitGTK under gamescope | GUI unusable in Gaming Mode | Phase 0 spike; Avalonia exit. The CLI works regardless. |
| AppImage FUSE on Bazzite [U] | The app does not start | Also ship a plain tarball. Flatpak is rejected because sandboxing blocks `~/Games` and prefixes by default. |
| Hardlink aliasing (game or tool writes in place) | Store corruption | Extension allowlist, temp+rename, read-only blobs, `check` re-hash, archive re-extraction. Reflink is preferred where available. |
| Capture determinism and side effects outside the tree | A capture misses effects | Optional second run. Registry and out-of-tree writes are reported. An unexplained effect → R4 stays. |
| FL27 exe name / `version.txt` path assumed from FL26 (F8) [U] | Wrong target detection | Verify on the Ally before writing `targets/fl27.toml`. |
| Camp Nou row 108 vs a base row for 108 [U] | A random stadium (F5) | Read the base `map_teams.txt` during adoption and set `exclusive` accordingly. |
| Prior art moving fast (SotA TL;DR) | Overlap with closed Windows tools | The differentiators are declarative rebuild, Linux, and an open manifest format. FL26 Mod Studio (MIT, SotA §1) is the reference for `.fl26pack` import and restore points. Reuse its ideas, not its imperative model. |

## Appendix A. Verified facts (2026-11-02, from the live ROG Ally install)

These resolve the `[U]` items above:

| Item | Fact | Source |
|---|---|---|
| FL27 exe name | `FL_2027.exe` (launcher `FL 2027 start.exe`, a .NET bundle needing Windows Desktop Runtime 10) | install tree |
| Version file | `version.ini` (content `27.1.0`), not `version.txt` | install tree |
| Ally `/home` filesystem | **ext4** (`/dev/nvme0n1p7`), so **no reflink** on the Ally; the deploy order there is hardlink (assets) → copy. Spike 0(d) answer: no. | `lsblk -f` |
| `map_teams.txt` row for team 108 | Absent in the FL27 base; the Camp Nou row was appended by hand (backup `~/Games/FL27-src/map_teams.txt.bak`). | base vs current diff |
| `settings.dat` location | `{DOCUMENTS}/SPGAME/Football Life/settings.dat` | game install |
| Why `Settings.exe` crashes under Proton | wine-mono `System.Drawing.Icon.BuildBitmapOnWin32`: "Unexpected number of bits: 0"; `winetricks dotnet48` fails with status 67 | Proton log |
| Proton launch requirement | `WINEDLLOVERRIDES="lsteamclient=d;steamclient64=n;steam_api64=n,b;winmm=n,b"`, otherwise "Unable to create interface ISteamUser" | live test |

## Appendix B. Decision update (2026-11-02): GUI framework

**egui/eframe replaces Tauri.** Plan B is Slint. Avalonia is dropped.

Reasons: one language (Rust) end to end; single native binary with no webview, which removes the WebKitGTK dependency under gamescope; built-in focus navigation that a gamepad can drive (gilrs → focus/activate events); the Rust GUI with the most public code, so LLM-assisted development produces compiling code more reliably. Iced is rejected because its API churns. Slint stays the fallback if the egui look is not acceptable after the MVP.

Spike 0(a) changes accordingly: an eframe hello-world (list of toggles + a button) on the Ally in Gaming Mode, driven only by the gamepad.

## Appendix C. Requirement (2026-11-02): built-in game settings

The user asked for resolution, sound and graphics settings inside fl-modman. This replaces the game's
`Settings.exe`, which crashes under Proton (wine-mono icon bug), so it is a real gap on Linux and handhelds.

Scope is exactly what `settings.dat` holds (layout: `crates/fl-core/src/codec/settings_dat.rs`):
display (resolution, fullscreen), frame rate (in game / replays), vsync mode, HDR, sound buffer size,
controller (XInput), and online (P2P, UDP port, voice chat). Keyboard mapping is out of scope until its
layout is mapped. There is no graphics-quality field in the game, so the UI must not invent one.

Graphics extras that already exist in the tree (ReShade preset on/off) appear in the same screen as ordinary
mods, not as settings. Phase placement: the codec and CLI (`fl-modman settings get|set`) are in phase 1;
the settings screen is in phase 2 with the GUI. Every write keeps a backup and an undo entry.
