#!/usr/bin/env bash
# Bangun AppImage self-contained (bundle stack GTK3) untuk Time-Jutsu.
# Tools (linuxdeploy + plugin gtk + patchelf) diunduh sekali ke cache.
#
#   cargo build --release
#   ./installer/linux/build-appimage.sh
#
# Hasil: target/appimage/Time-Jutsu-<versi>-x86_64.AppImage
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

VERSION="$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"(.*)".*/\1/')"
BIN="$REPO_ROOT/target/release/time-jutsu"
[[ -f "$BIN" ]] || { echo "Binary belum ada — jalankan: cargo build --release" >&2; exit 1; }

TOOLS="$HOME/.cache/appimage-tools"
WORK="$REPO_ROOT/target/appimage"
APPDIR="$WORK/AppDir"
mkdir -p "$TOOLS" "$WORK"

# AppImages butuh FUSE; bila tak ada, mode extract-and-run dipakai.
export APPIMAGE_EXTRACT_AND_RUN=1
export DEPLOY_GTK_VERSION=3
export VERSION
# strip bawaan linuxdeploy gagal pada section .relr.dyn (toolchain baru) dan
# menganggapnya fatal. Binary kita sudah di-strip saat build & lib sistem sudah
# ramping, jadi lewati saja.
export NO_STRIP=1

fetch() { # url dest
    [[ -f "$2" ]] || { echo "  unduh $(basename "$2")"; curl -fsSL -o "$2" "$1"; }
}

echo "== unduh tools (cache: $TOOLS) =="
fetch "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage" "$TOOLS/linuxdeploy"
fetch "https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh" "$TOOLS/linuxdeploy-plugin-gtk.sh"
fetch "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage" "$TOOLS/appimagetool"
# patchelf portabel (tanpa sudo)
if [[ ! -x "$TOOLS/patchelf" ]]; then
    fetch "https://github.com/NixOS/patchelf/releases/download/0.18.0/patchelf-0.18.0-x86_64.tar.gz" "$TOOLS/patchelf.tgz"
    tar -xzf "$TOOLS/patchelf.tgz" -C "$TOOLS" ./bin/patchelf
    mv "$TOOLS/bin/patchelf" "$TOOLS/patchelf"; rmdir "$TOOLS/bin" 2>/dev/null || true
fi
chmod +x "$TOOLS/linuxdeploy" "$TOOLS/linuxdeploy-plugin-gtk.sh" "$TOOLS/patchelf" "$TOOLS/appimagetool"
export PATH="$TOOLS:$PATH"

echo "== susun AppDir =="
rm -rf "$APPDIR"
install -Dm755 "$BIN" "$APPDIR/usr/bin/time-jutsu"
install -Dm644 "$REPO_ROOT/assets/logo-256.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/time-jutsu.png"

echo "== linuxdeploy (+gtk): populate AppDir =="
"$TOOLS/linuxdeploy" \
    --appdir "$APPDIR" \
    -d "$SCRIPT_DIR/time-jutsu.desktop" \
    -i "$REPO_ROOT/assets/logo-256.png" \
    --icon-filename time-jutsu \
    --plugin gtk

# Buang cluster library init/security level-rendah yang ikut terbawa via
# libgio (libtinysparql/libcloudproviders). Bundling-nya bikin crash di
# _dl_init (libcap) — harus dari host. Tiap distro pasti punya versi sistemnya.
echo "== prune deep-system libs =="
for lib in libcap.so.2 libsystemd.so.0 libseccomp.so.2 libselinux.so.1 \
           libmount.so.1 libblkid.so.1 libudev.so.1; do
    rm -fv "$APPDIR/usr/lib/$lib" || true
done

echo "== appimagetool: package =="
FINAL="$WORK/Time-Jutsu-${VERSION}-x86_64.AppImage"
"$TOOLS/appimagetool" "$APPDIR" "$FINAL"
echo
echo "== selesai =="
echo "AppImage: $FINAL"
ls -lh "$FINAL" | awk '{print "ukuran : "$5}'
