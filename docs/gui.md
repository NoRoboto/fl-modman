# GUI (slice 3)

`fl-modman-gui` (`crates/fl-gui`) is the user-facing front end (grilling Q3, Q16, Q17): egui/eframe, gamepad-first,
English only. It calls `fl-core` directly; every long operation (detect, init, load, plan, apply, import, rollback,
recover, accept-drift, restore, conflict analysis) runs on a background thread behind a spinner. Errors go to a
dismissable banner.

## Run

```sh
cargo build --release -p fl-gui
target/release/fl-modman-gui                 # fullscreen (Gaming Mode)
target/release/fl-modman-gui --windowed      # desktop
target/release/fl-modman-gui -w <workspace>  # or FL_MODMAN=<workspace>
FL_MODMAN_SCALE=1.4 target/release/fl-modman-gui   # pixels per point (default 1.6, 7" 1080p screen)
```

Workspace: `-w`, else `$FL_MODMAN`, else `<game>/../fl-modman` for the picked game (the CLI rules). Without `-w`
or `$FL_MODMAN` the app runs detection and opens the workspace directly when exactly one detected install has
one; otherwise it shows **Pick game**.

## Controls

| Gamepad | Keyboard | Action |
|---|---|---|
| D-pad / left stick | arrows | move the focus (stick dead zone 0.5; repeat after 400 ms, then every 150 ms) |
| A | Enter | activate |
| B | Escape / Backspace | back, cancel, dismiss the banner, up one folder |
| X | Space | toggle the focused mod |
| Y | M | move mode: Up/Down carry the mod, A/B/Y drop it |
| L1 / R1 | Q / E, PageUp / PageDown | previous / next tab |

Mouse and touch use the widgets directly; drag a mod by its ☰ handle to reorder. The hint bar at the
bottom lists the controls of the current screen (`[A]`, `[B]`… as text: the bundled fonts have no circled
letters).

## Design system (`ui.rs`)

Handheld-first: a 7" 1080p screen at arm's length, dark theme. Screens use only these tokens and components.

| Token | Value |
|---|---|
| Spacing | `S1` 8, `S2` 16, `S3` 24, `S4` 32 |
| Text | `TITLE` 26, `BODY` 20, `HINT` 16 |
| Colours | background `#101216`, surface `#21252D`, accent `#5AB4FF`, warning `#FFC440`, muted `#969EAC` (text `#ECEFF4`) |
| Radius | 8 |
| Focus ring | 3 px accent outline, the only focus indicator (hover and press look like rest) |

Components: **Card** (titled box), **Row** (label left, meta right, ⚠ badge lines, optional lead such as the
toggle; `hot` = accent tint for the mod being moved / the picked generation), **Toggle** (ON/OFF pill),
**Stepper** (`◀ value ▶`, `*` while pending), **Button** via the action bar (`actions`: right-aligned, the last
label is the primary, accent-filled; on a screen that opens with only buttons it takes the focus), **Badge**
(⚠ warning line), **HintBar** (fixed bottom panel). Layout: one primary action per screen, bottom-right above
the hint bar; scrolled content in `body`; no text explaining controls; long lists collapsed (the "Already
installed (base)" card opens with A, one row per item).

## Screens

1. **Pick game** (no workspace): detected installs paired with their prefixes, "Choose another folder…" and
   "Choose a Wine prefix…" (in-app browser), "Scan again". The choice is always confirmed, then init scans the game.
2. **My mods**: the ordered list (the lower mod wins) as rows with ON/OFF, type and file count, move mode / drag,
   ⚠ "X overrides N file(s) of Y" badges, the collapsed "Already installed (base)" card, and **Apply changes** → plan summary → confirm → apply. An interrupted generation shows Finish / Roll
   back; drift shows the changed files with Accept as new base / Restore.
3. **Add mod**: browse to a zip/7z/rar or folder → import → preview (type, root, files, conflicts) → Keep or
   Discard. Ambiguous roots are picked from a list; a stadium asks its team id on the numeric picker.
4. **History**: generations newest first (note, age, mods); A on a row or "Go back to this" (the last focused
   row) rolls back after a confirm.
5. **Game settings**: the `settings.dat` fields (Left/Right or A cycle a value; the UDP port uses the numeric
   picker), then **Rendering (in-game options)** from `GRAPHICS000000` (preset first; changing any other field shows
   the preset as `custom` at once, choosing a preset sets every field). One Save writes both files, each with a
   backup, after a confirm, refused while the game runs; plus the Proton launch-options diagnostic (read only).
6. **Storage**: measured on a background thread when the tab opens and after Free, never per frame
   (`Workspace::usage`, stat only). A ⚠ line when the workspace and the game are on different filesystems, a
   Workspace card (mods, only in the workspace, originals, history, staging leftovers, total, can be freed), then one
   focusable row per mod: id, state (applied / on, not applied yet / off, still in the game / off), file count, size
   and bytes shared with the game. **Free X** (only when something is reclaimable) lists what goes in a confirm, then
   runs `remove_disabled` and measures again.

## Code

| File | Contents |
|---|---|
| `main.rs` | arguments, viewport, style |
| `app.rs` | workspace, jobs, tabs, banner, confirm dialog, frame loop |
| `input.rs` | gilrs + keyboard → actions, stick dead zone, auto-repeat |
| `nav.rs` | the focus layer every screen registers its widgets with |
| `ui.rs` | design tokens and components |
| `widgets.rs` | numeric picker, folder browser |
| `pick.rs`, `mods.rs`, `add.rs`, `history.rs`, `settings.rs`, `storage.rs` | the six screens |
