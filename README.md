# Time-Jutsu

> Master your time. Strike with precision.

Desktop productivity app untuk manajemen waktu — fokus untuk developer & desainer. Window kecil (340×520), tinggal di system tray, ringan.

![logo](assets/logo-256.png)

## Fitur

- **Pomodoro** — timer fokus/istirahat dengan arc progress, long-break tiap 4 sesi, auto-pause saat idle (AFK)
- **Timer** — countdown multi-timer, stopwatch (lap + export), dan deadline countdown
- **Time Tracking** — lacak waktu per proyek (rekap harian & mingguan)
- **Scheduler** — auto shutdown / restart / sleep / hibernate (timer internal + Cancel All)
- **Alarm & Reminder** — alarm berulang + Break Alert
- **Dashboard** — ringkasan aktif, statistik mingguan + streak
- **Settings** — nada notifikasi, tema Dark/Light, perilaku tray

## Stack

Rust + [egui](https://github.com/emilk/egui) (`eframe`) · `rodio` (audio) · `chrono` · `serde`. Sudut window membulat via DWM (Windows 11). Di Linux: D-Bus untuk deteksi host tray & idle monitor.

## Build

```sh
cargo run            # debug
cargo build --release
```

### Dependensi sistem (Linux)

Butuh header dev berikut saat build (audio + system tray):

```sh
# Fedora
sudo dnf install -y alsa-lib-devel libxdo-devel libappindicator-gtk3-devel gtk3-devel

# Debian/Ubuntu
sudo apt install -y libasound2-dev libxdo-dev libappindicator3-dev libgtk-3-dev

# Arch
sudo pacman -S --needed alsa-lib xdotool libappindicator-gtk3 gtk3
```

Saat runtime hanya butuh shared library-nya (libasound, libgtk-3, libappindicator/ayatana, libxdo, libdbus) — umum tersedia di desktop Linux.

## Platform

Windows & Linux. Kode OS-specific diisolasi di `src/utils/platform.rs`.

**Catatan Linux (system tray):** GNOME modern secara default **tidak punya system tray**. App mendeteksi ini via D-Bus (`StatusNotifierWatcher`) — bila tak ada host tray, ikon tray tidak dibuat dan app tetap tampil di **dock/taskbar** (minimize → dock, close → keluar). Untuk mengaktifkan tray di GNOME, pasang ekstensi *AppIndicator and KStatusNotifierItem Support*. KDE/XFCE/Cinnamon punya tray secara default. Idle/auto-pause memakai Mutter (GNOME) dengan fallback freedesktop ScreenSaver; single-instance memakai flock.

---

*Binary kecil, RAM rendah, tanpa runtime.*
