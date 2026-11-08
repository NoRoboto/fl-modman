# Install rules for SP Football Life / PES 2021 (Sider 7) mods: a formal treatment

Date: 2026-11-02. Scope: FL26 (local evidence), FL27 (partial local evidence), PES 2021 + Sider 7.x (primary docs).
Companion to `state-of-the-art.md`.

## 0. Result in one screen

Every documented install in the corpus (section 2) compiles to **three state operations** over a game tree
whose text configs are parsed into typed documents:

| Primitive | Meaning | Typical mod wording |
|---|---|---|
| `PUT(p, b)` | file at path `p` gets bytes `b` (creates or overwrites) | "copy folder X to ...", "rename to SMK_Addmusic.cpk", "replace dt18_all.cpk" |
| `DEL(a)` | address `a` (a file, a folder, or a config entry) is absent | "delete sider.dll", "remove nets.lua", "comment out the CompMenus line", toggle ReShade off |
| `ENTRY(D, e, C)` | entry `e` is present in config document `D`, subject to order constraints `C` | "add `cpk.root = ...` at the top", "add `lua.module` below CommonLib", "add a line to map_teams.txt", "add the cpk with DpFileList Generator" |

They are complemented by **guards** (preconditions on the base: exe hash, patch version, Sider version),
**capabilities** (exclusive semantic resources, for example "broadcast camera" or "goal nets"), and a
per-document **collision semantics** σ ∈ {Ord, First, Last, Multi}.

Proven below:

- (T1) these three are sufficient for the corpus;
- (T2) none can be dropped (proof by contradiction for each); order constraints and per-document σ are also necessary;
- (T3, T4) in-place incremental patching has no correct uninstall in general, and with backups it is correct only under LIFO uninstall;
- (T5) "rebuild from immutable base + ordered set of enabled manifests" is exactly reversible and path-independent;
- (T6–T8) conflict definition, a deterministic priority-based resolver, and the cases that must be **refused**,
  not resolved: guard failure, order cycles, capability clashes, opaque effects, user-owned paths, and meta-level ops.

---

## 1. Evidence legend and primary sources

Every factual claim carries one tag:

- **[V]**: verified in a cited source text (fetched 2026-11-02).
- **[L]**: verified in a local FL26 install (read-only) or in a local copy of the FL27
  `SiderAddons/modules/` folder.
- **[U]**: unverified or inferred. Treat it as a hypothesis to test.

Primary sources about Sider itself (the program that interprets most installs):

- S1 Sider 7 readme: https://github.com/pes-modding/sider7/blob/master/doc/readme.html
- S2 Sider 7 scripting guide: https://github.com/pes-modding/sider7/blob/master/doc/scripting.html
- S3 Sider 7 release history: https://github.com/pes-modding/sider7/blob/master/doc/history.txt
- S4 stock kitserver module: https://github.com/pes-modding/sider7/blob/master/modules/kserv.lua
- S5 SmokePatch customizations (FL Switcher, modules): https://www.pessmokepatch.com/p/customizations-and-mods.html
- S6 SmokePatch installation: https://www.pessmokepatch.com/2022/10/installation.html
- S7 Sider 7.4.1 release post: https://www.pesmodding.com/2026/04/pes-2021-sider-v741.html

Note: the old `github.com/juce/sider` is gone (HTTP 404). The live repository is `pes-modding/sider7`. [V]

## 2. The machine: what the game and Sider actually read

These facts fix the semantics that the formal model must respect.

- **F1. cpk.root is first-wins.** "If multiple roots are specified, then they are checked in order that they are
  listed in sider.ini. As soon as there is a filename match, the lookup stops. (So, higher cpk.root wins...)".
  Relative roots resolve against the folder of sider.exe. [V S1]
- **F2. Lookup is by existence, not content.** A match on the file name stops the lookup. So a zero-byte file in a
  higher root still shadows the real asset. [V S1]
- **F3. lua.module order is load order**, and modules live under `modules/`. [V S1] For some events
  (stadium choice, for example), a handler that returns non-nil "stops further propagation". So module order is also a
  priority order for those events. [V S2] Libraries must come first: `BroadCastCam.lua` errors with
  "nesalib not found" when `lib\nesalib.lua` is not loaded before it. [L FL27 `modules/BroadCastCam.lua`]
- **F4. Folder presence is not activation.** FL26 has `SiderAddons/livecpk/Faces/` on disk but no
  `cpk.root` line for it, so it is inert. [L `sider.ini`]
- **F5. Keyed tables have different collision semantics:**
  - Kitserver `map.txt`: `map[tid] = path` per line, so **the last line wins**. [V S4; L FL26 `modules/common/kits.lua:465`]
  - Stadium server `map_teams.txt` / `map_competitions.txt`: a **set of rows per key**, exact duplicates
    skipped. When several stadiums map to one team, one is picked at random. [L FL26 `modules/common/stadiums.lua:380-416`, `:563-572`]
  - Scoreboard server `map_competitions.txt`: a **bag of rows per key** (no deduplication). [L FL26 `modules/common/scoreboards.lua:129-160`]
- **F6. Row format of map_teams:** `teamId, stadiumSlot, folder, name # comment`, comma separated, whitespace
  trimmed, BOM stripped. [L]
- **F7. Modules write files at runtime.** nesalib's `save_ini` rewrites the camera `.ini` from the overlay.
  The stadium, kit and scoreboard modules write their own ini or config files. Sider writes `sider.log`,
  `camera.cache` and `startup.cache`. [L `nesalib.lua:154`, `stadiums.lua:184`, `kits.lua:532,766`, `scoreboards.lua:521,549`]
  These are **user-owned paths**: a manager must never regenerate them blindly.
- **F8. The FL base layout differs from vanilla PES 2021 + Sider:**
  - Sider lives in `SiderAddons/`, and `exe.name = "\FL_2026.exe"`.
  - The stadium root is `content/stadiums` (mod readmes say `content/stadium-server`).
  - The kit root is `content/kits` (readmes say `content/kit-server`).
  - The download folder holds `spFileList.bin`, not `DpFileList.bin`.
  - The commentary language slots ship as `Data/dt41_{ags,eng,spa}_all.cpk`.
  - The patch version is in `dataSP/version.txt = 2.2.0`.
  - The FL Switcher state is in `dataSP/{GP,Controller,Classics}.txt`.
  - The Switcher keeps a backup at `Data/dt18_all_bk.cpk`.

  [L] The format of `spFileList.bin` and its equivalence to DpFileList are **[U]**.
- **F9. The FL Switcher swaps whole files.** It replaces `SMK_Addtheme.cpk`, `SMK_AddLUT.cpk`, `SMK_Addcontrol.cpk`,
  `SMK_Addlockdown.cpk` and `SMK_Addreplay.cpk` in `download/`, and `dt18_all.cpk` in the data folder. Theme and
  classic-teams options also swap a Lua file. "Sider custom menus may override this feature." [V S5]
- **F10. Saves.** FL27's save folder is `Documents\SPGAME\Football Life\2027\save`, and FL27 uses a different edit-file
  key. [V S5] Sider ≥ 7.4.0 supports `save.folder`, which redirects EDIT/SYSTEM/etc. to a chosen folder. [V S1, S3]
  The FL27 `sider.dll` is dated 2025-01-19, which equals the Sider 7.3.3 release date. So FL27 likely ships 7.3.3,
  which has **no** `save.folder`. [U, inferred from L + S3]

## 3. Corpus: 26 representative mods (install operations as documented)

The corpus was picked to cover distinct install patterns, not popularity alone. In the "Ops as documented" column:
`cp` = copy, `+ini` = add a sider.ini line, `+row` = add a table line, `rm` = remove.

| # | Mod (type) | Ops as documented | Deps / conflicts / uninstall | Source | Ev. |
|---|---|---|---|---|---|
| 1 | Single face, Sider method (Haaland 2026) | cp `<id>/` → `livecpk\Faces\Asset\model\character\face\real\`; +ini `cpk.root=".\livecpk\faces"` "if not already done" | shared root across all facepacks | https://www.pesmodding.com/2026/10/pes-2021fl-2026-erling-haaland-short.html | V |
| 2 | Facepack, CPK method (Facepack v3; La Liga mini #72) | pack with CriPackedFileMaker; cp `.cpk` → `download\`; register with DpFileList Generator | file-list order "can matter" | https://www.pesmodding.com/2021/09/pes-2021-facepack-v3-sider-and-cpk.html ; https://www.pesmodding.com/2026/09/pes-2021sp-football-life-2026-la-liga.html | V (FL list format U) |
| 3 | SmokePatch facepack cpks `01_smkdb_fa1..8.cpk` | ship in `download\`; patch updates replace them | part of the base, so a base update means a rebase | local tree | L |
| 4 | Premier League 2026/27 kitpack (Kitserver) | cp `Premier League\` → `content\kit-server`; +row `100, "Premier League\Manchester United"` ... in kserv map | needs Sider 7 + Kitserver; "IDs can vary between patches" | https://www.pesmodding.com/2025/10/premier-league-202526-kitpack-full.html ; https://www.pesmodding.com/2026/09/premier-league-202627-full-kitpack-pes.html | V |
| 5 | Mega Kitpack 25/26 (DN7) fonts | +ini `cpk.root=".\livecpk\kitserver special character fonts"` "at the very top ... above everything"; updates `UniNameFontPermissions.bin` | top-priority claim | https://www.pesmodding.com/2026/03/pes-2021sp-football-life-2026-mega.html | V |
| 6 | Referee Kit Server 25/26 | cp `content`,`livecpk`,`modules`; +ini `cpk.root=".\livecpk\RefColor"`, `lua.module="RefKitServer.lua"` | Sider ≥ 6.5 + CommonLib | https://www.pesmodding.com/2025/12/referee-kit-server-season-202526-pes.html | V |
| 7 | Etihad Stadium Expansion V2 | delete the previous version; cp → `content\stadium-server`; +row `173, 009, Etihad Stadium, Etihad Stadium # Manchester City`; optional +row in map_competitions `0, 009, Etihad Stadium, Etihad Stadium, , ,`; turf variant `MA_GFX_v1/v2`; rename `Cutscene_off`→`Cutscene` | requires Banner Server 3.0 + Stadium Server 1.80 | https://www.pesmodding.com/2026/07/pes-2021fl-2026-etihad-stadium.html | V |
| 8 | 56 Extra Konami Stadiums | back up `Data\dt30_g4.cpk` (so it is replaced); +ini `cpk.root=".\livecpk\Stadiums Extra DB"`; disable the Stadium Server module and delete `content\stadium-server` | conflicts with Stadium Server | https://www.pesmodding.com/2025/11/sp-football-life-2026-v12-essentials.html | V (replacement inferred from the backup advice) |
| 9 | Banner Server v3 | cp `content`,`modules`,`livecpk`; +ini `lua.module="BannerServer.lua"` **above** StadiumServer; `cpk.root=".\livecpk\Banner"` "no order" | remove other banner modules | https://www.pesmodding.com/2025/03/pes-2021-banner-server-v3-by-balkan-pes.html | V |
| 10 | World Cup 2026 scoreboard font | cp → `livecpk`; "add the registration from the text"; +ini `cpk.root` | requires Exclusive Scoreboard v15 AIO; Sider 7.4.1 | https://pes-files.com/pes-2021-scoreboard-fifa-world-cup-2026/ | V ("registration" target U) |
| 11 | LaLiga 2026/27 substitution board | cp `common\` → `content\SubBoard\Spain\La Liga` | requires the SubBoard server (FL26 ships `SubBoard.lua` [L]) | https://www.pesmodding.com/2026/10/pes-2021-laliga-ea-sports-202627.html | V |
| 12 | English commentary for FL26 (installer) | run `com269_xxx.exe`, pick the game folder, pick the language in game settings | files written are undocumented | https://pes-files.com/pes-2021-update-english-commentary-for-fl-2026/ | V (effects U) |
| 13 | Mariano Closs V5 commentary (livecpk) | root holds `ags/sound/{acb,awb,config}` + `mex/menu/*`, `mex/string/all.str`, `spa/string/all.str`; +ini `cpk.root` | the language slot is chosen by folder name; ships string tables that collide with menu mods | local tree | L |
| 14 | English callnames v20 | delete old callname files; extract the CPK; cp `preds-root` → `livecpk`; +ini `cpk.root=".\livecpk\preds-root"` | Sider 7.3.3 | https://pes-files.com/pes-2021-commentary-english-callnames-v20/ | V |
| 15 | Custom Commentary Server 2026 | cp `custom-comment-server.lua` → `modules`; +ini `lua.module`; create `content\custom-commentary-server\map.txt` (CSV `stadium, comp, home, away, audioPath`) | Sider 7.3.4 | https://pes-files.com/pes-2021-custom-commentary-server-2026-v1/ | V |
| 16 | Realistic EPL camera 2026 | cp `.lua` + `.ini`; +ini `lua.module` in order `lib\nesalib.lua`, `BroadCastCam.lua`, `FanViewCam.lua`, `PenaltyCam.lua`, `StadiumCam.lua`, `VerticalCam.lua` | nesalib before cams; the `.ini` is rewritten at runtime | https://pes-files.com/pes-2021-realistic-epl-camera-mod-2026/ ; https://pes-files.com/pes-2021-broadcast-camera-manager-update-v1-0/ | V + L |
| 17 | FL26 menu mod (MenuC1987) / Bayern graphic menu | cp `content`,`livecpk`,`modules`; +ini `cpk.root=".\livecpk\MenuC1987"`, `cpk.root=".\livecpk\UIColors"`, `lua.module="UIColors.lua"`; roots "higher than anything else"; "CommonLib must be first" | overrides the Switcher theme (F9) | https://www.pesmodding.com/2025/10/new-football-life-2026-menu-mod-intro.html ; https://www.pesmodding.com/2026/08/graphic-menu-fc-bayern-munchen-2027-sp.html | V |
| 18 | Goalnets module v0.6.1 | cp `livecpk` (SiderAddons on FL); cp `goalnets\` → `modules`; +ini `cpk.root=".\livecpk\goalnets_25-26"`, `lua.module="goalnets\goalnets_25-26.lua"`; optional `goalnets_preset.txt` inside each stadium folder (overrides) | "cannot be used with other nets mods" (`nets.lua`, `nets-server.lua`). FL26 base loads `nets.lua` [L] | https://pes-files.com/pes-2021-goalnets-module-v0-6-1-aio-update-2026/ | V + L |
| 19 | Ballpack AIO 2026-27 / ball-server AIO | "replace the `content\ball-server` folder" (Essentials); the 2026-27 post has no steps | — | https://www.pesmodding.com/2026/10/ballpack-aio-2026-27-update-new-balls.html ; Essentials (row 8) | V / U |
| 20 | ReShade preset (Soccer Revolution 2026 v11; Vibrant Shaders) | run the ReShade installer (d3d10/11) or ship `dxgi.dll`; replace `reshade-shaders\`; cp `*.ini` presets to the game root; "if this doesn't work, rename dxgi.dll to d3d11.dll". Soccer Revolution also ships `dinput8.dll` + `PES21_Hook.dll`, moves the Sider folder, and deletes `sider.dll` | Home key opens the menu | https://pes-files.com/pes-2021-soccer-revolution-2026-graphics-update-v11/ ; Essentials (row 8) | V |
| 21 | Alternative Gameplay Repack / Switcher gameplay | back up `Data\dt18_all.cpk`, extract over it (repack); the Switcher replaces the same file | both write `dt18_all.cpk` | Essentials (row 8); S5 | V + L (`dt18_all_bk.cpk`) |
| 22 | FL Switcher theme / LUT / controller / lockdown / replay | replace one of N variants of `download\SMK_Add*.cpk` (+ Lua for theme/classics) | menus override the theme | S5 | V |
| 23 | eFootball 2026 soundtrack | Sider variant: cp → `livecpk`, +ini `cpk.root`. FL variant: cp `eFutBol26 BGM.cpk` → `download`, delete `SMK_Addmusic.cpk`, rename the new file to `SMK_Addmusic.cpk` | conflicts with in-game music (soundtrack server) | https://pes-files.com/efootball-2026-soundtrack-for-pes-2021/ | V |
| 24 | Turf and Shape Loader v3.3 | extract into the game dir; +ini `lua.module` order `WeatherConditions.lua` < `TurfLoader.lua` < `Shape-turf-loader.lua` < `StadiumServer.lua` | inserts **between** existing lines | Essentials (row 8) | V |
| 25 | Option file 2026/27 | cp `EDIT00000000` → `Documents\KONAMI\eFootball PES 2021 SEASON UPDATE\2026\save`, "replace (backup first)" | the file is user-mutable (game saves) | https://pes-files.com/pes-2021-new-option-file-season-2026-27/ | V |
| 26 | "Road to Football Life 2027" patch | cp `livecpk`,`modules`,`content` into SiderAddons; +ini `cpk.root` lines; `lua.module="SoundtrackServer.lua"` "replaces the existing lua lines"; cp `EDIT00000000` into save | FL26 v2.2 recommended | https://pes-files.com/pes-2021-patch-road-to-football-life-2027/ | V |
| 27 | Carabao Cup (FL26 / PES 2021) | contents undocumented (RAR); FL26: "EXE Version: 2.2 only", "March 2026", not FL27; PES 2021: "DLL is hard-coded ... Version 1.7.2.0" | exe-specific | https://www.pesmodding.com/2026/10/sp-football-life-2026-carabao-cup-mod.html ; https://www.pesmodding.com/2026/10/pes-2021-carabao-cup-mod-by-dre-modding.html | V (payload U) |
| 28 | `narrador.lua` (own) | cp module; +ini `lua.module="narrador.lua"`; writes `narrador-events.jsonl`; rollback = comment out the line | — | local notes | L |

Not covered by any source: uninstall steps. Only rows 8, 21 and 25 (backup advice) and row 28 (rollback)
mention reversal. **Every uninstall is the manager's job.** [V, absence across all fetched pages]

---

## 4. Formal model

### 4.1 State

Let 𝒫 be the set of relative paths under the game root `G`, and let 𝔹 = {0,…,255}* be the set of byte strings.

**Definition 1 (registered documents).** A registered document D ∈ 𝒟 is a tuple
(path_D, π_D, ρ_D, κ_D, σ_D, side_D), where:

- path_D ∈ 𝒫 is the file that holds it. One file may hold several documents: `sider.ini` holds `cpk.root`, `lua.module` and the scalar keys.
- π_D : 𝔹 → Entry* × Trivia parses the file into entries, keeping comments and blank lines as trivia.
- ρ_D : Entry* × Trivia → 𝔹 renders it back, with ρ_D ∘ π_D = id on the managed fragment (the lens law).
- κ_D : Entry → Key is the key function.
- σ_D ∈ {Ord, First, Last, Multi} is the collision semantics (F1, F5):
  - Ord: an ordered set, keyed by the entry itself; order is meaningful (`cpk.root`, `lua.module`, the cpk file list).
  - First / Last: a map in which the first or last row for a key wins (scalars; kitserver `map.txt` = Last).
  - Multi: a key maps to the set or bag of its rows (`map_teams`, `map_competitions`).
- side_D ∈ {front, back} is the default placement of unconstrained new entries (§4.6).

**Definition 2 (state).** Σ = (F, Δ), where:

- F : 𝒫 ⇀ 𝔹 is a partial map of unregistered files, with dom F ∩ {path_D} = ∅;
- Δ : 𝒟 → Entry* gives the entry sequence of each document.

**Realization.** The physical tree is ⟦Σ⟧ = F ∪ { path_D ↦ ρ_D(Δ(D)) }.

**Address space.** 𝒜 = 𝒫 ∪ { (D, k) : D ∈ 𝒟, k ∈ Key }. Then addr(Σ) = dom F ∪ { (D, κ_D(e)) : e ∈ Δ(D) }.

**Partition of paths.** 𝒫 = 𝒫_base ⊎ 𝒫_mgd ⊎ 𝒫_user.

- 𝒫_user holds paths written by the game or Sider at runtime (F7): saves, `*.log`, `*.cache`, module-written ini files, `narrador-events.jsonl`.
- 𝒫_mgd holds every path the manager may write.

*Plain English:* the game tree is ordinary files plus a handful of config files that are understood structurally, as
lists and tables instead of bytes. Everything the game itself writes is fenced off as user data.

### 4.2 Observational semantics

What the game and Sider "see" is Obs(Σ) = (V, L, T):

- **VFS.** V(q) = F(r_i / q) for the least i such that r_i / q ∈ dom F, where (r_1, …, r_n) = Δ(cpk.root) (F1, F2).
  If no root matches, the lookup falls back to the packed cpks, ordered by the file list [U for the order semantics].
- **L** = Δ(lua.module): the module load sequence (F3).
- **T(D)** = the σ_D-quotient of Δ(D): for Multi, a key maps to the set or bag of its rows; for First/Last, a key maps to the winning row.

Two states are equivalent, Σ ≈ Σ', iff Obs(Σ) = Obs(Σ'). (Byte-identical comments and trivia are not required.)

### 4.3 Primitives

Let Π = { PUT, DEL, ENTRY }.

- **PUT(p, b)**, with p ∉ {path_D}: F' = F[p ↦ b].
- **DEL(a)**:
  - for a = p: F' = F restricted to dom F \ {p' : p' = p or p' has prefix p/}. This covers folders.
  - for a = (D, k): Δ'(D) = Δ(D) filtered by κ_D(e) ≠ k.
- **ENTRY(D, e, C)**: Δ'(D) contains e, with C ⊆ { e ◁ x, x ◁ e } a set of order constraints, where x names other
  entries (of D, by key) or the symbol ⊤ ("before every non-⊤ entry").
  - For σ_D = Ord, a duplicate key is a no-op (idempotent).
  - For First/Last, ENTRY adds a row. Which row wins is decided by rendering order (§4.6).

**Guards** are predicates on the base state, not operations, for example:

- sha256(`FL_2026.exe`) ∈ H;
- `dataSP/version.txt` ∈ [2.2.0, 2.3);
- Sider ≥ 7.4.1;
- the presence of a module.

**Capabilities** are tokens. Each manifest declares `provides` and `conflicts` sets of tokens, and certain tokens are
declared exclusive.

### 4.4 Manifests

**Definition 3.** A manifest is m = (id, G_m, Ops_m, Req_m, Prov_m, Conf_m, U_m), where:

- G_m is its set of guards;
- Ops_m ⊆ Π-instances, a finite **set**;
- Req_m and Conf_m are manifest ids with version ranges;
- Prov_m is its set of capabilities;
- U_m ⊆ 𝒫_user is its set of seed targets (§5.4, R5).

**Well-formedness (W1).** No two ops in Ops_m target the same address.

Authors may still write overlays such as "base stadium + turf variant". The compiler flattens them (the later one wins)
before the set is formed. So a manifest's effect is a partial function

eff_m : 𝒜 ⇀ 𝔹 ∪ {⊥} ∪ Entry,

where ⊥ means "absent".

**Axiom A1 (modularity).** Ops_m is fixed when m is authored and is independent of which other manifests are enabled.
Mods are written by independent authors who cannot know each other's file contents. This is the axiom that rules out
"one patch file that knows everything".

**Definition 4 (parametric manifest).** A family m(θ) indexed by a finite set of options Θ_m (for example: language
slot, turf variant, dll name `dxgi` vs `d3d11`, Switcher color). Each instance is an ordinary manifest, so parameters
add **no** primitive.

### 4.5 Build

**Definition 5.** Given an immutable base Σ₀, an enabled set E, and a strict total priority order ≺ on E
(user-controlled; the default is enable order), Build(Σ₀, E, ≺) is defined by the following steps.

1. **Check.**
   - Every guard in ⋃ G_m holds on Σ₀.
   - Every Req is satisfied within E.
   - Every m ∈ E has eff_m defined (no opaque effects).
   - No refusal condition of §5.5 holds.

   If any check fails, the build is **refused**.
2. **Files.** For each p ∈ 𝒫_mgd, take the providers of p in E. Then F(p) is the value of the ≺-maximal provider
   (⊥ means absent). If there is no provider, F(p) = F₀(p).
3. **Documents.**
   - Start from Δ₀(D).
   - Remove the keys that are DEL'd by any m ∈ E. A DEL by m on a key that some m' ≻ m ENTRYs is a value conflict,
     resolved like any other (§5.5).
   - Add the entries of all manifests.
   - Render the order using §4.6.
4. **User paths.** 𝒫_user is never written, except for one-shot seeding (R5).

### 4.6 Rendering order (linearization)

For σ_D = Ord, build a graph whose vertices are the entries and whose edges are:

- the hard constraints C from all ENTRY ops;
- the base chain: base entries keep their relative order.

⊤ means "before every entry not itself ⊤".

Output a topological order by Kahn's algorithm with a priority-queue tie-break:

1. Entries whose side matches side_D go first. The default is `front` for `cpk.root` (mods should override the patch,
   matching "at the very top" in rows 5 and 17) and `back` for `lua.module` (after the patch's libraries, matching
   "CommonLib must be first").
2. Then higher ≺ goes first.
3. Then the key, lexicographically, as a final deterministic tie-break.

For σ_D = First, emit rows in descending ≺. For σ_D = Last, emit rows in ascending ≺, so the maximal provider is
emitted last. For σ_D = Multi, the order is irrelevant; emit base rows, then manifests in ascending ≺.

---

## 5. Theorems

### 5.1 Sufficiency

**Theorem 1.** Every documented install in the corpus (rows 1–28), except the opaque effects of rows 12 and 27, is
expressible as a (parametric) manifest over Π plus guards and capabilities.

*Proof (constructive).* §7 exhibits a manifest for each row. Each documented action falls into one of these classes:

- "copy folder X to Y" is a finite set of PUTs (one per file);
- "rename A to B" is a PUT at B (and a DEL of A, if A was a base path);
- "replace base file" is a PUT on a p ∈ 𝒫_base;
- "delete" or "comment out" is a DEL;
- "add a sider.ini line at the top / below X" is an ENTRY on `cpk.root` or `lua.module` with a constraint;
- "add a map line" is an ENTRY on a Multi or Last document;
- "register with DpFileList Generator" is an ENTRY on the Ord document `download/*FileList.bin` (binary codec);
- "requires X / Sider ≥ v / EXE 2.2 only" is a guard or Req;
- "cannot be used with X" is Conf or a capability;
- "choose turf / slot / dll name" is a parameter.

These classes exhaust the verbs that occur in §3. ∎

Rows 12 (`com269` installer) and 27 (Carabao payload) are covered by Lemma 1 below or refused by R4.

**Lemma 1 (capture of opaque installers).** Let I be an installer that is a deterministic function of Σ₀ and touches
only 𝒫_mgd. Then diff(Σ₀, I(Σ₀)) is a finite set of PUT and DEL ops, and the manifest m_I with that op set satisfies
Build(Σ₀, {m_I}) ≈ I(Σ₀).

*Proof.* Both trees are finite. Each path whose content changed or that appeared becomes a PUT, and each path that
disappeared becomes a DEL. Applying these ops to Σ₀ reproduces I(Σ₀) path by path. ∎

If I reads files written by other mods, its diff depends on E. That violates A1, so capture must be redone per build,
or the installer is refused (R4).

*Plain English:* a dozen different readme phrasings reduce to three verbs. Even `.exe` installers can be turned into
those verbs by recording what they change in a sandbox, provided they behave the same every time.

### 5.2 Minimality

**Theorem 2.** No proper subset of Π suffices for the corpus.

**Lemma 2.1 (PUT is necessary).** Suppose Π \ {PUT} suffices.

- DEL only shrinks dom F.
- ENTRY changes only Δ, that is, files in {path_D}, and only with renderable entries.

Hence for every reachable Σ, dom F ⊆ dom F₀. Row 1 needs `livecpk/Faces/Asset/model/character/face/real/<id>/…` for a
new player id, which is not in dom F₀, and whose bytes (`.fpk` / `.dds` models) are not the rendering of any entry.
Contradiction. ∎

**Lemma 2.2 (DEL is necessary).** Suppose Π \ {DEL} suffices.

- PUT and ENTRY are **address-monotone**: addr(Σ) ⊆ addr(op(Σ)). PUT only adds or overwrites; ENTRY only adds or
  is idempotent.
- So addr(Σ₀) ⊆ addr(Σ) for every reachable Σ.

Row 18 on FL26 requires `(lua.module, "nets.lua") ∉ addr(Σ)`: "cannot be used with ... nets.lua" [V], while FL26's
base loads `nets.lua` [L]. Rows 8, 20 and 26 give further witnesses: base StadiumServer removed, base `sider.dll`
deleted, base lua lines replaced. Contradiction.

Could PUT emulate DEL? Take PUT(p, ε) with ε the empty string. By F2, Sider stops at the first root that has the
name, so V(q) = ε ≠ V₀(q). The empty file shadows instead of revealing. Likewise, a zero-byte `dxgi.dll` is still
loaded by the OS loader and fails, whereas an absent `dxgi.dll` falls through to the system DLL [U for the exact
failure mode]. So PUT(p, ε) ≉ DEL(p). ∎

**Lemma 2.3 (ENTRY is necessary).** Suppose Π \ {ENTRY} = {PUT, DEL} suffices under A1. Then sider.ini is an ordinary
file, and a mod can change it only by PUT(`sider.ini`, b_m) with fixed bytes b_m.

- Take m = Banner Server (row 9) and m' = any later mod with a new module name, for example Referee Kit Server
  (row 6).
- For E = {m, m'}, the build must contain both `lua.module="BannerServer.lua"` and `lua.module="RefKitServer.lua"`.
  The only way is that one of b_m and b_{m'} contains the other's line.
- By A1, b_m is fixed when m is authored. The space of module names is unbounded, and b_m is finite. So there is a
  name n that b_m does not contain, and a mod m'' with module n may be authored later.
- Then E = {m, m''} cannot be satisfied, although such pairs are installed together routinely.

Contradiction. ∎

Corollary: without sub-file granularity, at most one Sider mod can be enabled at a time.

**Lemma 2.4 (order constraints are necessary).** Suppose that for σ = Ord, the manager places entries by a rule φ that
ignores constraints. Two natural rules fail on the corpus:

- **Append** (install order): row 24 needs `TurfLoader.lua` *before* `StadiumServer.lua`, which is already present.
  Appending puts it after.
- **Lexicographic**: row 16 needs `lib\nesalib.lua` before `BroadCastCam.lua`, but 'B' (0x42) < 'l' (0x6C). F3
  shows the failure: "nesalib not found".

In general, for any constraint-free deterministic φ and any two fresh names x, y with φ placing x before y, a mod that
requires y ◁ x is admissible (authors choose names freely). So no such φ is correct for all mods. ∎

**Lemma 2.5 (per-document σ is necessary).** Suppose a single rendering rule r is used for all documents, and
resolution must make the ≺-maximal provider win.

- If r emits the higher priority first, then for the kitserver `map.txt` (Last, F5) the *lower* provider wins.
- If r emits the higher priority last, then for `cpk.root` (First-wins, F1) the lower provider wins.
- If the documents were Multi, no single winner exists at all: the stadium is random (F5).

So r must depend on D. ∎

*Plain English:* you need to create files, remove things, and add individual lines or rows. Lose any one and some
real mod can no longer be installed alongside others. Line placement must be by explicit "before/after" rules. Each
config file needs its own merge semantics because Sider's files genuinely disagree: cpk.root is first-wins, the kit
map is last-wins, the stadium map is "pick at random".

### 5.3 Incremental patching is not reversible

Let inst_m : Σ ↦ Σ' apply Ops_m in place (what readmes describe and users do). An **uninstall function** for m is a
map u_m from states to states with u_m(inst_m(Σ)) = Σ and u_m(inst_{m'}(inst_m(Σ))) ≈ inst_{m'}(Σ) for all m'.

**Theorem 3 (no history ⇒ no uninstall).** There are corpus mods m, m' for which no u_m exists.

*Proof (by contradiction).* Let m be the Alternative Gameplay Repack (row 21), which PUTs `Data/dt18_all.cpk` = b,
plus other files. Let m' be any admissible mod whose only effect is PUT(`Data/dt18_all.cpk`, b'). Examples: the
Switcher's gameplay swap restricted to the cpk, or the single-file pattern of row 23 variant B, which applies to
`SMK_Addmusic.cpk` in the same way.

- Assume u_m exists.
- Since inst_m overwrites the path, inst_m(Σ₀) = inst_m(inst_{m'}(Σ₀)) =: Σ*.
- The first uninstall law forces u_m(Σ*) = Σ₀, where `dt18_all.cpk` = b₀.
- The same law applied at Σ₁ = inst_{m'}(Σ₀) forces u_m(Σ*) = Σ₁, where `dt18_all.cpk` = b'.
- b₀ ≠ b' (the Switcher's own backup `dt18_all_bk.cpk` differs in size from the live file [L]), so a function would
  map one input to two outputs.

Contradiction. ∎

*Plain English:* overwriting destroys information. After the overwrite, the state no longer records what was there
before, so "uninstall" cannot know what to put back.

**Theorem 4 (backups help only under LIFO).** Let inst^b save the previous value of every address it overwrites, and
let uninstall restore it. Then for overlapping mods, uninstall is correct iff mods are removed in reverse install order.

*Proof.*

- (⇐) Induction on a stack: popping m restores exactly the values that were current before m, which are the values
  of the state without m.
- (⇒) Counterexample. Install m (saves b₀, writes b), then m' (saves b, writes b'), then uninstall m out of order
  (restores b₀). Now `dt18_all.cpk` = b₀, but m' is still enabled and requires b'. A later uninstall of m' would
  restore b, the value of a mod that is no longer installed.

The same failure holds at the entry level. Suppose m and m' both DEL the base line `cpk.root=".\livecpk\CompMenus"`
(a fix suggested in the Essentials comments). Uninstalling m re-adds the line while m' still needs it gone.
Reference counting fixes this case only because a reference count *is* a partial record of which manifests are
enabled. Taken to its limit, that record is E itself, which leads to Theorem 5. ∎

### 5.4 Regeneration is reversible

**Theorem 5 (path independence).** Let the manager keep (Σ₀, E, ≺) as its only state, and after every action set
Σ := Build(Σ₀, E, ≺). Then:

1. For any sequence of enable, disable and reorder actions ending in (E, ≺), the tree equals Build(Σ₀, E, ≺)
   on 𝒫_base ∪ 𝒫_mgd.
2. Disable(m) ∘ Enable(m) = id whenever m ∉ E.
3. 𝒫_user is never changed by Build.

*Proof.*

1. Build is a pure function of its arguments (Definitions 5 and 4.6; Kahn's algorithm with a total tie-break is
   deterministic). Each action changes only (E, ≺), so the final tree depends only on the final pair.
2. (E ∪ {m}) \ {m} = E when m ∉ E. Restoring ≺ to E is the restriction of a total order, so it is well-defined.
   Apply part 1.
3. By Definition 5, step 4. ∎

Conditions under which Theorem 5 holds:

- **Σ₀ is immutable and stored** (content-addressed snapshot of 𝒫_base ∪ registered docs at adoption time). A game or
  patch update changes Σ₀. That is a **rebase**: re-check the guards, then rebuild.
- **Every manifest has a defined eff** (no un-captured opaque installer).
- **The user's hand edits to registered documents are imported.** Either import them into Σ₀ (adoption) or wrap them
  in a "local" manifest. Otherwise the next build overwrites them. The FL26 tree already contains hand edits, for
  example `sider.ini.bak-narrador` and the Mariano Closs root [L].

Implementation note (not part of the proof): Build need not rewrite everything. Compute target ⟦Build⟧, diff it
against the current managed tree, and apply the minimal PUT/DEL set. This is the same correctness argument with
fewer writes. It is the GNU Stow / Nix generation pattern.

### 5.5 Conflicts, resolution, refusal

**Definition 6 (conflicts between m, m' ∈ E).**

- **Value conflict.** There is an a ∈ dom eff_m ∩ dom eff_{m'} with eff_m(a) ≠ eff_{m'}(a), where a is a file path
  or (D, k) with σ_D ∈ {First, Last}. PUT vs PUT, PUT vs DEL, and ENTRY vs DEL on the same key all count.
- **Shadow conflict.** There is a virtual path q and distinct roots r ∈ roots(m), r' ∈ roots(m') with r/q and r'/q
  both in dom F. Examples:
  - row 13's `mex/string/all.str` vs a menu mod's `all.str`;
  - row 17's menus vs the Switcher theme.

  A shadow conflict is invisible at the file level because the paths differ; it shows only in V.
- **Multiplicity warning (not a conflict).** σ_D = Multi and both add rows with the same key. The result is defined
  (random choice, F5) but probably unintended. Row 7 on FL26: base already maps 173 → Etihad. An exact duplicate row
  is skipped by deduplication [L]; a different folder name would make the stadium random. An `exclusive` flag on the
  ENTRY means DEL(D, k) followed by ENTRY.
- **Order conflict.** The union of all constraints C has a cycle.
- **Capability conflict.** Some exclusive token is in Prov_m ∩ Prov_{m'}, or m' ∈ Conf_m.

**Theorem 6 (deterministic resolution).** If E passes the refusal checks below, then Build resolves every value and
shadow conflict in favour of the ≺-maximal provider, and the result is unique.

*Proof.*

- Files: step 2 picks the ≺-max provider.
- First/Last documents: §4.6 emits the ≺-max row in the winning position (Lemma 2.5).
- Shadow conflicts: ≺ induces the root order through the tie-break of §4.6, so the ≺-max root is the earliest
  `cpk.root` among those providing q. By F1, it wins.
- Uniqueness: every step is a function of (Σ₀, E, ≺).

Caveat: an explicit constraint can override ≺ in the root order. Such a constraint is author intent, and the manager
should report it. ∎

**Theorem 7 (priority is irrelevant when there is no conflict).** If E has no value, shadow or capability conflicts,
then Obs(Build(Σ₀, E, ≺)) is the same for every ≺.

*Proof.*

- Every address has at most one non-base provider, so its value is independent of ≺.
- Every virtual path is provided by at most one mod root, so V is independent of the root order.
- Multi documents are set or bag unions, which are commutative.
- The lua order may vary only between modules with no constraint between them. By the capability declarations,
  those modules do not compete for the same events.

That last point is an *assumption* about declared capabilities, which motivates Theorem 8. ∎

**Theorem 8 (address-level detection is incomplete).** There are m, m' with no value or shadow conflict that still
cannot coexist.

*Witnesses.*

- Row 18 vs FL26's `nets.lua`: disjoint files and disjoint entries, yet the author says "cannot be used with" it [V].
  Both patch the same goal-net behaviour.
- Two camera modules (row 16 and any other BroadCastCam provider) write the same game memory offsets (`game_info`
  bases `bzoom_pitch` / `bheight_angle` [L]), from disjoint files.

Hence capabilities (or explicit Conf) are necessary. ∎

**Refusal rules.** Priority chooses among values competing for one address. It cannot help when no assignment of
values to addresses yields a valid state. Each rule below is such a case.

- **R1. Guard failure** (row 27: EXE 2.2 only, PES 2021 1.7.2.0 only; Essentials: "no longer compatible with V2.0
  and V2.2"). The guard concerns Σ₀ (the exe, the patch version), which Build never modifies. No choice of E or ≺
  satisfies it. Refuse.
- **R2. Order cycle.** A topological order exists iff the constraint graph is acyclic (a standard result). With a
  cycle, no rendering satisfies all constraints. Refuse and show the cycle. Treating ⊤ as "before all non-⊤ entries"
  (and ordering ⊤ entries by ≺) avoids spurious cycles when two mods each say "very top" (rows 5, 17).
- **R3. Capability clash, declared conflict, or missing requirement.** The constraint is semantic, not an address
  value, so there is nothing to pick. Refuse, and offer to disable one side. Examples: row 18 vs `nets.lua`; row 8 vs
  Stadium Server; row 7 without Banner Server ≥ 3.0.
- **R4. Opaque effect.** If eff_m is unknown, conflicts cannot be computed and Theorem 5 does not apply. Refuse until
  the installer is captured per Lemma 1. Examples: row 12's `com269_xxx.exe`; row 27's undocumented payload; the
  ReShade *installer*, though a shipped `dxgi.dll` + `.ini` set is fine.
- **R5. Writes into 𝒫_user.** Rows 25 and 26 write `EDIT00000000`, which the game rewrites on every save.
  - Suppose the manager did own this path. Then disabling the mod must restore the pre-mod file, which destroys every
    save made since. Keeping the current file instead means disabling is not an inverse. Either way, part 2 of
    Theorem 5 fails. So the operation is **irreversible by nature**.
  - Allow only an explicit, confirmed **seed**: back up the file, write it once, and record it as not undoable.
  - Escape hatch: Sider ≥ 7.4.0 `save.folder` (F10) turns "replace the save file" into "select a save folder", which
    is a scalar ENTRY and therefore reversible. That needs a Sider ≥ 7.4.0 on the target, and FL27 likely ships
    7.3.3 [U].
- **R6. Meta-level ops.** Some ops change the interpreter rather than the state it reads:
  - relocating the Sider folder or deleting `sider.dll` (row 20, Soccer Revolution);
  - setting `livecpk.enabled = 0`;
  - changing `exe.name`;
  - "replaces the existing lua lines" wholesale (row 26).

  These change Obs for *every* manifest, so the per-address reasoning of Theorems 6–7 no longer applies. Allow them
  only from a privileged "platform" manifest, at most one per profile. Otherwise refuse.
- **R7. Cross-ownership writes.** Row 18's optional `goalnets_preset.txt` goes *inside another mod's stadium folder*.
  Allow it only as a PUT on a path that the owning manifest declares as an extension point. Otherwise it is a value
  conflict on a foreign path, and uninstalling the stadium silently drops it.

---

## 6. Manifest schema (TOML)

Two layers: **target profiles + document registry** (shipped with the manager), and **mod manifests** (one per mod).

### 6.1 Target profile and document registry (manager-side)

```toml
# profiles/fl26.toml
[profile]
id = "fl26"
version_file = "dataSP/version.txt"        # guard source [L]
exe = "FL_2026.exe"

[roots]                                     # logical root -> relative path (F8)
game     = "."
sider    = "SiderAddons"
livecpk  = "SiderAddons/livecpk"
modules  = "SiderAddons/modules"
content  = "SiderAddons/content"
download = "download"
data     = "Data"
save     = "{DOCUMENTS}/KONAMI/eFootball PES 2021 SEASON UPDATE/2026/save"   # per pes-files row 25; FL26 exact path [U]
# stadiums / kits are bound by the provider of the matching capability (see 6.2 `binds`)

[user_paths]                                # P_user: never regenerated (F7)
globs = ["SiderAddons/*.log", "SiderAddons/*.cache", "SiderAddons/modules/*.ini",
         "SiderAddons/content/*/config.ini", "SiderAddons/narrador-events.jsonl", "{save}/**"]

[doc."sider.cpk_root"]
file = "@sider/sider.ini"
codec = "ini-multikey"
key = "cpk.root"
semantics = "ordered"          # Ord, first-wins (F1)
default_side = "front"

[doc."sider.lua_module"]
file = "@sider/sider.ini"
codec = "ini-multikey"
key = "lua.module"
semantics = "ordered"          # Ord (F3)
default_side = "back"

[doc."sider.scalar"]
file = "@sider/sider.ini"
codec = "ini-scalar"
semantics = "last"             # duplicate scalar keys: winner [U]
meta_keys = ["exe.name", "livecpk.enabled", "lua.enabled", "save.folder"]   # R6

[doc."stadiums.map_teams"]
file = "@stadiums/map_teams.txt"
codec = "csv-hash-comment"
key_col = 0
semantics = "multi-dedup"      # F5

[doc."stadiums.map_competitions"]
file = "@stadiums/map_competitions.txt"
codec = "csv-hash-comment"
key_col = 0
semantics = "multi-dedup"

[doc."kits.map"]
file = "@kits/map.txt"
codec = "csv-hash-comment"
key_col = 0
semantics = "last"             # F5

[doc."scoreboards.map_competitions"]
file = "@content/scoreboards/map_competitions.txt"
codec = "csv-hash-comment"
key_col = 0
semantics = "multi"

[doc."cpk.filelist"]
file = "@download/spFileList.bin"
codec = "sp-filelist"
semantics = "ordered"          # codec and order meaning [U]
```

### 6.2 Mod manifest (one file per mod)

```toml
schema = 1

[mod]
id       = "etihad-stadium-v2"            # unique id
name     = "Etihad Stadium Expansion V2"
version  = "2.0.0"
source   = "https://www.pesmodding.com/2026/07/pes-2021fl-2026-etihad-stadium.html"
kind     = "stadium"                      # informational
supersedes = ["etihad-stadium-v1"]        # upgrade: auto-disable, not a conflict ("delete previous version")

[guards]                                  # R1, checked on the immutable base
profiles     = ["fl26", "pes2021"]
base_version = ">=2.2.0, <2.3.0"          # optional, read from profile.version_file
sider        = ">=7.4.1"                  # optional
exe_sha256   = []                         # optional allow-list

[requires]
mods = ["banner-server >=3.0", "stadium-server >=1.80"]

[conflicts]
mods = []
capabilities = []                         # exclusive tokens this mod refuses to share

[provides]
capabilities = []                         # e.g. ["goalnets"], ["broadcast-camera"], ["stadium-server"]
binds = {}                                # only server modules: e.g. { stadiums = "@content/stadium-server" }
extension_points = ["@stadiums/Etihad Stadium/goalnets_preset.txt"]   # R7

[params.turf]                             # parametric manifest (Definition 4)
choices = ["MA_GFX_v1", "MA_GFX_v2"]
default = "MA_GFX_v1"

[params.aerial]
choices = ["off", "on"]
default = "off"

[[put]]                                   # PUT (later items overlay earlier; flattened per W1)
from = "Etihad Stadium/"
to   = "@stadiums/Etihad Stadium/"

[[put]]
from = "AddOn/{turf}/"
to   = "@stadiums/Etihad Stadium/"

[[put]]
when = "aerial == 'on'"
from = "AddOn/Cutscene_off/"
to   = "@stadiums/Etihad Stadium/Cutscene/"   # the readme's "rename" is a PUT at the new name

[[entry]]                                 # ENTRY
doc = "stadiums.map_teams"
row = ["173", "009", "Etihad Stadium", "Etihad Stadium"]
comment = "Manchester City"
exclusive = false                         # true => DEL(doc, "173") first

[[entry]]
doc = "stadiums.map_competitions"
row = ["0", "009", "Etihad Stadium", "Etihad Stadium", "", "", ""]
optional = true                           # user-toggleable sub-feature

# [[delete]]                              # DEL examples (other mods)
# path = "@game/dxgi.dll"
# doc = "sider.lua_module"
# key = "nets.lua"

# [[seed]]                                # R5: one-shot, confirmed, recorded as irreversible
# from = "EDIT00000000"
# to = "@save/EDIT00000000"

# [opaque]                                # R4: refused unless `captured` exists
# installer = "com269_xxx.exe"
# captured = "capture/com269-fl26-2.2.0.toml"   # PUT/DEL list from a sandboxed snapshot diff (Lemma 1)
```

Order constraints in `[[entry]]` use `before = [...]`, `after = [...]` (entries named as `"<doc>:<key>"` or bare keys
of the same doc) and `top = true` (⊤). Example for row 24:

```toml
[[entry]]
doc = "sider.lua_module"
value = "TurfLoader.lua"
after = ["WeatherConditions.lua"]
before = ["StadiumServer.lua"]
```

Field ↔ formal object: `put` = PUT, `delete` = DEL, `entry` = ENTRY(D, e, C), `guards` = G_m, `requires` = Req_m,
`conflicts` = Conf_m, `provides` = Prov_m, `seed` = U_m, `params` = Θ_m, `opaque` = a Lemma 1 capture or R4.

## 7. Corpus → manifest sketches

Notation:

- `P a→b` = put; `D x` = delete; `E doc:value [constraints]` = entry.
- `G` = guards; `R` = requires; `C` = conflicts / capabilities.
- `cr` = `sider.cpk_root`; `lm` = `sider.lua_module`.

| # | Manifest sketch | Notes |
|---|---|---|
| 1 | `P <id>/ → @livecpk/Faces/Asset/model/character/face/real/<id>/`; `E cr:".\livecpk\Faces"` | Shared root: the ENTRY is idempotent across facepacks; two packs with the same `<id>` give a value conflict, resolved by ≺ |
| 2 | `P X.cpk → @download/X.cpk`; `E cpk.filelist:X.cpk` | The FL list codec is [U]; fall back to variant 1 (livecpk) |
| 3 | base content, not a mod | Patch update = rebase Σ₀ (§5.4) |
| 4 | `P Premier League/ → @kits/Premier League/`; `E kits.map:(100,"Premier League\Manchester United")`… | R `kitserver`; team ids as a param table per profile ("IDs vary") |
| 5 | `P fonts/ → @livecpk/kitserver special character fonts/`; `E cr:… top`; `P UniNameFontPermissions.bin → @…` | A whole-binary table is a value conflict with any other font pack (no row-level codec) [U] |
| 6 | `P content/ modules/ livecpk/`; `E cr:".\livecpk\RefColor"`; `E lm:"RefKitServer.lua" after CommonLib` | R CommonLib, G sider ≥ 6.5 |
| 7 | the TOML in §6.2 | Multiplicity check vs base row 173 |
| 8 | `P dt30_g4.cpk → @data/dt30_g4.cpk`; `P → @livecpk/Stadiums Extra DB/`; `E cr:…`; `D lm:<stadium server>`; `D @stadiums/` | C capability `stadium-server`. The DEL of a base module is R6-adjacent: keep it as an explicit, reviewed DEL |
| 9 | `P content/ modules/ livecpk/`; `E lm:"BannerServer.lua" before StadiumServer.lua`; `E cr:".\livecpk\Banner"` | P capability `banner-server` |
| 10 | `P → @livecpk/World Cup 2026 Font/`; `E cr:…`; `E scoreboards.map_competitions:<row from archive>` | R `exclusive-scoreboard >=15`; the row is [U] until the archive is read |
| 11 | `P common/ → @content/SubBoard/Spain/La Liga/common/` | R `subboard-server` (FL26 base) |
| 12 | `[opaque] installer = "com269_xxx.exe"` | R4 until captured; expected writes `@data/dt41_<slot>_all.cpk` [U] |
| 13 | `P ags/ mex/ spa/ → @livecpk/Mariano Closs V5/{slot}/…`; `E cr:".\livecpk\Mariano Closs V5"` | Param `slot` ∈ {eng, spa, ags, …} = the "move `<lang>/sound`" trick; shadow conflict on `*/string/all.str` with menus |
| 14 | `P preds-root/ → @livecpk/preds-root/`; `E cr:".\livecpk\preds-root"` | "Delete old callname files" becomes `supersedes` / C on other callname mods |
| 15 | `P custom-comment-server.lua → @modules/`; `E lm:…`; `seed map.txt → @content/custom-commentary-server/map.txt` | The map is user-authored, so seed, not managed |
| 16 | `P *.lua → @modules/`; `seed *.ini → @modules/` (F7: rewritten by the overlay); `E lm:"lib\nesalib.lua"`; `E lm:"BroadCastCam.lua" after lib\nesalib.lua` … | P capability `broadcast-camera` (exclusive); G sider ≥ 7.3.4 |
| 17 | `P content/ livecpk/ modules/`; `E cr:".\livecpk\MenuC1987" top`; `E cr:".\livecpk\UIColors" top`; `E lm:"UIColors.lua" after lib\CommonLib.lua` | Shadow conflicts with the Switcher theme and row 13 strings, resolved by ≺ |
| 18 | `P livecpk/goalnets_25-26/`; `P goalnets/ → @modules/goalnets/`; `E cr:…`; `E lm:"goalnets\goalnets_25-26.lua"`; `D lm:"nets.lua"` | P capability `goalnets` (exclusive); C `nets`, `nets-server` |
| 19 | `D @content/ball-server/`; `P ball-server/ → @content/ball-server/` | Folder replacement = DEL + PUT; steps for 2026-27 are [U] |
| 20 | `P dxgi.dll → @game/{dll}.dll` (param dll ∈ {dxgi, d3d11}); `P reshade-shaders/`; `P *.ini → @game/`; `seed ReShade.ini` | Toggle = enable/disable (presence semantics). The hook DLL + Sider relocation part is R6 (platform-only) |
| 21 | `P dt18_all.cpk → @data/dt18_all.cpk` | Value conflict with the Switcher gameplay variant (row 22): the Theorem 3 witness |
| 22 | one parametric manifest `fl-switcher`: `P variants/{theme}/SMK_Addtheme.cpk → @download/SMK_Addtheme.cpk`, same for LUT / control / lockdown / replay / gameplay | Replaces the Switcher's own swap-and-backup with Build; `dataSP/*.txt` stay Switcher-owned [U whether the game reads them] |
| 23 | Variant A: `P → @livecpk/Soundtrack eFootball 2026/`; `E cr:…`. Variant B: `P eFutBol26 BGM.cpk → @download/SMK_Addmusic.cpk` | Two variants of one parametric manifest; B conflicts with any other `SMK_Addmusic.cpk` provider |
| 24 | `P …`; `E lm:"TurfLoader.lua" after WeatherConditions.lua before StadiumServer.lua`; `E lm:"Shape-turf-loader.lua" after TurfLoader.lua before StadiumServer.lua` | The Lemma 2.4 witness |
| 25 | `seed EDIT00000000 → @save/EDIT00000000` | R5: confirm, back up, irreversible; better: `E sider.scalar:save.folder` if Sider ≥ 7.4.0 |
| 26 | `P livecpk/ modules/ content/`; `E cr:…`; `D lm:*` + `E lm:"SoundtrackServer.lua"`…; `seed EDIT00000000` | Wholesale lua replacement is R6, so it must be a platform manifest; R5 for the edit file |
| 27 | `G exe_sha256 = [<FL_2026.exe 2.2 Mar-2026>]` (FL26) or `[<PES2021 1.7.2.0>]`; `[opaque]` until the payload is inspected | R1 on FL27 (the author says it is unsupported) |
| 28 | `P narrador.lua → @modules/`; `E lm:"narrador.lua"`; user path `narrador-events.jsonl` | Rollback = disable (Theorem 5), replacing the manual `sider.ini.bak-narrador` |

## 8. Open items (to verify before implementation)

1. The codec of the FL `download/spFileList.bin`, and whether a later or an earlier entry wins among packed cpks. [U]
2. How the FL26 and FL27 Sider handles duplicate scalar keys in `sider.ini`, for example the duplicated
   `match-stats.enabled` in FL26's `sider.ini` [L]. [U]
3. What `com269_xxx.exe` writes: capture it in a sandbox (Lemma 1). [U]
4. The bundled Sider version on FL26 and FL27, and whether `save.folder` (7.4.0+) can be used. The FL27 date points
   to 7.3.3. [U]
5. Whether FL's Lua-based Switcher features (`SPcolor.lua`) use the memory color hack that Sider 7.4.0 says "no longer
   works". If they do, upgrading Sider on FL is itself guarded. [U]
6. Which stadium root a given FL26 build actually reads (`content/stadiums` per [L], vs `content/stadium-server` when
   the Essentials StadiumServer.lua replaces the base module). Model it as a capability-bound root (`binds`). [L/U]
7. Install steps for row 19 (the archive readme) and the "registration" text in row 10. [U]
