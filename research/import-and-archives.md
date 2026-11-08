# Import of manifest-less archives + archive extraction (fl-modman)

Date: 2026-11-02. Method: web fetch/search. "UNVERIFIED" marks claims I could not
confirm from a primary source in this pass.

## 1. Mod Organizer 2 (MO2)

Primary source read: `installerquick.cpp` (the "Simple" installer plugin)
https://github.com/ModOrganizer2/modorganizer-installer_quick
(raw: .../master/src/installerquick.cpp). Official MO2 doc pages for the
installer were not fetchable (404 / redirect); dialog behaviour below comes from
community guides (see Sources).

Flow for a plain archive (no manifest):
1. Installers run by priority; the first whose `isArchiveSupported` is true wins.
   Specialised ones (FOMOD, BAIN, game-specific) go first, "Simple" is a generic one.
2. Simple installer, `getSimpleArchiveBase`: at each tree level, ask the game plugin's
   `ModDataChecker` "is this level a valid data root?". If yes, that level is the root.
   If no and the level has exactly ONE entry and it is a directory, descend into it
   and repeat. Otherwise log "Archive is not a simple archive." and give up.
   Extra rule: one dir named like the game data folder + only text/image files
   (txt, pdf, md, jpg, png, bmp) beside it = "data + readme" layout; data dir is unwrapped.
3. If the checker reports the tree as `FIXABLE`, it calls `checker->fix()` (game-specific
   rewrite, e.g. move loose files into the right subfolder).
4. User sees a small `SimpleInstallDialog`: mod name field, OK / Manual / Cancel.
   Skipped when the plugin setting `silent` is on.
5. If detection fails (or user picks Manual): Manual installer = full archive tree view.
   User right-clicks the folder that should map to game Data and chooses
   "Set data directory" (shown as `<data>`). Dialog warns "No game data at top level"
   when nothing valid is at the root, "Looks good" otherwise. Only ONE data dir can be
   chosen; multi-root archives are a known pain (users re-pack by hand).
6. Re-install of a same-name mod asks Merge / Replace / Rename.

Key idea: root detection = (a) game-defined predicate "does this dir look like game
data" + (b) unwrap single wrapper folders + (c) human fallback with a tree picker.

FOMOD in MO2: handled by a separate installer plugin (wizard UI). BAIN: separate plugin.
Manual button in the FOMOD wizard falls back to the manual tree installer.

## 2. Vortex

Primary sources NOT fetched (InstallManager.ts / api.d.ts returned 404 via raw URLs).
What I could confirm (Vortex legacy wiki + extension examples, via search):
- Extensions call `registerInstaller(id, priority, testSupported, install)`.
  Numeric priority sets the order installers are tried (25, 30, 50, 99 seen in examples).
  Direction (low-first) UNVERIFIED.
- `testSupported(files, gameId)` returns `{ supported, requiredFiles }`; if false
  Vortex tries another, more generic installer (docs: "Creating a game extension").
- `install(...)` returns a list of instructions (copy source->destination, etc.),
  i.e. installers are pure functions archive-file-list -> file mapping. Good design
  to copy conceptually.
- Mod types are a flag on the mod, separate from the installer (Nexus forum 8548433).
  Extensions register `modType` tests that look at the instruction list to tag a mod.
- Built-in chain (from memory, UNVERIFIED here): FOMOD installer, then a generic
  "fallback" installer that copies everything, with root detection via game-supplied
  hints (stop patterns / `ignoreConflicts`/`mergeMods` flags on the game).
- Conflict UI: Vortex shows a file-level conflict graph and per-pair "mod A loads
  before/after B" rules; last-deployed wins. (UNVERIFIED in this pass.)

## 3. FOMOD format

Sources: https://fomod-docs.readthedocs.io/en/latest/tutorial.html ;
https://stepmodifications.org/wiki/Guide:FOMOD/ModuleConfigXML ;
Rust parser https://docs.rs/crate/fomod/latest (MIT/Apache, structs Config, InstallStep,
Group, Plugin, DependencyPattern, ConditionalInstallPattern).

Files: `fomod/info.xml` (metadata) + `fomod/ModuleConfig.xml` (logic).
ModuleConfig.xml can express:
- `moduleName`, `moduleImage`; `moduleDependencies` (fileDependency Active/Inactive/Missing,
  gameDependency min version, nested And/Or).
- `requiredInstallFiles`: always-installed `file`/`folder` entries.
- `installSteps` -> `installStep` (pages) with optional `visible` condition.
- `group` types: SelectAny, SelectAll, SelectExactlyOne, SelectAtMostOne, SelectAtLeastOne.
- `plugin` (an option): name, description, image, `files`, `typeDescriptor`
  (Optional/Required/Recommended/NotUsable, or conditional `dependencyType`).
- File mapping: `file`/`folder` with `source`, `destination`, `priority`, `alwaysInstall`,
  `installIfUsable` (attributes from the FOMOD schema; tutorial page did not list them,
  UNVERIFIED here).
- `conditionFlags` set on selection; `flagDependency` tests them.
- `conditionalFileInstalls` -> `patterns` -> `dependencies` + `files`.
Cost: a wizard UI + condition evaluator + XML parser (UTF-16 files are common).

PES/FL world: searched pesmodding.com and general web for "mod manager / mod.json /
manifest / fomod" for PES 2021 / Football Life. Found NOTHING: no manifest standard,
no FOMOD usage. Mods ship as archives (mostly .rar, some .zip/.7z) with a readme and
either (a) a folder tree to drop into the game dir / Sider `livecpk`, (b) a `.cpk` to
register in `sider.ini` (`cpk.root`) or the `download` folder via DpFileList, or
(c) loose files for Sider content roots (faces, kits, etc.). Sources disagree on
locations (livecpk vs download). Search coverage was shallow (SEO-heavy results);
evoweb.uk forum not checked directly.

## 4. Lessons / pitfalls

- Wrong root is THE failure mode: wrapper folder(s) around the real tree
  (`ModName/ModName v2/content/...`). MO2 mitigates with single-child unwrapping
  and a manual picker; forums are full of "set data directory" confusion.
- Multi-root archives (two top-level folders that belong in different places) do not
  fit a single "data dir" model; users re-pack. Plan for per-folder mapping or reject.
- Readme/screenshots next to the data folder must not defeat detection (MO2's
  text/image allowance rule).
- Detection must be game-defined, not generic: a predicate on known directory/file names.
  BAIN does the same: dirs matched case-INSENSITIVELY against a list of standard game dirs;
  everything else skipped unless "Has Extra Directories" (Wrye Bash Readme).
  BAIN package kinds: Simple, Complex (sub-packages), Complex/Simple, Scattered
  (needs a FOMOD/wizard). Authors are told to avoid Scattered.
- Case sensitivity: Windows/NTFS is case-insensitive; Linux ext4/btrfs is not. Mods
  authored on Windows mix `Faces`/`faces`. Forum reports of "silent load failure" from
  case mismatch exist for other games (Factorio, Arma 3, RimWorld). Wine/Proton
  case-folds lookups on its side (from my knowledge, UNVERIFIED here), but native
  Linux code (our manager, extraction, collision detection) does not. Rule: normalise
  to ONE canonical case at install time and compare case-insensitively when detecting
  conflicts, or two "different" files will collide in-game.
- Zip-slip / absolute paths / `..` in archive entries: sanitise on extract (generic).
- Windows backslashes in zip entry names; non-UTF8 names (cp437/Shift-JIS) in old PES
  archives.

## 5. Archive extraction in Rust

| crate | formats | license | notes |
|---|---|---|---|
| `zip` 9.2.2 | zip | MIT | pure Rust, fine |
| `sevenz-rust2` 0.24.2 | 7z | Apache-2.0 | pure Rust, maintained; features bzip2/ppmd/deflate/zstd/brotli/lz4 |
| `sevenz-rust` 0.6.1 | 7z | Apache-2.0 | last release 2024-07; prefer `sevenz-rust2` |
| `unrar` 0.5.8 / `unrar_sys` | rar (RAR4+RAR5) | wrapper MIT OR Apache-2.0 | vendors UnRAR C++ source, statically compiled; needs C++ toolchain at build |
| `compress-tools` 0.16.1 | everything libarchive reads | MIT OR Apache-2.0 | needs system libarchive; RAR/RAR5 read only "with limitations" |

UnRAR license (vendored `license.txt`, https://github.com/muja/unrar.rs):
free to use/redistribute in software to handle RAR archives, source may be modified if
the license paragraph is kept, BUT it may not be used to develop a RAR-compatible
archiver or re-create the RAR compression algorithm. It is not an OSI licence and the
text says nothing about GPL; treat it as GPL-incompatible (FSF/distro position: non-free).
So: shipping the crate in an MIT/Apache binary is okay-ish for decompress-only;
a GPL project should NOT link it. Debian/Fedora treat unrar as non-free.
The 7-Zip RAR decoder carries the same restriction ("LGPL + unRAR restriction",
https://www.7-zip.org/license.txt).

Packaging:
- Pure-Rust (`zip`, `sevenz-rust2`): zero system deps, static binary, AppImage trivial.
- `unrar`: static C++ in the binary, no runtime dep; needs g++ at build; license burden.
- `compress-tools`: dynamic libarchive (`libarchive.so`); in AppImage must be bundled
  (and its deps: lzma, zstd, bz2, xml2...); RAR5 support gaps.
- System `7z`/`7zz`/`unrar`/`bsdtar` as subprocess: sane fallback. Zero license burden
  for us (user's own binary), tiny code (`Command::new("7z").args(["x","-o<dir>",...])`),
  and handles all three formats. Caveat: Debian/Ubuntu standalone `7zz`
  (`7zip-standalone`) lacks the RAR plugin (needs `7zip-rar`/`7zip`); Fedora naming
  `7zip`/`p7zip` varies; RAR support must be probed at runtime (`7z i` or try-extract),
  with a clear error "install unrar / 7zip with RAR support". Never trust names from
  archive on the command line (use `--`, cap output dir, sanitise after extract).
- AppImage: bundling `7zz` is allowed (LGPL, keep license text) but unRAR part makes it
  murkier; simply document the dependency.

PES reality: most mods are .rar, so RAR support is mandatory, not optional.

## Minimal recommendation for fl-modman MVP

IMPORT FLOW (copy only MO2's core idea):
1. Extract archive to a temp staging dir (never straight into the game).
2. Auto-detect root: game-defined predicate "dir contains known FL/Sider layout names"
   (case-insensitive: e.g. `livecpk`, `content`, `*.cpk`, known asset dirs); if the
   level fails and has exactly one subdir, descend; ignore readme/image files beside it.
3. Show a plain preview: detected root, file list, destination, conflicts with
   installed mods. Confirm / Cancel.
4. On detection failure: CLI/TUI prompt to pick the root dir from a tree (MO2 "set data
   directory"), stored with the mod. One root per mod; multi-root = reject with message.
5. Normalise case at install, keep an own mod record (file list) in a tiny metadata
   file so enable/disable/uninstall work without any manifest from the author.

EXTRACTION:
- `zip` crate (MIT) + `sevenz-rust2` (Apache-2.0), pure Rust, in-process.
- RAR: shell out to system `7z`/`7zz`/`unrar`/`bsdtar` (probe in that order at runtime;
  friendly error with install hint). No `unrar` crate: avoids C++ build and UnRAR licence.

DO NOT COPY: FOMOD (wizard, flags, conditions), BAIN sub-packages/anneal, Vortex's
installer plugin chain/priority API and mod-type system, MO2's merge/replace/rename
dialog, `compress-tools`/libarchive packaging, game-plugin "fixable" rewrites, any
Windows-style virtual filesystem.

## Sources
- https://github.com/ModOrganizer2/modorganizer-installer_quick (installerquick.cpp)
- MO2 manual-install guides: https://nexusmods.com/starfield/articles/286 ; https://en.uesp.net/wiki/Better_Cities:Mod_Organizer_2_Install
- https://nexus-mods.github.io/vortex-api/2022/04/03/Creating-a-game-extension.html (via search); https://github.com/Nexus-Mods/Vortex/wiki/LEGACY-General-Creating-a-game-extension ; https://forums.nexusmods.com/topic/8548433-help-understanding-mod-types-and-installer-selection/
- https://fomod-docs.readthedocs.io/en/latest/tutorial.html ; https://stepmodifications.org/wiki/Guide:FOMOD/ModuleConfigXML ; https://docs.rs/crate/fomod/latest
- https://wrye-bash.github.io/docs/Wrye%20Bash%20General%20Readme.html (BAIN)
- https://github.com/muja/unrar.rs ; https://www.7-zip.org/license.txt ; https://github.com/libarchive/libarchive
- crates.io API: zip, sevenz-rust2, sevenz-rust, unrar, unrar_sys, compress-tools
- https://www.pesmodding.com (sample mod pages; no manager/manifest found)
