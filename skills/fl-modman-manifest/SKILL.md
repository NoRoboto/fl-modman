---
name: fl-modman-manifest
description: "Trigger: mod.toml, fl-modman manifest, package/convert a Football Life or PES 2021 Sider mod, livecpk, lua.module, map_teams, stadium. Turn a mod archive + readme into a verified mod.toml."
license: MIT
metadata:
  author: "NoRoboto"
  version: "1.0"
---

## Activation Contract

Use to write or review a fl-modman `mod.toml` for an SP Football Life 2027 / PES 2021 (Sider 7) mod, given its
archive and its readme or download page. `docs/mod-toml.md` is the schema; this skill is the procedure.

## Hard Rules

- Keys: `name`, `version`, `exe`, `cpk_root`, `lua_module`, `map_teams`, `[[put]] from/to`. Any other key fails to load.
- Every value traces to the readme, the page or the archive listing. Unknowns go in your report as [U], never guessed.
- `from` is relative to `mods/<id>/` after import (payload table, step 2). `to` is `@game @sider @livecpk @modules
  @stadiums @download @data` + path; a trailing `/` means folder. `cpk_root` values start with `.\`.
- `map_teams` row: `teamId, slot, name, folder # comment`, one row per team.
- **Refuse** (no manifest; say which rule): `.exe` installers; save/edit files (`EDIT00000000`, option files);
  replacing `sider.ini`, `sider.dll`, `exe.name` or the game exe; deleting a base file, folder or `lua.module` line;
  a module that must load above or between base lines; rows in other tables (kitserver `map.txt`,
  `map_competitions.txt`, `config.xml`); copying into another mod's or the base's folder (stadium `AddOn`).
  When part of the mod is expressible, offer that part as its own mod.

## Decision Gates

| Readme phrase | Manifest |
|---|---|
| copy `X` to livecpk + `cpk.root = ".\livecpk\X"` | put `X/` → `@livecpk/X/`; `cpk_root = ['.\livecpk\X']` |
| face folder (numbers) to `livecpk\Faces\…\face\real` | put → `@livecpk/Faces/Asset/model/character/face/real/`; `cpk_root = ['.\livecpk\Faces']` |
| copy `.lua` to modules + `lua.module = "M.lua"` | put → `@modules/`; `lua_module = ["M.lua"]` in readme order |
| copy `content`, `livecpk`, `modules` to the Sider folder | folder puts → `@sider/content/`, `@livecpk/`, `@modules/` + their lines |
| stadium to `content\stadium-server` + map_teams line | put → `@stadiums/<F>/` + that row (FL27 reads `content/stadiums`) |
| replace / "delete A, rename B to A" in `download` | put `B` → `@download/A` (the original is kept for undo) |
| copy dll / preset ini next to the exe | put → `@game/` |
| "at the very top", "if not already done", CommonLib / nesalib first | nothing: new roots go first, values dedupe, FL27 loads both libs first |
| needs StadiumServer / BannerServer / Sider ≥ v | skip servers FL27 ships (`common\stadiums.lua` 1.60); report the requirement |
| works only on one exe version | `exe = "<sha256>"` if the hash is known, else report it |

Stadium team: take the id and slot from the readme row; with no row, ask the user for the team id and use the slot
of that team's row in the base `map_teams.txt` (`fl-modman import --team <id>[,<slot>]`).

## Execution Steps

1. List the archive (`unrar lb`, `7z l`). Done when every readme step maps to a gate row or a refusal.
2. Write the manifest (`catalog/<id>/mod.toml`). `from` is the payload in `mods/<id>/` after import: livecpk,
   stadium → basename of `R` (archive stem when `R` is `.`); lua → `modules/`; download → `download/`; loose →
   `game/`. Multi-part mods and face folders with unknown ids: `--root . --type loose`, the verbatim **carrier**
   under `game/`. Prefer folder puts; list files only to leave some out.
3. Import with it: `fl-modman import <archive|dir> [--root R] [--type T] --manifest catalog/<id>/mod.toml`. Done
   when it loads: a `from` that matches no payload file fails the import and nothing is added. Stadiums are
   detected by their `stadium*` entry or `AddOn/` folder; `--team id,slot` only generates a row without
   `--manifest` and is an error on anything else.
4. **Dry run**: `fl-modman apply` without `--yes` reloads every manifest and prints the plan. Done when its
   `cpk.root` / `lua.module` / `map_teams` lines and puts match the readme, and every conflict is explained. A
   "managed document or user file" error names a file to drop (`sider.ini`, `map_teams.txt`,
   `content/*/config.ini`, `ReShade.ini`). Pass `--yes` only when the user asks.

## Output Contract

The `mod.toml`; the import command; the type; FL27 status (verified / claimed / unknown, with evidence); what was not
mapped and why; the dry-run plan summary.

## References

Paths from the fl-modman repo root, except `references/` (next to this file).

- `docs/mod-toml.md`: schema and merge rules. `crates/fl-core/targets/fl27.toml`: roots and user paths.
- `references/examples.md`: three worked examples from `catalog/`. `catalog/*/SOURCE.md`, `catalog/EXCLUDED.md`.
- `research/install-rules.md` §3 (corpus) and §5.5 (refusal rules R1–R7).
