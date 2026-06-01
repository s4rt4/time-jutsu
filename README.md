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

Rust + [egui](https://github.com/emilk/egui) (`eframe`) · `rodio` (audio) · `chrono` · `serde`. Sudut window membulat via DWM (Windows 11).

## Build

```sh
cargo run            # debug
cargo build --release
```

## Platform

Windows (utama). Kode OS-specific diisolasi di `src/utils/platform.rs` agar mudah diperluas ke Linux.

---

*Binary kecil, RAM rendah, tanpa runtime.*
