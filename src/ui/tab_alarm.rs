use egui::{Align, ComboBox, Layout, RichText, ScrollArea, Ui};
use egui_phosphor::regular as icon;

use crate::core::alarm::{AlarmState, Repeat};
use crate::i18n::t;
use crate::ui::theme;

/// Render Tab Alarm. Kembalikan true bila ada perubahan (untuk disimpan).
pub fn render(ui: &mut Ui, alarm: &mut AlarmState) -> bool {
    let mut changed = false;

    ui.label(RichText::new("Alarm").color(theme::text()).strong().size(15.0));
    ui.label(
        RichText::new(t("Alarm, Pengingat & Istirahat", "Alarm, Reminder & Break Alert"))
            .color(theme::muted())
            .size(11.0),
    );
    ui.add_space(10.0);

    // ── Tambah alarm ───────────────────────────────────────────────
    ui.add(
        egui::TextEdit::singleline(&mut alarm.input_label)
            .hint_text(t("Label / pesan", "Label / message"))
            .desired_width(f32::INFINITY),
    );
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut alarm.input_hour).range(0..=23).suffix("h"));
        ui.label(RichText::new(":").color(theme::muted()));
        ui.add(egui::DragValue::new(&mut alarm.input_minute).range(0..=59).suffix("m"));
        ComboBox::from_id_salt("alarm_repeat")
            .selected_text(alarm.input_repeat.label())
            .show_ui(ui, |ui| {
                for r in Repeat::ALL {
                    ui.selectable_value(&mut alarm.input_repeat, r, r.label());
                }
            });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let add = egui::Button::new(
                RichText::new(format!("{}  {}", icon::PLUS, t("Tambah", "Add")))
                    .color(theme::on_accent())
                    .strong(),
            )
            .fill(theme::ACCENT)
            .corner_radius(6.0);
            if ui.add(add).clicked() {
                alarm.add();
                changed = true;
            }
        });
    });

    ui.add_space(10.0);

    // ── Daftar alarm ───────────────────────────────────────────────
    let mut remove = None;
    if alarm.alarms.is_empty() {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t("Belum ada alarm", "No alarms yet")).color(theme::muted()).size(12.0));
        });
    } else {
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(220.0)
            .show(ui, |ui| {
                for i in 0..alarm.alarms.len() {
                    let a = &mut alarm.alarms[i];
                    let time = a.time_label();
                    let rep = a.repeat.label();
                    let label = a.label.clone();
                    let time_color = if a.enabled { theme::text() } else { theme::muted() };

                    egui::Frame::new()
                        .fill(theme::surface())
                        .corner_radius(6.0)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if ui.checkbox(&mut a.enabled, "").changed() {
                                    changed = true;
                                }
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(time)
                                            .color(time_color)
                                            .monospace()
                                            .size(16.0),
                                    );
                                    ui.label(
                                        RichText::new(format!("{label}  ·  {rep}"))
                                            .color(theme::muted())
                                            .size(11.0),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let x = egui::Button::new(
                                        RichText::new(icon::TRASH).color(theme::muted()).size(14.0),
                                    )
                                    .frame(false);
                                    if ui.add(x).clicked() {
                                        remove = Some(i);
                                    }
                                });
                            });
                        });
                    ui.add_space(6.0);
                }
            });
    }
    if let Some(i) = remove {
        alarm.alarms.remove(i);
        changed = true;
    }

    ui.add_space(10.0);

    // ── Break Alert ────────────────────────────────────────────────
    egui::Frame::new()
        .fill(theme::surface())
        .corner_radius(6.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .checkbox(
                        &mut alarm.break_alert.enabled,
                        RichText::new(t("Pengingat Istirahat", "Break Alert")).color(theme::text()).strong(),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(t("Ingatkan istirahat tiap", "Remind to rest every"))
                        .color(theme::muted())
                        .size(12.0),
                );
                if ui
                    .add(
                        egui::DragValue::new(&mut alarm.break_alert.interval_minutes)
                            .range(1..=240)
                            .suffix(t(" menit", " min")),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
        });

    changed
}
