# Grilling decisions (2026-11-02)

| # | Decision | Answer |
|---|---|---|
| Q1 | First published audience | **Linux/Proton only** (Deck, Ally, Bazzite, desktop Linux). Windows later; core stays OS-agnostic. |
| Q2 | Games supported in MVP | **FL27 only.** Others later as target config files (`targets/*.toml`), no code changes. |
| Q3 | First public release includes GUI? | **Yes.** Publish only with the egui GUI (toggles, plan, apply, undo, game settings). CLI is the engine + dev/test tool, not polished for users. |
| Q4 | How a new mod enters the tool | **Archive in → extract to staging → auto-detect root (descend single wrapper dirs, ignore readme/images, match FL layout case-insensitively) → preview with conflicts → manual root pick if detection fails.** One root per mod. zip + 7z via pure-Rust crates; rar via system 7z/unrar probed at runtime. No FOMOD, BAIN or VFS. Source: research/import-and-archives.md. |
| Q5 | Priority in the mod list | **Freely reorderable list (move anywhere); on conflict the lower mod wins** (MO2/Vortex convention). Each conflict shown explicitly ("X overrides N files of Y"). |
| Q6 | Instances and profiles in MVP | **One install, one mod list** + numbered generation history for undo. Multiple instances / named profiles later if users ask. LAB stays a dev-only CLI target. (Accepted as "sigue".) |
| Q7 | Mod types in MVP | **5 types:** livecpk (copy + cpk.root), Lua module (copy .lua/.ini + lua.module), stadium (copy + ask team → map_teams row), download/*.cpk replacement (keep original for undo), loose files in game root (e.g. ReShade). Opaque .exe installers out of MVP. |
| Q8 | Pre-existing hand-installed mods | **First-run snapshot is the base.** Old mods stay frozen inside it, working but unmanaged; re-import to manage them. "Disable all" returns to day-one state, not factory FL27. Full adoption against a clean reference: later, on demand. |
| Q9 | Game updates / outside changes | **Detect and ask.** Before each apply, compare disk with expected; on drift refuse and list changed files with "Accept as new base" (game update case: base updated, mods re-applied) or "Restore". New exe hash disables exe-guarded mods with a warning. No automatic 3-way rebase. |
| Q10 | Game discovery on Linux | **Hybrid:** quick bounded scan for FL_2027.exe in typical places (~/Games, Steam libraries and compatdata, Lutris/Heroic/umu defaults) → list of candidates with their prefixes + "Choose another folder". **The user always confirms the selection.** Prefix auto-found by `SPGAME/Football Life`, asked if missing. |
| Q11 | Mod storage and deployment | **One folder per mod** next to the game: `fl-modman/mods/<id>/` (files + mod.toml), `fl-modman/originals/` (game files overwritten by mods, for undo), `fl-modman/history/NNNN.toml` (generations). Deploy by hardlink on same filesystem, else copy; never symlinks. No content-addressed store. |
| Q12 | Apply safety | **Simple:** write plan to history/NNNN.toml as "in progress" → apply file by file → mark "complete"; on startup an in-progress generation offers finish or roll back. Hardlink only heavy never-written assets (.cpk, .acb/.awb, .ftex, .dll); text files (.ini/.lua/.txt) always copied. Refuse to apply while the game runs. No full transactional journal. |
| Q13 | Advanced rules kept in MVP | **Only an optional exe guard** (`exe = "<hash>"`). Drop before/after constraints (list order decides), exclusive capabilities (shown as plain file/line conflicts), params/`when` (variants = separate mods), seeds, opaque installers. Cut these from slice 1. |
| Q14 | License and owner | **MIT**, copyright NoRoboto. Published on GitHub under the NoRoboto account. |
| Q15 | Linux distribution format | **AppImage** (single file, works on Bazzite/SteamOS/Fedora/Ubuntu, addable to Steam). Flathub later. |
| Q16 | GUI screens in MVP | **5 screens, gamepad-first:** pick game (first run), my mods (reorderable list, toggles, ⚠ conflicts, Apply with plan summary), add mod (archive → preview → team picker for stadiums), history (generations, "go back"), game settings (Appendix C). Out: profiles, online mod browsing/downloads, themes, mod artwork. |
| Q17 | UI language | **English only** in MVP. |
| Q18 | Ally/Bazzite fixes (tdp-guard, InputPlumber chord, Steam art) | **Out of scope**, stay separate scripts. Only a diagnostic: detect missing Proton overrides and show the exact launch-options text to paste (never edits Steam). |
| Q19 | Code budget and work order | **Explicit budget:** core + CLI ≤ ~2,500 src lines (tests excluded); egui GUI ~1,500. Exceeding needs justification. Order: (1) cut slice 1 per Q13 to budget, tests green; (2) slice 2: import, apply + history, undo, drift detection, game settings via CLI, tested on LAB; (3) slice 3: egui GUI tested on the Ally in Gaming Mode. |

**Grilling closed when the user confirms shared understanding.** These decisions override `docs/design.md` where they differ.
| Q19b | Budget measurement | Standard rustfmt (no `use_small_heuristics` tweak). Budget re-set to **~2,900 production lines** for core + CLI; formatting must never be tuned to meet a line count. |
| Q19c | GUI budget | **Accepted ~1,750 production lines** for fl-gui (trimmed 1,838 → 1,738 without behaviour change). Remaining length is useful handheld features: touch drag-to-reorder, numeric picker +/- buttons, in-memory mod cache for fast conflict checks. |
| Q20 | Performance + architecture | (a) Per-mod immutable file index written at import (`mods/<id>/index.json`); plan/conflicts read indexes, never rehash payloads; explicit `check` verifies. Target: open + plan in milliseconds (was 10.8–13.8 s for 10.4 GB). (b) Then refactor to hexagonal layout with mod types as compile-time plugins (`ModKind` trait: detect / contribute / describe; one file per kind); no runtime dynamic loading. |
| Q20c | Engine budget after refactor | **Accepted ~3,500 production lines** for core+CLI (3,462 after hexagonal + ModKind plugins; +~300 buys add-a-kind-in-one-file and a Windows adapter seam). Still enforced: further growth needs justification. |
| Q19d | GUI budget after design system | **Accepted ~2,000 lines** (user: "se ve mucho mejor"). Design system in `crates/fl-gui/src/ui.rs`. |

## Earlier decisions (2026-11-02, before grilling)

- Standalone desktop app, not a Decky plugin. (Windows support was later deferred by Q1.)
- **Minimal code is a requirement, not a preference.** Build only what the current phase needs (YAGNI). Prefer
  deleting code over adding options. Each slice ends with a simplification pass that keeps the tests green.
