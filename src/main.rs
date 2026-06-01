// Time-Jutsu — entry point.
// Sembunyikan console window di Windows pada build release (GUI app).
#![cfg_attr(all(target_os = "windows", not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod core;
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

fn main() -> eframe::Result<()> {
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
