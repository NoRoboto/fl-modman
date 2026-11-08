# PES 2021 Custom Commentary Server 2026 v1

- Authors: efootball_multiverse, xusmendev. Posted 2026-02-24. Archive `PES_2021_Custom_Commentary_Server_v1.0.rar`.
- URL: https://pes-files.com/pes-2021-custom-commentary-server-2026-v1/
- Type: Lua module. FL27: **unknown** (needs Sider 7.3.4; FL27 most likely ships 7.3.3 [U]).

## Original instructions (brief)

> Copy `custom-comment-server.lua` into your Sider modules folder; add `lua.module = "custom-comment-server.lua"`.
> Create `Sider\content\custom-commentary-server\map.txt` (CSV: stadium, competition id, home id, away id,
> audio path; `*`, ranges `100-120` and lists `108.109` allowed).

## Mapping

- The module file + one `lua_module` line.
- `map.txt` is written by the user, not shipped, so it is not in the manifest. Keep it in a small mod of its own
  (`map.txt` → `@sider/content/custom-commentary-server/`) so it is undoable too.

## Archive layout and import

`custom-comment-server.lua` somewhere in the archive [U]:

```sh
fl-modman import PES_2021_Custom_Commentary_Server_v1.0.rar --manifest catalog/custom-commentary-server-2026/mod.toml
```

The payload lands at `mods/<id>/modules/`; `--manifest` uses this `mod.toml`, which puts only the `.lua` (the generated one puts the whole folder).
