# Mods left out of the catalog

Each one needs an operation the five mod types (`docs/mod-toml.md`, grilling Q7/Q13) cannot express, or one that
fl-modman refuses by design (install-rules §5.5). None counts toward the 20.

| Mod | Source | Why it is out |
|---|---|---|
| SmokePatch match commentaries (`com269_xxx.exe`) | https://www.pessmokepatch.com/2025/10/flcoms.html ; https://pes-files.com/pes-2021-update-english-commentary-for-fl-2026/ | Opaque `.exe` installer: its writes are undocumented (R4). |
| SmokePatch BGM (`BGM_xxx.exe`) | https://www.pessmokepatch.com/2024/06/BGM.html | Opaque `.exe` installer (R4). `soundtrack-efootball-2026` is the expressible alternative. |
| Machete FL27 ReShade Setup (`FL27_Reshade_Setup_-_Machete.rar`, local) | Ally `~/Games/FL27-src/mods` | Ships `ReShade_Setup_6.8.0_Addon.exe`; the `dxgi.dll` comes from that installer (R4). Its "Copy to main game folder" files alone (DLSS dlls, `.addon64`, preset `.ini`) are loose files, but useless without the installer's output. |
| Machete FL27 Immersive Sider Config (`FL27_Immersive_Sider_Config.rar`, local) | Ally `~/Games/FL27-src/mods` | A whole replacement `sider.ini`. A put may not target `sider.ini`; it is a managed document built from every mod's lines (R6). |
| Soccer Revolution 2026 v12 (gameplay + menu + ReShade) | https://www.pesmodding.com/2026/03/pes-2021-soccer-revolution-2026-v12.html | Runs `Reshade_Setup_6.7.1.exe` (R4) and replaces `PES2021.exe` (an exe patch: not the FL27 exe). Its menu part alone would fit (livecpk + `UIColors.lua`). |
| Option file 2026/27 (`EDIT00000000`) | https://pes-files.com/pes-2021-new-option-file-season-2026-27/ | Writes the save folder, which the game rewrites on every save (R5). FL27 also uses another edit-file key (https://www.pessmokepatch.com/p/customizations-and-mods.html). |
| Road to Football Life 2027 patch | https://pes-files.com/pes-2021-patch-road-to-football-life-2027/ | "Replaces the existing lua lines" wholesale (R6) and copies an `EDIT00000000` into the save (R5). |
| Carabao Cup mod (FL26 / PES 2021) | https://www.pesmodding.com/2026/10/sp-football-life-2026-carabao-cup-mod.html | "EXE Version: 2.2 only" (FL26); "cannot be used directly with SP Football Life 2027; a separate version would be required". The PES 2021 build is hard-coded to 1.7.2.0. |
| Goalnets module v0.6.1 | https://pes-files.com/pes-2021-goalnets-module-v0-6-1-aio-update-2026/ | "Cannot be used with other nets mods", and the FL27 base loads `nets.lua`: it needs a base `lua.module` line removed. Manifests have no delete. |
| Banner Server v3 (Balkan PES BOX) | https://www.pesmodding.com/2025/03/pes-2021-banner-server-v3-by-balkan-pes.html | `BannerServer.lua` must load **above** the stadium server; new modules always go after the base ones (no order constraints, Q13). |
| Turf and Shape Loader v3.3 | https://www.pesmodding.com/2025/11/sp-football-life-2026-v12-essentials.html | Its modules must sit **between** base lines (`WeatherConditions` < `TurfLoader` < `StadiumServer`). |
| 56 Extra Konami Stadiums | same Essentials page | Disables the stadium server module and deletes `content\stadium-server` (deletes), and replaces `Data\dt30_g4.cpk` (a Data cpk, not `download/`). |
| Etihad Stadium Expansion V2 | https://www.pesmodding.com/2026/07/pes-2021fl-2026-etihad-stadium.html | "Delete any previous version": FL27 ships an `Etihad Stadium` folder, so a clean install needs that base folder removed (no delete; a put over it would mix old and new files). Also wants Stadium Server 1.80 + Banner Server 3.0 (FL27: `stadiums.lua` 1.60). |
| Stadio Renato Dall'Ara update (turf + UCL ads) | https://www.pesmodding.com/2025/09/pes-2021-stadio-renato-dallara-update.html | Copies `AddOn` / `Asset` into another mod's stadium folder (cross-ownership, R7). |
| World Cup 2026 Turf / Realistic Turf Full HD (Moiduran2) | https://www.pesmodding.com/2026/07/pes-2021-world-cup-2026-turf.html ; https://www.pesmodding.com/2026/05/pes-2021-realistic-turf-full-hd-by.html | `AddOn` into the stadium folder you pick, plus renaming by turf id (R7). |
| EPL 2026/27 Animated Adboard Pack V1.0 | https://www.pesmodding.com/2026/10/pes-2021-epl-202627-animated-adboard.html | Edits `config.xml` lines inside an existing adboard pack: a line-level edit of a file no manifest field owns. |
| Premier League 2026/27 full kitpack (kitserver) | https://www.pesmodding.com/2026/09/premier-league-202627-full-kitpack-pes.html | Needs kitserver `map.txt` rows; only `map_teams` rows are a manifest field. |
| FIFA World Cup 2026 Concept scoreboard | https://www.pesmodding.com/2025/12/pes-2021-fifa-world-cup-2026-concept.html | Needs `map_competitions.txt` rows in the scoreboard server. |
| LaLiga 2026/27 substitution board | https://www.pesmodding.com/2026/10/pes-2021-laliga-ea-sports-202627.html | Needs a SubBoard server; FL27's `content/` has no `SubBoard` folder or module [L, Ally]. |
| SmokePatch FL Sider Stadiums (country packs) | https://www.pessmokepatch.com/2024/11/siderstadiums.html | Expressible (one folder put per stadium + one `map_teams` row each), but the 12–50 rows per pack are in the archive's `map_teams.txt`, not on the page. Not written, to avoid inventing team ids; build one manifest per pack from that file. |
| FL Switcher options (theme, LUT, controller, lockdown, replay, gameplay) | https://www.pessmokepatch.com/p/customizations-and-mods.html | Already managed by the game's own `FL27 Switcher.exe`, which swaps the same `download/SMK_Add*.cpk` and `Data/dt18_all.cpk` files; a mod writing them would fight the Switcher. |
