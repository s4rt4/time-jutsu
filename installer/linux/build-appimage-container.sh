#!/usr/bin/env bash
# Build AppImage PORTABEL di dalam container Debian (glibc lama).
#
# Kenapa container: build di host bleeding-edge (mis. Fedora 43) menghasilkan
# AppImage yang crash & kurang portabel. Base glibc lama = jalan di lebih banyak
# distro (aturan emas AppImage: build di distro setua mungkin yang ingin didukung).
#
#   ./installer/linux/build-appimage-container.sh
#
# Hasil: target/appimage/Time-Jutsu-<versi>-x86_64.AppImage
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUT="$REPO_ROOT/target/appimage"
IMAGE="docker.io/library/rust:1-bullseye"   # Debian 11, glibc 2.31
mkdir -p "$OUT"

echo "== build AppImage di $IMAGE (rootless podman) =="
podman run --rm \
    -v "$REPO_ROOT":/src:ro,z \
    -v "$OUT":/out:z \
    -e APPIMAGE_EXTRACT_AND_RUN=1 \
    "$IMAGE" bash -euc '
        export DEBIAN_FRONTEND=noninteractive
        echo "-- apt deps --"
        apt-get update -qq
        apt-get install -y --no-install-recommends \
            pkg-config file ca-certificates curl patchelf \
            libasound2-dev libgtk-3-dev libxdo-dev libappindicator3-dev \
            libgdk-pixbuf2.0-dev libglib2.0-bin librsvg2-common \
            desktop-file-utils shared-mime-info >/dev/null
        echo "-- salin sumber (tanpa target/) --"
        mkdir -p /work && cd /work
        cp -a /src/Cargo.toml /src/Cargo.lock /src/build.rs /src/src /src/assets /src/installer /work/
        echo "-- cargo build --release --"
        cargo build --release
        echo "-- bundling (linuxdeploy + gtk) --"
        ./installer/linux/build-appimage.sh
        cp -v target/appimage/*.AppImage /out/
    '

echo "== selesai =="
ls -lh "$OUT"/*.AppImage
