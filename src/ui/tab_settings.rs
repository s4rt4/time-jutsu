use egui::{vec2, Frame, RichText, Ui};
use egui_phosphor::regular as icon;

use crate::i18n::{self, t, Lang};
use crate::ui::theme;
use crate::ui::theme::ThemeMode;
use crate::utils::sound::AlarmSound;

pub struct SettingsResult {
    pub changed: bool,
    pub theme_changed: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn render(
    ui: &mut Ui,
    sound: &mut AlarmSound,
    theme_mode: &mut ThemeMode,
    lang: &mut Lang,
    tray_on_close: &mut bool,
    tray_on_minimize: &mut bool,
    long_break_minutes: &mut u32,
    idle_autopause: &mut bool,
) -> SettingsResult {
    let mut changed = false;
    let mut theme_changed = false;

    ui.label(
        RichText::new(t("Pengaturan", "Settings"))
            .color(theme::text())
            .strong()
            .size(15.0),
    );
    ui.add_space(12.0);

    // ── Bahasa ─────────────────────────────────────────────────────
    section(ui, t("BAHASA", "LANGUAGE"));
    card(ui, |ui| {
        ui.horizontal(|ui| {
            if lang_btn(ui, lang, Lang::Id, "Indonesia") {
                changed = true;
            }
            if lang_btn(ui, lang, Lang::En, "English") {
                changed = true;
            }
        });
    });

    ui.add_space(12.0);

    // ── Nada notifikasi ────────────────────────────────────────────
    section(ui, t("NADA NOTIFIKASI", "NOTIFICATION SOUND"));
    card(ui, |ui| {
        if crate::ui::sound_picker(ui, sound) {
            changed = true;
        }
        ui.add_space(4.0);
        ui.label(
            RichText::new(t(
                "Dipakai untuk Pomodoro, Timer, Alarm & Break Alert.",
                "Used for Pomodoro, Timer, Alarm & Break Alert.",
            ))
            .color(theme::muted())
            .size(11.0),
        );
    });

    ui.add_space(12.0);

    // ── Tema ───────────────────────────────────────────────────────
    section(ui, t("TEMA", "THEME"));
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
            ui.label(
                RichText::new(t("Istirahat panjang tiap 4 sesi", "Long break every 4 sessions"))
                    .color(theme::text())
                    .size(12.0),
            );
            if ui
                .add(egui::DragValue::new(long_break_minutes).range(1..=60).suffix(t(" mnt", " min")))
                .changed()
            {
                changed = true;
            }
        });
        ui.add_space(4.0);
        if ui
            .checkbox(
                idle_autopause,
                RichText::new(t("Auto-pause saat idle (AFK)", "Auto-pause when idle (AFK)"))
                    .color(theme::text()),
            )
            .changed()
        {
            changed = true;
        }
    });

    ui.add_space(12.0);

    // ── Perilaku ───────────────────────────────────────────────────
    section(ui, t("PERILAKU", "BEHAVIOR"));
    card(ui, |ui| {
        if ui
            .checkbox(
                tray_on_close,
                RichText::new(t(
                    "Sembunyikan ke system tray saat ditutup",
                    "Hide to system tray on close",
                ))
                .color(theme::text()),
            )
            .changed()
        {
            changed = true;
        }
        ui.add_space(4.0);
        if ui
            .checkbox(
                tray_on_minimize,
                RichText::new(t(
                    "Sembunyikan ke system tray saat di-minimize",
                    "Hide to system tray on minimize",
                ))
                .color(theme::text()),
            )
            .changed()
        {
            changed = true;
        }
        ui.add_space(4.0);
        ui.label(
            RichText::new(t(
                "Jika dimatikan: tombol tutup keluar dari app, minimize ke taskbar.",
                "If off: close button exits the app, minimize goes to taskbar.",
            ))
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

fn lang_btn(ui: &mut Ui, current: &mut Lang, l: Lang, label: &str) -> bool {
    let active = *current == l;
    let (fg, bg) = if active {
        (theme::on_accent(), theme::ACCENT)
    } else {
        (theme::text(), theme::surface_hi())
    };
    let btn = egui::Button::new(RichText::new(label).color(fg))
        .fill(bg)
        .corner_radius(6.0)
        .min_size(vec2(100.0, 28.0));
    if ui.add(btn).clicked() && *current != l {
        *current = l;
        i18n::set_lang(l); // langsung berlaku
        return true;
    }
    false
}
