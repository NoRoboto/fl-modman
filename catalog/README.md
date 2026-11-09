# Catalog

Ready-made `mod.toml` files for public mods. The catalog holds **links and manifests only**, never mod files:
download each mod from its author's page.

Each folder has `mod.toml` and `SOURCE.md` (author, link, archive name, what goes where, and the import command when
the mod needs `--root` or `--type`). Import with:

```sh
fl-modman import <downloaded archive> --manifest catalog/<id>/mod.toml
```

**FL27 column**

- **verified**: imported and applied with fl-modman on SP Football Life 2027 (ROG Ally, Bazzite).
- **claimed**: the mod page says it works with SP Football Life (sometimes an older FL).
- **unknown**: the page does not say, or the mod needs something FL27 may not ship.

| Mod | Type | FL27 | Source |
|---|---|---|---|
| [Cloth FX 1.1-rc](cloth-fx-1-1-rc) | Lua module + DLL | verified | [video](https://youtu.be/y8AvOylMSQk) |
| [Offside VAR 1.0-rc](offside-var-1-0-rc) | Lua module + livecpk + ReShade add-on | verified | [video](https://www.youtube.com/watch?v=RPtgnXa21W4) |
| [Next Gen Graphics 2026](next-gen-graphics) | livecpk | verified | [pesmodding](https://www.pesmodding.com/2026/04/pes-2021-next-gen-graphics-2026-by.html) |
| [Spotify Camp Nou 2026](spotify-camp-nou-2026) | stadium | verified | [pesmodding](https://www.pesmodding.com/2026/10/pes-2021-spotify-camp-nou-2026.html) |
| [Machete Immersive indicators](machete-immersive-indicators) | livecpk | verified | no public page |
| [Mariano Closs V5 (English slot)](mariano-closs-v5-eng) | livecpk | verified | no public page |
| [Machete Immersive camera angles](machete-immersive-camera) | Lua module | claimed | no public page |
| [Boots & Gloves Pack October 2026](boot-root-october-2026) | livecpk | claimed | [pesmodding](https://www.pesmodding.com/2026/10/pes-2021-boots-gloves-pack-boot-root.html) |
| [Christian Pulisic face](christian-pulisic-face) | livecpk (faces) | claimed | [pesmodding](https://www.pesmodding.com/2026/10/pes-2021-christian-pulisic-face-ac.html) |
| [Erling Haaland face](erling-haaland-face) | livecpk (faces) | claimed | [pesmodding](https://www.pesmodding.com/2026/10/pes-2021fl-2026-erling-haaland-short.html) |
| [Premier League 2027 Facepack Vol.1](premier-league-2027-facepack-vol1) | livecpk (faces) | claimed | [pesmodding](https://www.pesmodding.com/2026/08/pes-2021-premier-league-2027-facepack.html) |
| [Real Faces FL26/27 (SmokePatch)](smokepatch-real-faces-fl2627) | `download/*.cpk` | claimed | [pessmokepatch](https://www.pessmokepatch.com/2025/10/faces2627.html) |
| [eFootball 2026 Soundtrack](soundtrack-efootball-2026) | `download/*.cpk` | claimed | [pes-files](https://pes-files.com/efootball-2026-soundtrack-for-pes-2021/) |
| [Referee Kit Server 2025/26](referee-kit-server-2526) | livecpk + Lua module | claimed | [pesmodding](https://www.pesmodding.com/2025/12/referee-kit-server-season-202526-pes.html) |
| [Graphic Menu FC Barcelona 2027](menu-barcelona-2027) | livecpk + Lua module | claimed (FL26) | [pesmodding](https://www.pesmodding.com/2026/07/graphic-menu-fc-barcelona-2027-sp.html) |
| [Graphic Menu FC Bayern 2027](menu-bayern-2027) | livecpk + Lua module | claimed (FL26) | [pesmodding](https://www.pesmodding.com/2026/08/graphic-menu-fc-bayern-munchen-2027-sp.html) |
| [Football Life 2026 Menu (MenuC1987)](menu-c1987-fl26) | livecpk + Lua module | claimed (FL26) | [pesmodding](https://www.pesmodding.com/2025/10/new-football-life-2026-menu-mod-intro.html) |
| [English Callnames v20](english-callnames-v20) | livecpk | claimed (older FL) | [pes-files](https://pes-files.com/pes-2021-commentary-english-callnames-v20/) |
| [Custom Commentary Server 2026](custom-commentary-server-2026) | Lua module | unknown | [pes-files](https://pes-files.com/pes-2021-custom-commentary-server-2026-v1/) |
| [Realistic EPL Camera 2026](realistic-epl-camera-2026) | Lua module | unknown | [pes-files](https://pes-files.com/pes-2021-realistic-epl-camera-mod-2026/) |
| [Realistic TV ReShade preset](realistic-tv-reshade) | loose file | unknown | [pesmodding](https://www.pesmodding.com/2024/01/pes-2021-realistic-tv-broadcast-reshade.html) |
| [ultra_hdgraphic](ultra-hdgraphic) | livecpk | unknown | no public page |

Mods that fl-modman cannot install safely (opaque `.exe` installers, whole `sider.ini` files, save-folder edits) are
listed with the reason in [EXCLUDED.md](EXCLUDED.md).

To add a mod to this list, see the wiki page
[Add your mod to the catalog](https://github.com/NoRoboto/fl-modman/wiki/Add-your-mod-to-the-catalog).
