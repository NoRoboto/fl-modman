# Spotify Camp Nou 2026

- Author: minhanh158 (post by G. Leroy, 2026-10-23). Archive `Spotify_Camp_Nou_2026.rar`.
- URL: https://www.pesmodding.com/2026/10/pes-2021-spotify-camp-nou-2026.html
- Type: stadium. FL27: **verified** (installed on the Ally FL27 in `content/stadiums` with row `108, 002`; design.md §11).

## Original instructions (brief)

> "Stadium server 1.80 is a must." Copy everything inside `Banners-Coaches` to your sider folder. "Extract the stadium
> and place it where you want", then edit `map_teams.txt`. Add `cpk.root = ".\livecpk\Add-On\Banner"`,
> `cpk.root = ".\livecpk\Add-On\Real_Coach"`, `lua.module = "BannerServer.lua"`, `lua.module = "StadiumServer.lua"`.

## Mapping

- Stadium folder → `@stadiums/Spotify Camp Nou 2026/`; one `map_teams` row for team 108 (Barcelona), slot `002`
  (the slot of FL27's stock 108 row). The row replaces the stock `108, 002, Spotify Camp Nou, common\Camp Nou` row,
  so the stadium is never picked at random.
- Not mapped: `Banners-Coaches` (BannerServer.lua + StadiumServer.lua + two roots). FL27 ships its own
  `common\stadiums.lua` 1.60; a second stadium server would compete with it. On the Ally the stadium works without
  them (banners and real coaches are skipped).

## Archive layout and import

The stadium folder holds `AddOn/`, `Asset/`, `common/` [L, Ally]. Its path inside the archive is not documented [U]:

```sh
fl-modman import Spotify_Camp_Nou_2026.rar --root "<path>/Spotify Camp Nou 2026" \
  --manifest catalog/spotify-camp-nou-2026/mod.toml
```

Detection takes the folder as a stadium (livecpk folders next to `AddOn/`; checked on a synthetic folder of this shape
in `crates/fl-core/tests/import.rs`). The row comes from this `mod.toml`, so no `--team` (with `--manifest` it is an
error).

The payload lands at `mods/<id>/Spotify Camp Nou 2026/`; without `--manifest`, `--team 108,002` generates this `mod.toml`
without the `# Barcelona` comment.
