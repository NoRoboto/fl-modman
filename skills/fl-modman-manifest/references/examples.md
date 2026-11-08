# Worked examples (from `catalog/`)

Each was run through `fl-modman import --manifest` + `fl-modman apply` (dry run) on the synthetic FL27 tree
`tests/fixtures/fl27-game`.

## 1. Multi-part menu: livecpk + Lua module + content (`catalog/menu-barcelona-2027`)

Readme: "copy the content, livecpk, and modules folders to your Sider 7 folder"; add
`cpk.root = ".\livecpk\Menu Barcelona"`, `cpk.root = ".\livecpk\UIColors"`, `lua.module = "UIColors.lua"`;
"CommonLib should be the first entry".

Plain `import` stops with candidates (`livecpk/Menu Barcelona`, `livecpk/UIColors`, `modules`): one root per mod.
Use the carrier:

```sh
fl-modman import Menu_Barcelona.rar --root . --type loose --manifest mod.toml   # payload: mods/menu-barcelona/game/{content,livecpk,modules}
```

```toml
name = "Graphic Menu FC Barcelona 2027"
cpk_root = ['.\livecpk\Menu Barcelona', '.\livecpk\UIColors']
lua_module = ["UIColors.lua"]

[[put]]
from = "game/livecpk/"
to = "@livecpk/"

[[put]]
from = "game/modules/"
to = "@modules/"

[[put]]
from = "game/content/"
to = "@sider/content/"
```

CommonLib: nothing to write (FL27's first base module). Dry run: `+ cpk.root` ×2, `+ lua.module` ×1, the puts.

## 2. Stadium with a team row, companion servers skipped (`catalog/spotify-camp-nou-2026`)

Readme: "Stadium server 1.80 is a must"; copy `Banners-Coaches` to the sider folder; add two `cpk.root` lines and
`lua.module = "BannerServer.lua"`, `"StadiumServer.lua"`; place the stadium and edit `map_teams.txt` (no row given).

- No row in the readme → ask the team: Barcelona = 108. The slot `002` is the one of FL27's stock 108 row.
- `StadiumServer.lua` / `BannerServer.lua`: FL27 already runs `common\stadiums.lua` 1.60; a second server would
  compete with it. Skip `Banners-Coaches` and report it (the stadium works without banners and real coaches).

```sh
fl-modman import Spotify_Camp_Nou_2026.rar --root "<path>/Spotify Camp Nou 2026" --manifest mod.toml
```

```toml
name = "Spotify Camp Nou 2026"
map_teams = ["108, 002, Spotify Camp Nou 2026, Spotify Camp Nou 2026 # Barcelona"]

[[put]]
from = "Spotify Camp Nou 2026/"
to = "@stadiums/Spotify Camp Nou 2026/"
```

The row replaces the stock `108` row (no random stadium). Detection takes the folder as a stadium (livecpk folders
next to `AddOn/`); without `--manifest`, `--team 108,002` generates the row (minus the comment).

## 3. download cpk with delete + rename (`catalog/soundtrack-efootball-2026`)

Readme (FL variant): extract "eFutBol26 BGM.cpk" into `download`, delete "SMK_Addmusic.cpk", rename the new file to
"SMK_Addmusic.cpk".

"Delete A + rename B to A" is one put onto `A`; fl-modman keeps the original `A` for undo.

```sh
fl-modman import <archive> --root <folder holding the cpk> --type download --manifest mod.toml   # payload: mods/<id>/download/eFutBol26 BGM.cpk
```

```toml
name = "eFootball 2026 Soundtrack (FL variant)"

[[put]]
from = "download/eFutBol26 BGM.cpk"
to = "@download/SMK_Addmusic.cpk"
```

Dry run: `+ put download/SMK_Addmusic.cpk`. The generated manifest would have kept the cpk's own name, which the
game never reads; that is why the manifest is rewritten.

## A refusal (`catalog/EXCLUDED.md`)

Goalnets module v0.6.1: "cannot be used with other nets mods", and FL27's base loads `nets.lua`. Installing it means
removing a base `lua.module` line, and manifests have no delete: refuse, and say so.
