use egui::{vec2, Frame, RichText, Ui};
use egui_phosphor::regular as icon;

use crate::ui::theme;
use crate::ui::theme::ThemeMode;
use crate::utils::sound::AlarmSound;

pub struct SettingsResult {
    pub changed: bool,
    pub theme_changed: bool,
}

pub fn render(
    ui: &mut Ui,
    sound: &mut AlarmSound,
    theme_mode: &mut ThemeMode,
    tray_on_close: &mut bool,
    tray_on_minimize: &mut bool,
    long_break_minutes: &mut u32,
    idle_autopause: &mut bool,
) -> SettingsResult {
    let mut changed = false;
    let mut theme_changed = false;

    ui.label(RichText::new("Settings").color(theme::text()).strong().size(15.0));
    ui.add_space(12.0);

    // ── Nada notifikasi ────────────────────────────────────────────
    section(ui, "NADA NOTIFIKASI");
    card(ui, |ui| {
        if crate::ui::sound_picker(ui, sound) {
            changed = true;
        }
        ui.add_space(4.0);
        ui.label(
            RichText::new("Dipakai untuk Pomodoro, Timer, Alarm & Break Alert.")
                .color(theme::muted())
                .size(11.0),
        );
    });

    ui.add_space(12.0);

    // ── Tema ───────────────────────────────────────────────────────
    section(ui, "TEMA");
    card(ui, |ui| {
        ui.horizontal(|ui| {
            if theme_btn(ui, theme_mode, ThemeMode::Dark, icon::MOON, "Dark") {
                theme_changed = true;
                changed = true;
            }
            if theme_btn(ui, theme_mode, ThemeMode::Light, icon::SUN, "Light") {
                theme_changed = true;
                changed = true;
            }
        });
    });

    ui.add_space(12.0);

    // ── Pomodoro ───────────────────────────────────────────────────
    section(ui, "POMODORO");
    card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Istirahat panjang tiap 4 sesi").color(theme::text()).size(12.0));
            if ui
                .add(egui::DragValue::new(long_break_minutes).range(1..=60).suffix(" mnt"))
                .changed()
            {
                changed = true;
            }
        });
        ui.add_space(4.0);
        if ui
            .checkbox(
                idle_autopause,
                RichText::new("Auto-pause saat idle (AFK)").color(theme::text()),
            )
            .changed()
        {
            changed = true;
        }
    });

    ui.add_space(12.0);

    // ── Perilaku ───────────────────────────────────────────────────
    section(ui, "PERILAKU");
    card(ui, |ui| {
        if ui
            .checkbox(
                tray_on_close,
                RichText::new("Sembunyikan ke system tray saat ditutup").color(theme::text()),
            )
            .changed()
        {
            changed = true;
        }
        ui.add_space(4.0);
        if ui
            .checkbox(
                tray_on_minimize,
                RichText::new("Sembunyikan ke system tray saat di-minimize").color(theme::text()),
            )
            .changed()
        {
            changed = true;
        }
        ui.add_space(4.0);
        ui.label(
            RichText::new("Jika dimatikan: tombol tutup keluar dari app, minimize ke taskbar.")
                .color(theme::muted())
                .size(11.0),
        );
    });

    SettingsResult {
        changed,
        theme_changed,
    }
}

fn section(ui: &mut Ui, title: &str) {
    ui.label(RichText::new(title).color(theme::muted()).size(10.0));
    ui.add_space(4.0);
}

/// Kartu SURFACE selebar konten (modern, rounded).
fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) {
    Frame::new()
        .fill(theme::surface())
        .corner_radius(8.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui);
        });
}

fn theme_btn(ui: &mut Ui, current: &mut ThemeMode, mode: ThemeMode, glyph: &str, label: &str) -> bool {
    let active = *current == mode;
    let (fg, bg) = if active {
        (theme::on_accent(), theme::ACCENT)
    } else {
        (theme::text(), theme::surface_hi())
    };
    let btn = egui::Button::new(RichText::new(format!("{glyph}  {label}")).color(fg))
        .fill(bg)
        .corner_radius(6.0)
        .min_size(vec2(96.0, 28.0));
    if ui.add(btn).clicked() && *current != mode {
        *current = mode;
        return true;
    }
    false
}
