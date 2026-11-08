# mod.toml

A mod is a folder `mods/<id>/`: the folder name is the id, `mod.toml` describes the mod, and every other file is
payload. Unknown keys are an error. Schema for slice 1 (grilling Q7, Q13):

```toml
name = "Camp Nou 2026"        # optional, display name (defaults to the id)
version = "1.0.0"             # optional, free text
exe = "<sha256 hex>"          # optional guard: on any other FL_2027.exe the mod is disabled with a warning

cpk_root = ['.\livecpk\Camp Nou']           # sider.ini cpk.root lines; must start with .\
lua_module = ["Camp.lua"]                   # sider.ini lua.module lines
map_teams = ["108, 002, Camp Nou, Camp Nou # Barcelona"]   # map_teams.txt rows, one per team id

[[put]]                       # copy a file, or a folder when `from` ends in /
from = "Camp Nou/"            # relative to the mod folder
to = "@stadiums/Camp Nou/"    # @root/path; a `to` ending in / is a folder
```

Roots (`targets/fl27.toml`): `@game`, `@sider`, `@livecpk`, `@modules`, `@stadiums`, `@download`, `@data`.

## The five mod types

| Type | Manifest |
|---|---|
| livecpk | `[[put]]` folder → `@livecpk/<Name>/`, plus `cpk_root = ['.\livecpk\<Name>']` |
| Lua module | `[[put]]` `.lua`/`.ini` files → `@modules/`, plus `lua_module = [...]` |
| Stadium | `[[put]]` folder → `@stadiums/<Name>/`, plus one `map_teams` row with the team id |
| `download/*.cpk` replacement | `[[put]]` file → `@download/<file>.cpk` (the base file is the original kept for undo) |
| Loose files in the game root | `[[put]]` file → `@game/<file>` (e.g. ReShade's `dxgi.dll`) |

## Rules

- The mod list is ordered; on any conflict **the lower mod in the list wins**, and the plan reports it as
  "X overrides N file(s) of Y". Conflicts are: the same file from two mods with different bytes, the same team id in
  `map_teams`, and the same virtual path served by two mods' `cpk_root` folders (new roots go first in `sider.ini`,
  lowest mod first, because Sider's lookup is first-wins).
- New `lua.module` lines go after the base ones, in list order. A `cpk_root` or `lua_module` value already present
  (case-insensitive, `/` = `\`) is left where it is.
- A `map_teams` row replaces every other row of its team, base rows included (Sider would pick one at random). A base
  row equal to it (same slot, name and folder) stays in place.
- A put may not target `sider.ini`, `map_teams.txt` or a user path (logs, caches, `ReShade.ini`, …).
