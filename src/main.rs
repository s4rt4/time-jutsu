// Time-Jutsu — entry point.
// Sembunyikan console window di Windows pada build release (GUI app).
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod core;
mod i18n;
mod ui;
mod utils;

use std::sync::Arc;

use app::TimeJutsuApp;

const WINDOW_W: f32 = 340.0;
const WINDOW_H: f32 = 520.0;

/// Logo brand untuk window/taskbar icon (PNG transparan di-embed).
fn load_window_icon() -> egui::IconData {
    let bytes = include_bytes!("../assets/logo-256.png");
    let img = image::load_from_memory(bytes)
        .expect("decode logo png")
        .to_rgba8();
    let (width, height) = img.dimensions();
    egui::IconData {
        rgba: img.into_raw(),
        width,
        height,
    }
}

/// Single-instance: jika sudah ada instance jalan, tampilkan window-nya lalu
/// minta instance baru keluar. Cegah app dobel saat shortcut diklik lagi.
#[cfg(target_os = "windows")]
fn should_exit_duplicate() -> bool {
    use windows::core::w;
    use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows::Win32::System::Threading::CreateMutexW;
    unsafe {
        let handle = CreateMutexW(None, true, w!("TimeJutsu-SingleInstance-Mutex"));
        if GetLastError() == ERROR_ALREADY_EXISTS {
            // instance lain sudah ada → munculkan window-nya, lalu keluar
            utils::platform::show_window("Time-Jutsu");
            return true;
        }
        // HANDLE bertahan sampai proses keluar (tak ada Drop yang menutupnya);
        // mutex tetap dipegang → guard single-instance aktif.
        let _ = handle;
        false
    }
}

#[cfg(not(target_os = "windows"))]
fn should_exit_duplicate() -> bool {
    false
}

fn main() -> eframe::Result<()> {
    if should_exit_duplicate() {
        return Ok(());
    }

    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([WINDOW_W, WINDOW_H])
        .with_min_inner_size([WINDOW_W, WINDOW_H])
        .with_max_inner_size([WINDOW_W, WINDOW_H])
        .with_resizable(false)
        .with_decorations(false) // frameless — titlebar digambar sendiri
        .with_icon(Arc::new(load_window_icon()))
        .with_title("Time-Jutsu");

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Time-Jutsu",
        options,
        Box::new(|cc| Ok(Box::new(TimeJutsuApp::new(cc)))),
    )
}
