#!/usr/bin/env bash
# Builds dist/fl-modman-<version>-x86_64.AppImage with the GUI and the CLI.
#
# Needs: cargo, curl, sha256sum. appimagetool (https://github.com/AppImage/appimagetool) is
# downloaded at the pinned release below into ${XDG_CACHE_HOME:-~/.cache}/fl-modman/ and
# checked against its SHA-256 on every run (no root needed). APPIMAGETOOL=<path> uses another
# copy instead, unchecked.
# Runtime deps (OpenGL, X11/Wayland libs) come from the host system and are not bundled.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

version="$(cargo pkgid -p fl-gui | sed 's/.*[#@]//')"
appdir="$root/target/appimage/AppDir"
out="$root/dist/fl-modman-$version-x86_64.AppImage"

cargo build --release --locked -p fl-gui -p fl-cli

rm -rf "$appdir"
install -Dm755 target/release/fl-modman-gui "$appdir/usr/bin/fl-modman-gui"
install -Dm755 target/release/fl-modman "$appdir/usr/bin/fl-modman"
install -Dm755 packaging/AppRun "$appdir/AppRun"
install -Dm644 packaging/fl-modman.desktop "$appdir/fl-modman.desktop"
install -Dm644 packaging/fl-modman.png "$appdir/fl-modman.png"
install -Dm644 LICENSE "$appdir/usr/share/doc/fl-modman/LICENSE"

tool_version="1.9.1"
tool_sha256="ed4ce84f0d9caff66f50bcca6ff6f35aae54ce8135408b3fa33abfc3cb384eb0"
tool="${APPIMAGETOOL:-}"
if [ -z "$tool" ]; then
    tool="${XDG_CACHE_HOME:-$HOME/.cache}/fl-modman/appimagetool-$tool_version-x86_64.AppImage"
    if [ ! -f "$tool" ]; then
        mkdir -p "$(dirname "$tool")"
        curl -fL -o "$tool.part" \
            "https://github.com/AppImage/appimagetool/releases/download/$tool_version/appimagetool-x86_64.AppImage"
        mv "$tool.part" "$tool"
    fi
    # Checked on every run, so a corrupted or replaced cached copy is never executed.
    if ! echo "$tool_sha256  $tool" | sha256sum -c --quiet -; then
        echo "appimagetool checksum mismatch; delete $tool and retry" >&2
        exit 1
    fi
    chmod +x "$tool"
fi

mkdir -p dist
# --appimage-extract-and-run lets the tool run without FUSE (containers, CI).
ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "$tool" "$appdir" "$out"
(cd dist && sha256sum "$(basename "$out")" > "$(basename "$out").sha256")
echo "$out"
