use egui::{vec2, Align, Frame, Layout, RichText, ScrollArea, TextEdit, Ui};
use egui_phosphor::regular as icon;

use crate::core::fmt_hms;
use crate::core::tracking::Tracker;
use crate::ui::theme;
use std::time::Duration;

/// Kembalikan true bila ada perubahan (untuk disimpan).
pub fn render(ui: &mut Ui, tracker: &mut Tracker) -> bool {
    let mut changed = false;

    ui.label(RichText::new("Time Tracking").color(theme::text()).strong().size(15.0));
    ui.label(RichText::new("Lacak waktu per proyek").color(theme::muted()).size(11.0));
    ui.add_space(10.0);

    // ── Tambah proyek ──────────────────────────────────────────────
    ui.horizontal(|ui| {
        let resp = ui.add(
            TextEdit::singleline(&mut tracker.input_name)
                .hint_text("Nama proyek")
                .desired_width(f32::INFINITY),
        );
        let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let add = egui::Button::new(RichText::new(icon::PLUS).color(theme::on_accent()).strong())
                .fill(theme::ACCENT)
                .corner_radius(6.0)
                .min_size(vec2(30.0, 24.0));
            if (ui.add(add).clicked() || enter) && !tracker.input_name.trim().is_empty() {
                tracker.add();
                changed = true;
            }
        });
    });
    ui.add_space(12.0);

    if tracker.projects.is_empty() {
        ui.add_space(16.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Belum ada proyek").color(theme::muted()).size(12.0));
        });
        return changed;
    }

    // ── Daftar proyek ──────────────────────────────────────────────
    let mut toggle = None;
    let mut remove = None;
    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for i in 0..tracker.projects.len() {
            let active = tracker.is_active(i);
            let name = tracker.projects[i].name.clone();
            let today = fmt_hms(Duration::from_secs(tracker.live_today(i)));
            let week = fmt_hms(Duration::from_secs(tracker.projects[i].week_secs()));

            Frame::new()
                .fill(if active { theme::surface_hi() } else { theme::surface() })
                .corner_radius(8.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        // tombol start/stop
                        let (gl, col) = if active {
                            (icon::PAUSE, theme::ACCENT)
                        } else {
                            (icon::PLAY, theme::muted())
                        };
                        let btn = egui::Button::new(RichText::new(gl).color(col).size(16.0))
                            .frame(false)
                            .min_size(vec2(24.0, 24.0));
                        if ui.add(btn).clicked() {
                            toggle = Some(i);
                        }
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&name).color(theme::text()).size(13.0));
                            ui.label(
                                RichText::new(format!("Minggu ini: {week}"))
                                    .color(theme::muted())
                                    .size(10.0),
                            );
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_button(ui, icon::TRASH).clicked() {
                                remove = Some(i);
                            }
                            let tcol = if active { theme::ACCENT } else { theme::text() };
                            ui.label(RichText::new(today).color(tcol).monospace().size(15.0));
                        });
                    });
                });
            ui.add_space(6.0);
        }
    });

    if let Some(i) = toggle {
        tracker.toggle(i);
        changed = true;
    }
    if let Some(i) = remove {
        tracker.remove(i);
        changed = true;
    }
    changed
}

fn icon_button(ui: &mut Ui, glyph: &str) -> egui::Response {
    let btn = egui::Button::new(RichText::new(glyph).color(theme::muted()).size(14.0))
        .frame(false)
        .min_size(vec2(22.0, 22.0));
    ui.add(btn)
}
