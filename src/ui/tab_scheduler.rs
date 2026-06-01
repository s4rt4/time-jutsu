use egui::{vec2, ComboBox, RichText, Ui};
use egui_phosphor::regular as icon;

use crate::core::fmt_hms;
use crate::core::scheduler::Scheduler;
use crate::ui::theme;
use crate::utils::platform::SystemAction;

pub fn render(ui: &mut Ui, sched: &mut Scheduler) {
    ui.label(
        RichText::new("Scheduler")
            .color(theme::text())
            .strong()
            .size(15.0),
    );
    ui.label(
        RichText::new("Auto shutdown / sleep / restart")
            .color(theme::muted())
            .size(11.0),
    );
    ui.add_space(12.0);

    let armed = sched.is_armed();

    // ── Aksi ───────────────────────────────────────────────────────
    ui.add_enabled_ui(!armed, |ui| {
        ui.label(RichText::new("AKSI").color(theme::muted()).size(10.0));
        ui.add_space(4.0);
        ComboBox::from_id_salt("sched_action")
            .selected_text(sched.action.label())
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for a in SystemAction::ALL {
                    ui.selectable_value(&mut sched.action, a, a.label());
                }
            });

        ui.add_space(10.0);

        // ── Waktu ──────────────────────────────────────────────────
        ui.label(RichText::new("WAKTU").color(theme::muted()).size(10.0));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            mode_button(ui, sched, false, "Setelah");
            mode_button(ui, sched, true, "Pada jam");
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if sched.use_at_time {
                ui.add(egui::DragValue::new(&mut sched.at_hour).range(0..=23).suffix("h"));
                ui.label(RichText::new(":").color(theme::muted()));
                ui.add(egui::DragValue::new(&mut sched.at_minute).range(0..=59).suffix("m"));
            } else {
                ui.add(
                    egui::DragValue::new(&mut sched.after_minutes)
                        .range(1..=1440)
                        .suffix(" menit"),
                );
            }
        });
    });

    ui.add_space(12.0);

    // ── Preview ────────────────────────────────────────────────────
    ui.label(
        RichText::new(sched.preview())
            .color(theme::ACCENT_2)
            .size(12.0)
            .italics(),
    );

    ui.add_space(10.0);

    // ── Status armed + countdown ───────────────────────────────────
    if let Some(rem) = sched.remaining() {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Tersisa").color(theme::muted()).size(11.0));
            ui.label(
                RichText::new(fmt_hms(rem))
                    .color(theme::ACCENT)
                    .monospace()
                    .size(28.0),
            );
        });
        ui.add_space(10.0);
    }

    // ── Tombol aksi ────────────────────────────────────────────────
    ui.vertical_centered(|ui| {
        if !armed {
            let set = egui::Button::new(
                RichText::new(format!("{}  Set Jadwal", icon::ALARM))
                    .color(theme::on_accent())
                    .strong(),
            )
            .fill(theme::ACCENT)
            .corner_radius(6.0)
            .min_size(vec2(160.0, 34.0));
            if ui.add(set).clicked() {
                sched.arm();
            }
        } else {
            let cancel = egui::Button::new(
                RichText::new(format!("{}  Cancel All", icon::X_CIRCLE))
                    .color(theme::text())
                    .strong(),
            )
            .fill(theme::surface_hi())
            .corner_radius(6.0)
            .min_size(vec2(160.0, 34.0));
            if ui.add(cancel).clicked() {
                sched.cancel();
            }
        }
    });
}

fn mode_button(ui: &mut Ui, sched: &mut Scheduler, at_time: bool, text: &str) {
    let active = sched.use_at_time == at_time;
    let (fg, bg) = if active {
        (theme::on_accent(), theme::ACCENT)
    } else {
        (theme::text(), theme::surface_hi())
    };
    let btn = egui::Button::new(RichText::new(text).color(fg))
        .fill(bg)
        .corner_radius(6.0)
        .min_size(vec2(90.0, 26.0));
    if ui.add(btn).clicked() {
        sched.use_at_time = at_time;
    }
}
