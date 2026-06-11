#!/usr/bin/env bash
# Pasang Time-Jutsu untuk user saat ini (tanpa sudo).
# Menyalin binary, ikon (hicolor), dan .desktop agar logo muncul di dock/menu.
#
#   cargo build --release
#   ./installer/linux/install.sh
#
# Uninstall: ./installer/linux/install.sh --uninstall
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

BIN_DIR="$HOME/.local/bin"
ICON_BASE="$HOME/.local/share/icons/hicolor"
APP_DIR="$HOME/.local/share/applications"

refresh_caches() {
  command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -f -t "$ICON_BASE" >/dev/null 2>&1 || true
  command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APP_DIR" >/dev/null 2>&1 || true
}

if [[ "${1:-}" == "--uninstall" ]]; then
  rm -f "$BIN_DIR/time-jutsu" \
        "$ICON_BASE/256x256/apps/time-jutsu.png" \
        "$ICON_BASE/64x64/apps/time-jutsu.png" \
        "$ICON_BASE/32x32/apps/time-jutsu.png" \
        "$APP_DIR/time-jutsu.desktop"
  refresh_caches
  echo "Time-Jutsu dihapus dari ~/.local."
  exit 0
fi

BIN_SRC="$REPO_ROOT/target/release/time-jutsu"
if [[ ! -f "$BIN_SRC" ]]; then
  echo "Binary belum ada: $BIN_SRC" >&2
  echo "Jalankan dulu: cargo build --release" >&2
  exit 1
fi

install -Dm755 "$BIN_SRC" "$BIN_DIR/time-jutsu"
install -Dm644 "$REPO_ROOT/assets/logo-256.png" "$ICON_BASE/256x256/apps/time-jutsu.png"
install -Dm644 "$REPO_ROOT/assets/logo-64.png"  "$ICON_BASE/64x64/apps/time-jutsu.png"
install -Dm644 "$REPO_ROOT/assets/logo-32.png"  "$ICON_BASE/32x32/apps/time-jutsu.png"
install -Dm644 "$SCRIPT_DIR/time-jutsu.desktop" "$APP_DIR/time-jutsu.desktop"
refresh_caches

echo "Time-Jutsu terpasang."
case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) echo "Catatan: tambahkan $BIN_DIR ke PATH agar bisa jalan via 'time-jutsu'." ;;
esac
echo "Cari 'Time-Jutsu' di menu aplikasi, atau jalankan: time-jutsu"
