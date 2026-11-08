# Christian Pulisic face (AC Milan 2027)

- Facemaker deNv (post by G. Leroy, 2026-10-05). Archive `C._Pulisic_by_deNv.rar`.
- URL: https://www.pesmodding.com/2026/10/pes-2021-christian-pulisic-face-ac.html
- Type: livecpk (shared `Faces` root). FL27: **claimed** (PES 2021 and SP Football Life 2026). Player id not given.

## Original instructions (brief)

> Locate the folder with the new face ("usually identified by numbers"), copy it to
> `...\livecpk\Faces\Asset\model\character\face\real`, add `cpk.root = ".\livecpk\faces"` if not already done.

## Mapping

Same as `erling-haaland-face`: the numbered folders → the shared `Faces` root's `real/` folder, one shared `cpk_root`.

## Archive layout and import

The numbered face folder at the top of the archive [U]:

```sh
fl-modman import C._Pulisic_by_deNv.rar --root . --type loose --manifest catalog/christian-pulisic-face/mod.toml
```

`--manifest` uses this `mod.toml` instead of the generated one (see `erling-haaland-face/SOURCE.md`).
