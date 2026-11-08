# PES 2021 Premier League 2027 Facepack Vol.1

- Converted from eFootball 2026 by Facemaker Pudd (post by G. Leroy, 2026-08-13). 6 faces.
  Archive `Premier_League_Facepack_Vol.1.rar`.
- URL: https://www.pesmodding.com/2026/08/pes-2021-premier-league-2027-facepack.html
- Type: livecpk (shared `Faces` root). FL27: **claimed** ("PES 2021 & SP Football Life").

## Original instructions (brief)

> Find the folder with the new faces ("usually identified by numbers"), copy it to
> `...\livecpk\Faces\Asset\model\character\face\real`, add `cpk.root = ".\livecpk\faces"` if not already done.

## Mapping

Same as `erling-haaland-face`; a pack just has several numbered folders, all copied by the one folder put.

## Archive layout and import

The numbered face folders at the top of the archive [U]:

```sh
fl-modman import Premier_League_Facepack_Vol.1.rar --root . --type loose --manifest catalog/premier-league-2027-facepack-vol1/mod.toml
```

`--manifest` uses this `mod.toml` instead of the generated one (see `erling-haaland-face/SOURCE.md`).
