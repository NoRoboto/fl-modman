# fl-modman

A mod manager for **SP Football Life 2027** (the PES 2021 mod) on **Linux with Proton**.
You keep a list of mods; fl-modman works out what to copy and which `sider.ini` and
`map_teams.txt` lines to add, applies it, and can undo any apply.

> Screenshot: not available yet.

## What it does

- Imports mods from zip, 7z, rar or a folder, and detects the mod type: livecpk folder, Lua module,
  stadium, `download/*.cpk` replacement, or loose files in the game folder.
- Keeps one ordered mod list. When two mods touch the same file, **the lower mod wins**, and the
  conflict is shown.
- Shows the plan before it writes anything. Every apply is a numbered generation you can roll back to.
- Detects files changed outside fl-modman (for example, a game update) and asks before it continues.
- Edits the game settings (`settings.dat` and the in-game rendering options).
- Gamepad-first GUI (works in Steam Gaming Mode) plus a CLI.

Scope: Linux/Proton and FL27 only. The GUI is in English.

## Install

1. Download `fl-modman-<version>-x86_64.AppImage` from the Releases page.
2. Make it executable: `chmod +x fl-modman-*.AppImage`
3. Run it. To use it from Steam Gaming Mode, add it as a non-Steam game.

rar archives need a system tool: `7z`, `7zz`, `unrar` or `bsdtar`. zip and 7z work out of the box.

Build from source (Rust 1.96+): `cargo build --release`, or `scripts/build-appimage.sh` for the AppImage.

## Quick start

### GUI

Run the AppImage. It looks for FL27 installs, lets you pick one, and then shows the mod list.
Add a mod, review the plan, apply. The History tab rolls back to an earlier generation.
Controls and screens: [docs/gui.md](docs/gui.md).

### CLI

The AppImage runs the CLI with `cli` as the first argument (`./fl-modman.AppImage cli --help`).
From a source build the binary is `fl-modman`.

```sh
fl-modman detect                          # list FL27 installs and Wine prefixes
fl-modman init ~/Games/FL27               # workspace in ~/Games/fl-modman + base snapshot
export FL_MODMAN=~/Games/fl-modman        # or pass -w <workspace> to each command

fl-modman import ~/Downloads/some-mod.zip # added at the bottom of the list (it wins)
fl-modman apply                           # show the plan only
fl-modman apply --yes                     # write it as a new generation

fl-modman history                         # list generations
fl-modman rollback 1 --yes                # go back to generation 1
```

Stadiums need a team: `fl-modman import <src> --team <id>[,<slot>]`.
All commands: `fl-modman --help` and [docs/engine.md](docs/engine.md).

The `init` snapshot is your base. Mods you installed by hand before that stay in place, but
fl-modman does not manage them. Re-import them to manage them.

## Writing a mod.toml

Each mod has a `mod.toml` that says what to copy and which lines to add. Import generates one;
you can also write your own. Format: [docs/mod-toml.md](docs/mod-toml.md).
`catalog/` has ready-made manifests for some public mods. It holds links only, no mod files.

## Safety

- **Mods can contain code.** `.lua` modules run inside the game through Sider, `.addon64` files are
  ReShade add-ons (native code), and some mods ship `.exe` or `.dll` files. Only install mods from
  sources you trust. fl-modman copies files; it does not scan or sandbox them.
- fl-modman refuses to apply while the game is running.
- A game file a mod replaces is saved first, in `<workspace>/originals/`, so undo can restore it. This
  covers only files that were in the base snapshot taken at `init`.
- fl-modman does not edit Steam files. `fl-modman doctor` prints the Proton launch options to paste yourself.

## License

MIT. See [LICENSE](LICENSE). Not affiliated with Konami or SmokePatch.
