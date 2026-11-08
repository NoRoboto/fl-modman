# State of the art: mod managers for PES 2021 / SP Football Life (Sider 7)

Research date: 2026-11-02. Scope: PES 2021, SmokePatch SP Football Life 2026/2027 (FL26/FL27), Sider 7, Linux / Steam Deck / Bazzite / ROG Ally.
Verification legend: **[V]** read on the primary page/API myself; **[S]** only a search-engine snippet or a secondary page; **[?]** could not be verified.

## TL;DR verdict

- **Partial solutions exist, none matches the declarative + Linux target.** Two or three Windows-only GUI tools now do toggle / reorder / conflict / profile work on Sider setups. All are imperative (they edit `sider.ini` and copy files), none publishes a manifest format, none supports Linux/Deck/Proton, and the two best candidates are closed (Patreon-gated) or tied to one author's modules.
- **Closest to your idea:** "PES Mod Manager" (Patreon + netlify site, beta, Windows 10/11 only, PES 2021 + FL26). It advertises profiles, conflict detection, enable/disable by commenting lines out, a managed block in `sider.ini`, and `cpk.root`/`lua.module` README parsing. No FL27 mention, no Linux mention, no manifests, author anonymous, closed source [V site + Patreon]. Treat as competitor/prior art, not a base.
- **Active in the last 5 weeks** (Sep-Oct 2026): PES Mod Manager v1.2.1 (Patreon), "Sider Mod Manager v1" (pes-files, 2026-09-29), blue41 PES2021 Mod Studio v0.2.0 beta, FL26 Mod Studio 0.2.5.1 (2026-10-30), Sider Mod Menu v1.2 (2026-10-23, in-game). The space is moving fast; assume someone will announce a "full" manager soon.
- **Linux gap is real and open.** Sider itself is Linux/Deck-aware since 7.2.1/7.3.0, and one GitHub repo (Aug 2026) automates FL26 on Linux via Proton, but no tool manages mods there. Every GUI manager found is a Windows .exe (Wine/Proton behaviour untested).
- **Sider's own semantics make a declarative layer easy:** `cpk.root` lines are an ordered list, first match wins (primary doc), `lua.module` lines are an ordered list. Conflicts = path overlap between roots, computable offline. State = a generated block of `sider.ini`.
- **Recommended gap to fill:** a Linux-first, scriptable (CLI/TUI) manager with a manifest per mod, profile files that regenerate `sider.ini` from a clean base, offline conflict report, and Proton-prefix awareness. Borrow: Thunderstore manifest, MO2 priority model, Nexus App loadouts, packwiz-style text files.

## 1. PES-specific tools

| Tool | What it does | State / conflicts? | Last activity (evidence) | Platform |
|---|---|---|---|---|
| **PES Mod Manager** (Patreon `pesmodmanager`, site pesmodmanager.netlify.app) | One-click install from zip/7z/rar/folder, structure detection (content, module, livecpk, multi-root), README parsing for `cpk.root`/`lua.module`, enable/disable, profiles, conflict detection, `CommonLib.lua` always first, "managed by PES Mod Manager" block in `sider.ini` | Yes: profiles + file-competition flags. No manifest mention. Placement mechanism (copy/link) unstated [V] | Patreon post "v1.2.1 (redesigned...)" dated "September 23" (year not shown, locked post); site shows (c) 2026 [V]. "First public release, still beta" [V] | Windows 10/11 only. PES 2021 + FL26. FL27: not mentioned. Free for Patreon supporters |
| **Sider Mod Manager v1** (pes-files, Muhammad Ridhuwan) | Manage LiveCPK + LUA modules: toggle, rename, reorder, delete; copy/move install; missing/duplicate-file detection; `.bak` on save | Duplicate detection, no profiles listed [V] | Posted 2026-09-29 [V] | Platform unstated, paths imply Windows; download via Sharemods/Modsfire (low-trust hosts) |
| **blue41 PES2021 MOD STUDIO** (github.com/LimaDiogenes/blue41_PES2021_MOD_STUDIO) | `sider.ini` option editor with before/after diff; toggle + reorder Lua modules and livecpk folders; backups and undo; plus own gameplay modules, chants, boots/gloves | Backups/undo yes; explicit conflict detection no [V] | Repo updated 2026-10-25; v0.2.0 beta; 14 commits, 0 stars [V] | Windows 10/11, freeware, all rights reserved |
| **FL26 Mod Studio** (github.com/Matozanato/FootballLife-new-leagues, MIT) | Builds new leagues via Sider modules + `.fl26pack` packages. README claims: toggle/reorder content & modules, profiles, restore point per changed file, import modpack and report where parts go, uninstall what it installed, health check | Restore points + health check; conflicts not stated [V from README via fetch] | v0.2.5 2026-10-28, 0.2.5.1 2026-10-30 [V] | Windows (`FL26ModStudio.exe`) |
| **Sider Mod Menu v1.2** (TwanVRI, pesmodding 2026-10-23) | In-game mouse menu (F1) listing Sider modules; installer adds `lua.module = "modmenu.lua"` first and backs up `sider.ini`. Not a package manager | No [V] | 2026-10-23; says built with Claude Code [V] | Windows installer (.bat/PowerShell); Linux not mentioned |
| **PES 2021 Mod Manager v1.3.0** (StpN_17, pes-files) | Pick CPKs, sort, generate `DpFileList.bin` (backs up old as `.BACKUP`). DLC 7.0 era | Ordering only [V] | 2023-05-13 [V] | Windows, .NET 4.5, run as admin |
| **DpFileList Generator** (pesmodding, DLC 4.0 / 7.0 posts) | Generates `DpFileList.bin` for CPK mods | None | DLC 7.0 post 2021-06 [S] | Windows, .NET 4.5 |
| **PES CPK Tool v1.0.0** (StpN_17, pesmodding 2021-11) | Extract/build CPK for PES 2017-2021 | None | 2021-11 [S] | Windows |
| **PES 2021 Stadium Manager / Stadium Checker** (pesmodding tools label, 2025-10-16 / 2025-12-09) | Edits/validates `map_teams.txt` for the stadium server, validation + backups | Per-server validation | Late 2025 [V listing; contents not opened] | Windows (assumed [?]) |
| **PES 2021 Profile Switcher v1.1** (efootball_multiverse, 2025-10-20) | Switches user profiles / DpFile sets with backups | Profiles of *save data*, not mods [S from listing summary] | 2025-10 | Windows (assumed [?]) |
| **Sider 7** (juce, github.com/pes-modding/sider7) | Loader. LiveCPK (`livecpk.enabled`, ordered `cpk.root` lines), Lua modules (`lua.module` ordered), overlay, `start.game`, patcher mode | `cpk.root`: "first listed wins". No state beyond `sider.ini` [V readme.html] | v7.4.1 released 2026-04-10; repo pushed 2026-07-22; 60 stars, 10 open issues [V GitHub API] | Win; Linux, macOS, Steam Deck since 7.3.0 (2024-11-27) [V releases] |
| **Servers / modules** (Kitserver, Stadium/Banner/Ball/Goalnets servers, scoreboards, etc.) | Sider Lua modules reading mapping files (`map_teams.txt` etc.). Each has its own config; no shared registry | None; collisions are manual (pesmodding comments advise "disable mods added after install") [V] | Active; evoweb forum Oct 2026 full of Sider modules [V] | Same as Sider |
| **SmokePatch "Sider SP2x" installers** (pessmokepatch.com/2021/07/sidersp21.html) | Installer that drops a pre-configured Sider folder in the game dir; DLL 7.1.4 listed there | No | Page dated 2021-07 [V] | Windows installer |
| **FL Switcher / `FL27 Switcher.exe`** (SmokePatch) | FL27 post: "additional customization options through the FL Switcher, including theme colors, render lighting, gameplay, controller buttons, classic teams" [V]. Treat as a settings/options switcher for SmokePatch's own content, **not** a third-party mod manager. Its internals, file format and Wine behaviour: **[?] not verified** (I did not have the exe). The FL26 `Settings.exe` is documented to crash under Proton (.NET 4.8) in the Linux repo below [V] | Unknown | FL27 v1.00 posted 2026-10-31 (site lists 1 Nov as scheduled date too) [V] | Windows exe |

Notes:
- A naming caution: pesmodding "ML Manager Mod" is a Master League mod, not a mod manager. Evoweb's "FIFA Mod Manager" mention is for FIFA, not PES [V].
- evoweb.uk: no thread about a PES mod manager among the latest ~30 threads of the PES 2021 forum (fetched 2026-11-02). There is a pinned "List of important tools & tutorials" (updated 2025-07-03) whose body I could not read, so a manager listed there is **[?]**.
- Reddit (r/WEPES, r/SPFootballLife) and Nexus returned HTTP 403 to automated fetch; no Reddit evidence either way **[?]**. Several "Steam Deck + Football Life" SEO pages appeared in search; they are low-quality generated pages and were not used.

## 2. General-purpose managers: what to borrow

| Manager | Status (evidence) | Mechanism | Idea to borrow |
|---|---|---|---|
| **Mod Organizer 2** (github.com/ModOrganizer2/modorganizer) | Repo pushed 2026-10-05; last release v2.5.2 (2024-08-04) [V API] | USVFS: Windows API hooking so selected processes see a virtual overlay; game folder never touched [V usvfs README]. Priority list = who wins | **Priority-ordered overlay, game dir stays clean.** For Sider this maps 1:1 onto ordered `cpk.root` lines. Per-mod folders + a flat priority list is the right mental model |
| **MO2 on Linux** (Furglitch/modorganizer2-linux-installer, GPL-3.0) | Pushed 2026-11-02, 1.9k stars; Python rewrite [V] | Runs MO2 under Proton; registers NXM handler; per-game workarounds; Deck/Decky not mentioned [V] | Proton-prefix awareness + per-game quirks table. Also shows USVFS cannot be used natively on Linux; a Linux manager should use symlinks/overlay or Sider's own roots instead |
| **Linux Mod Organizer** (codeberg.org/DoggoNako/linux-mod-organizer) | Exists per search result; I did not open it [S] | OverlayFS instead of USVFS | Evidence that native overlay is the Linux answer; not directly usable for Proton prefixes |
| **Vortex** (Nexus-Mods/Vortex) | v2.9.1 released 2026-10-27 [V API] | Staging folder + deploy by hardlink/symlink; per-mod conflict rules that pick a winner [S forum threads] | Separate *staging* from *deployment*; "deploy" as an explicit idempotent step; conflict rules as data. Nexus announced SteamOS support (Deck, Steam Machine), source says other distros unsupported [V via Lemmy repost of nexusmods.com/news/15433, date not shown] |
| **Nexus Mods App** (Nexus-Mods/NexusMods.App) | Latest release v0.21.1 2025-11-17, repo pushed 2026-02-16; alpha; Windows + Linux, Deck "works but not yet optimised" [V docs + API] | Loadouts, event-sourced undo, health check, "Wine and Proton detection design" (docs titles) [V titles only] | **Loadout = named, versioned set + undo log.** Health check as first-class command. Proton detection design is directly on topic; worth reading their dev docs |
| **r2modman / Thunderstore** (ebkr/r2modmanPlus) | v3.2.21 2026-10-30 [V API] | Profiles (named mod sets), packages are zips with root `manifest.json` | **Manifest standard**: `name` (a-zA-Z0-9_), `version_number` (semver), `description` (<=250), `dependencies` (list of "Author-Name-Version"), `website_url`; plus `icon.png`, `README.md` [V wiki.thunderstore.io/mods/creating-a-package]. Dependencies + profiles in plain files is exactly the declarative core |
| **BepInEx** (BepInEx/BepInEx) | Pushed 2026-10-23 [V] | Loader with `plugins/`, `patchers/`, `config/` dirs; mod managers just place files by convention | Convention-over-config directory contract per mod type (like Sider's `modules/` + `livecpk/`) |
| **Wabbajack** (Wabbajack-Tools/wabbajack, pushed 2026-10-31) | Active [V API] | A "modlist" is a declarative recipe (source hashes + patches) that rebuilds an install | **Regenerate-from-base**: store the recipe, not the output. Closest to Nix-style in mainstream modding; content-addressed sources |
| **packwiz** (packwiz/packwiz) | Pushed 2026-10-14 [V API] | Git-friendly TOML metadata files describing a pack instead of binaries [V README] | Text-only, diffable, git-versioned profile files; per-mod metadata + hash. Good template for `profile.toml` |
| **Decky-Nexus** (RedRanger14/decky-nexus) | Created 2026-07-16, 113 stars, v1.21.40 on 2026-10-25 [V API]. Needs Nexus Premium; no PES support; load-order editing "not available yet"; built with Claude per its notes [S release notes] | Decky plugin that downloads/installs into game dir, Proton prefix or loader profile | Existence proof that a Decky plugin can drive a mod install inside Gaming Mode. Pattern: CLI/daemon does the work, Decky UI just toggles profiles |
| **Lutris / Heroic** | Active (pushed 2026-11-02) [V API] | Wine prefix management, runner selection, installer scripts. Lutris has a "Football Life 2023" installer by SmokePatch, last published ~3 years ago, no Sider mention [V lutris.net] | Installer-script format for prefix setup; not a mod layer |
| **Nix-like declarative approaches** | No PES-specific or mainstream game-mod example found beyond Wabbajack/packwiz | n/a | Principle only: profile (inputs) -> pure function -> generated `sider.ini` + links; garbage-collect what is not referenced. Say clearly this is design inspiration, not an existing tool |

## 3. Linux / Proton / Steam Deck / Bazzite / ROG Ally

Verified facts:

- **Sider is Linux-aware.** v7.2.1 "Special Edition" (2024-03-28) added a patched-exe attachment mode for Windows, Linux and Steam Deck; v7.3.0 (2024-11-27) unified both launch methods for "Windows, macOS, Linux, and Steam Deck"; v7.4.1 is current (2026-04-10) [V GitHub API + releases page]. Proton/Wine is never named in release notes.
- **Two launch modes** [V `doc/readme.html`, `doc/install-with-patcher.html`]: (a) `sider.exe` launches the game (can use a Steam URL, "one-click launch", then exits); (b) `patcher.exe` modifies `PES2021.exe` once so it loads `sider.dll` itself, then you start via Steam normally. Re-patch needed after a game exe update. Uninstall = run patcher again, or rename the sider dir (`sider.disabled`) to disable temporarily. For Deck/Linux/macOS the doc points at mode (b) [S].
- **Evoweb origin:** juce, 2022-10-28, said 7.2.0 "easier to use on Steam Deck" but also "I don't actually have a Steam Deck", so no step-by-step exists from the author [V thread 92469].
- **SPFL26 on Linux, end to end:** github.com/agentkyo/spfl26-pes2021-linux (created 2026-08-08, last push 2026-08-10, 6 stars, 2 issues from 2026-10-22). `install.sh` with GE-Proton11-3, no sudo, claims Bazzite/SteamOS/Deck compatibility but **tested on Zorin/Ubuntu only** (tested = Ryzen 9 3900X + RTX 4070 Ti). Facts worth reusing: all three installers (base, 2.0, 2.2) must target the same folder; never use `Z:\` paths ("Sider gets weird"); `Settings.exe` (.NET 4.8) crashes under Proton's Wine Mono; installers are GUI-only; game reports version via `dataSP/version.txt` [V README]. It does not manage mods after install.
- **Headless Proton launch:** github.com/eloymelo/pes2021-sider-headless: Debian + Proton Experimental, Steam launch option wrapper that starts `sider.exe` with `start.game = "PES2021.exe"`. No Deck claims; commit dates not visible [V].
- **ProtonDB** for PES 2021 (AppID 1259970): tier gold, 46 reports, best reported platinum, confidence strong [V ProtonDB API 2026-11-02]. SPFL26 is a separate standalone installer, not the Steam title, so the rating is only indirect.
- **Handheld mod tooling gap:** Decky Loader plugins for mods exist for Nexus-hosted games only (Decky-Nexus). A Windows `.exe` manager can in principle be added as a non-Steam game and run in the same prefix (the pattern used for Fallout NV mod managers [S]), but I found no one doing it for Sider [?].
- **ROG Ally:** default OS is Windows, where the Windows managers above run natively. Bazzite on the Ally (the author's setup) is a different case and no source covers it [?].

Design hints for a Linux-first manager (inference, not sourced):
- Sider paths are relative to the Sider folder, which sits in the game directory inside the Proton prefix; Linux symlinks pointing out of the prefix are risky, so prefer *copy or reflink into the Sider tree* or put mod roots *inside* the prefix and reference them with `C:\...` paths.
- Writing `sider.ini` with LF vs CRLF and UTF-8 needs testing under Wine.
- Steam Deck Gaming Mode: no Desktop-mode round trips -> a Decky front end over a CLI is plausible.

## 4. Open questions

1. What exactly is `FL27 Switcher.exe` (file format, does it rewrite `sider.ini`, does it run under Proton)? Only the FL27 post sentence was verified. It should be inspected on the user's own copy.
2. Does PES Mod Manager's closed source copy files or symlink, and does it work under Wine? Unknown, and no Linux mention exists.
3. Has PES Mod Manager's Patreon post "v1.2.1 (September 23)" a 2026 date? Year not shown. Evidence (c) 2026 on the site only.
4. Does the evoweb "List of important tools & tutorials" pinned thread list a manager? Not readable via fetch.
5. Reddit and Nexus content (r/WEPES, r/SPFootballLife, Nexus PES 2021 pages) could not be fetched (HTTP 403); anything there is unchecked.
6. Which Sider module authors, if any, ship machine-readable metadata (versions, dependencies, conflicts)? Spot checks of pesmodding posts found only prose and README steps; "Minimum version of Sider required: 7.0.0+" appears as text in a related mod post [S].
7. Does Sider honor symlinked `cpk.root` directories on Proton? Not documented, needs a test.
8. Update interplay: SmokePatch updates (FL26 2.0 / 2.2, FL27 1.00) overwrite game files; the "Essentials" pack post states it is incompatible with later updates [V]. How a declarative layer should pin to a base version is undefined.

## 5. Sources (all accessed 2026-11-02 unless noted)

Primary / near-primary
- Sider 7 readme: https://raw.githubusercontent.com/pes-modding/sider7/master/doc/readme.html (cpk.root order, livecpk, lua.module, start.game)
- Sider 7 patcher install doc: https://raw.githubusercontent.com/pes-modding/sider7/master/doc/install-with-patcher.html (v7.4.1, 2026-04-10)
- Sider 7 default config: https://raw.githubusercontent.com/pes-modding/sider7/master/sider.ini
- Sider releases (API): https://api.github.com/repos/pes-modding/sider7/releases ; repo https://github.com/pes-modding/sider7 ; org https://github.com/pes-modding
- SmokePatch FL27 release (2026-10-31): https://www.pessmokepatch.com/2026/10/spfl27.html
- SmokePatch FL26: https://www.pessmokepatch.com/2025/10/spfl26.html ; Sider for SmokePatch: https://www.pessmokepatch.com/2021/07/sidersp21.html ; install: https://www.pessmokepatch.com/2022/10/installation.html
- Evoweb Steam Deck thread (2022-10-28): https://evoweb.uk/threads/i-know-that-someone-has-already-run-the-patch-for-pes-21-steam-deck-please-any-help-or-recommendation.92469/
- Evoweb PES 2021 forum index: https://evoweb.uk/forums/pes-2021.337/
- Pesmodding: Sider 7.2.0 SE https://www.pesmodding.com/2023/04/pes-2021-sider-720-special-edition.html ; Sider 7.3.3 https://www.pesmodding.com/2025/08/pes-2021-sider-733.html ; Sider Mod Menu https://www.pesmodding.com/2026/10/sider-mod-menu-v12-modern-mouse.html ; Tools label https://www.pesmodding.com/search/label/PES%202021%20Tools ; Essentials pack https://www.pesmodding.com/2025/11/sp-football-life-2026-v12-essentials.html
- PES Mod Manager: https://pesmodmanager.netlify.app/ ; https://www.patreon.com/cw/pesmodmanager
- Sider Mod Manager v1: https://pes-files.com/pes-2021-sider-mod-manager-update-v1/ ; Mod Manager v1.3.0: https://pes-files.com/efootball-pes-2021-mod-manager-v1-3-0/
- blue41 Mod Studio: https://github.com/LimaDiogenes/blue41_PES2021_MOD_STUDIO ; FL26 Mod Studio: https://github.com/Matozanato/FootballLife-new-leagues
- Linux repos: https://github.com/agentkyo/spfl26-pes2021-linux ; https://github.com/eloymelo/pes2021-sider-headless
- ProtonDB: https://www.protondb.com/app/1259970 (summary via protondb.com/api/v1/reports/summaries/1259970.json)
- Lutris FL23: https://lutris.net/games/football-life-2023/

General managers
- MO2: https://github.com/ModOrganizer2/modorganizer ; USVFS: https://github.com/ModOrganizer2/usvfs ; MO2 Linux installer: https://github.com/Furglitch/modorganizer2-linux-installer
- Vortex: https://github.com/Nexus-Mods/Vortex ; Vortex SteamOS repost: https://peachpie.theatl.social/c/linux_gaming/p/198839/vortex-mod-manager-from-nexus-mods-will-be-supported-for-linux-steamos
- Nexus Mods App: https://github.com/Nexus-Mods/NexusMods.App ; docs https://nexus-mods.github.io/NexusMods.App/users/
- r2modman: https://github.com/ebkr/r2modmanPlus ; Thunderstore package format: https://wiki.thunderstore.io/mods/creating-a-package
- BepInEx: https://github.com/BepInEx/BepInEx ; Wabbajack: https://github.com/Wabbajack-Tools/wabbajack ; packwiz: https://github.com/packwiz/packwiz
- Decky-Nexus: https://github.com/RedRanger14/decky-nexus
- Linux Mod Organizer (not opened): https://codeberg.org/DoggoNako/linux-mod-organizer
