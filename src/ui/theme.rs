//! Tema & palet. Warna bergantung mode (Dark/Light) lewat state global,
//! diakses via fungsi (mis. `theme::bg()`). Accent coral & teks-di-atas-coral
//! tetap sama di kedua tema; hanya bg/surface/text/muted yang berganti.

use std::sync::atomic::{AtomicU8, Ordering};

use egui::{Color32, Context, Stroke};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

impl ThemeMode {
    pub fn is_dark(self) -> bool {
        matches!(self, ThemeMode::Dark)
    }
}

static THEME: AtomicU8 = AtomicU8::new(0); // 0 = dark, 1 = light

pub fn set_theme(t: ThemeMode) {
    THEME.store(if t.is_dark() { 0 } else { 1 }, Ordering::Relaxed);
}

fn dark() -> bool {
    THEME.load(Ordering::Relaxed) == 0
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

// ── Brand (sama di kedua tema) ──────────────────────────────────────
pub const ACCENT: Color32 = rgb(0xf9, 0x6a, 0x5c); // coral
pub const ACCENT_2: Color32 = rgb(0xff, 0x9e, 0x91); // coral terang
const ON_ACCENT: Color32 = rgb(0x0c, 0x1c, 0x2c); // teks/icon di atas coral

pub fn on_accent() -> Color32 {
    ON_ACCENT
}

// ── Warna per-tema ──────────────────────────────────────────────────
pub fn bg() -> Color32 {
    if dark() {
        rgb(0x0c, 0x1c, 0x2c)
    } else {
        rgb(0xe9, 0xee, 0xf3)
    }
}
pub fn surface() -> Color32 {
    if dark() {
        rgb(0x13, 0x2b, 0x42)
    } else {
        rgb(0xff, 0xff, 0xff)
    }
}
pub fn surface_hi() -> Color32 {
    if dark() {
        rgb(0x1d, 0x3a, 0x55)
    } else {
        rgb(0xd7, 0xdf, 0xe8)
    }
}
pub fn text() -> Color32 {
    if dark() {
        rgb(0xff, 0xff, 0xff)
    } else {
        rgb(0x12, 0x28, 0x3c)
    }
}
pub fn muted() -> Color32 {
    if dark() {
        rgb(0x6e, 0x86, 0x9b)
    } else {
        rgb(0x5a, 0x71, 0x87)
    }
}

/// Daftarkan icon font Phosphor (Regular) sebagai fallback.
pub fn setup_fonts(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}

/// Terapkan visuals egui sesuai tema aktif. Panggil ulang saat tema berganti.
pub fn apply(ctx: &Context) {
    let mut visuals = if dark() {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = bg();
    visuals.window_fill = bg();
    visuals.extreme_bg_color = surface();
    visuals.override_text_color = Some(text());
    visuals.selection.bg_fill = ACCENT.linear_multiply(0.6);

    for w in [
        &mut visuals.widgets.noninteractive,
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.fg_stroke.color = text();
    }
    visuals.widgets.inactive.bg_fill = surface();
    visuals.widgets.inactive.weak_bg_fill = surface();
    visuals.widgets.hovered.bg_fill = surface_hi();
    visuals.widgets.hovered.weak_bg_fill = surface_hi();
    visuals.widgets.active.bg_fill = ACCENT.linear_multiply(0.5);

    visuals.window_stroke = Stroke::NONE;
    visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;

    ctx.set_visuals(visuals);
}
