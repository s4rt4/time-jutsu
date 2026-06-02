use egui::{vec2, Align, Layout, RichText, ScrollArea, Sense, TextEdit, Ui};
use egui_phosphor::regular as icon;

use crate::core::countdown::{TimerMode, TimerState};
use crate::core::fmt_hms;
use crate::core::stopwatch::{fmt_sw, Stopwatch};
use crate::i18n::t;
use crate::ui::theme;

/// Kembalikan true bila ada perubahan deadline (untuk disimpan).
pub fn render(ui: &mut Ui, timer: &mut TimerState) -> bool {
    ui.label(RichText::new("Timer").color(theme::text()).strong().size(15.0));
    ui.add_space(8.0);

    // ── Mode: Countdown / Stopwatch / Deadline ─────────────────────
    ui.horizontal(|ui| {
        mode_button(ui, timer, TimerMode::Countdown, t("Hitung Mundur", "Countdown"));
        mode_button(ui, timer, TimerMode::Stopwatch, "Stopwatch");
        mode_button(ui, timer, TimerMode::Deadline, t("Tenggat", "Deadline"));
    });
    ui.add_space(12.0);

    match timer.mode {
        TimerMode::Countdown => {
            render_countdown(ui, timer);
            false
        }
        TimerMode::Stopwatch => {
            render_stopwatch(ui, &mut timer.stopwatch);
            false
        }
        TimerMode::Deadline => render_deadline(ui, timer),
    }
}

// ════════════════════════════ Deadline ════════════════════════════

fn render_deadline(ui: &mut Ui, timer: &mut TimerState) -> bool {
    let mut changed = false;

    ui.add(
        TextEdit::singleline(&mut timer.dl_label)
            .hint_text(t("Label deadline", "Deadline label"))
            .desired_width(f32::INFINITY),
    );
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(t("Dalam", "In")).color(theme::muted()).size(12.0));
        ui.add(egui::DragValue::new(&mut timer.dl_days).range(0..=365).suffix(t(" hari", " days")));
        ui.add(egui::DragValue::new(&mut timer.dl_hours).range(0..=23).suffix(t(" jam", " hrs")));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let add = egui::Button::new(
                RichText::new(format!("{}  Set", icon::PLUS)).color(theme::on_accent()).strong(),
            )
            .fill(theme::ACCENT)
            .corner_radius(6.0);
            let valid = timer.dl_days > 0 || timer.dl_hours > 0;
            if ui.add_enabled(valid, add).clicked() {
                timer.add_deadline();
                changed = true;
            }
        });
    });
    ui.add_space(12.0);

    if timer.deadlines.is_empty() {
        ui.add_space(16.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t("Belum ada deadline", "No deadlines yet")).color(theme::muted()).size(12.0));
        });
        return changed;
    }

    let mut remove = None;
    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for i in 0..timer.deadlines.len() {
            let d = &timer.deadlines[i];
            let past = d.is_past();
            let rem = d.remaining_text();
            let label = d.label.clone();
            let target = d.target_text();
            let rem_color = if past { theme::muted() } else { theme::ACCENT_2 };

            egui::Frame::new()
                .fill(theme::surface())
                .corner_radius(6.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(label).color(theme::text()).size(13.0));
                            ui.label(RichText::new(target).color(theme::muted()).size(10.0));
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_button(ui, icon::X).clicked() {
                                remove = Some(i);
                            }
                            ui.label(RichText::new(rem).color(rem_color).strong().size(13.0));
                        });
                    });
                });
            ui.add_space(6.0);
        }
    });
    if let Some(i) = remove {
        timer.deadlines.remove(i);
        changed = true;
    }
    changed
}

// ════════════════════════════ Countdown ════════════════════════════

fn render_countdown(ui: &mut Ui, timer: &mut TimerState) {
    ui.add(
        TextEdit::singleline(&mut timer.input_label)
            .hint_text(t("Label (opsional)", "Label (optional)"))
            .desired_width(f32::INFINITY),
    );
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut timer.input_h).range(0..=99).suffix("h"));
        ui.add(egui::DragValue::new(&mut timer.input_m).range(0..=59).suffix("m"));
        ui.add(egui::DragValue::new(&mut timer.input_s).range(0..=59).suffix("s"));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let add = egui::Button::new(
                RichText::new(format!("{}  {}", icon::PLUS, t("Tambah", "Add")))
                    .color(theme::on_accent())
                    .strong(),
            )
            .fill(theme::ACCENT)
            .corner_radius(6.0);
            if ui.add_enabled(timer.input_total_secs() > 0, add).clicked() {
                timer.add();
            }
        });
    });
    ui.add_space(12.0);

    if timer.countdowns.is_empty() {
        ui.add_space(16.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new(t("Belum ada countdown", "No countdowns yet")).color(theme::muted()).size(12.0));
        });
        return;
    }

    let mut toggle = None;
    let mut remove = None;
    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for i in 0..timer.countdowns.len() {
            let c = &timer.countdowns[i];
            let (running, finished) = (c.is_running(), c.finished);
            let rem = fmt_hms(c.remaining());
            let label = c.label.clone();
            let frac = c.progress();
            let glyph = if finished {
                icon::CHECK_CIRCLE
            } else if running {
                icon::PAUSE
            } else {
                icon::PLAY
            };
            let time_color = if finished { theme::ACCENT_2 } else { theme::text() };

            egui::Frame::new()
                .fill(theme::surface())
                .corner_radius(6.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if !finished && icon_button(ui, glyph).clicked() {
                            toggle = Some(i);
                        }
                        if finished {
                            ui.label(RichText::new(glyph).color(theme::ACCENT_2).size(15.0));
                        }
                        ui.label(RichText::new(label).color(theme::text()).size(12.0));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if icon_button(ui, icon::X).clicked() {
                                remove = Some(i);
                            }
                            ui.label(RichText::new(rem).color(time_color).monospace().size(14.0));
                        });
                    });
                    progress_bar(ui, frac, finished);
                });
            ui.add_space(6.0);
        }
    });

    if let Some(i) = toggle {
        timer.countdowns[i].toggle();
    }
    if let Some(i) = remove {
        timer.countdowns.remove(i);
    }
}

// ════════════════════════════ Stopwatch ════════════════════════════

fn render_stopwatch(ui: &mut Ui, sw: &mut Stopwatch) {
    ui.vertical_centered(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new(fmt_sw(sw.elapsed()))
                .color(theme::text())
                .monospace()
                .size(36.0),
        );
        ui.add_space(14.0);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 230.0).max(0.0) / 2.0);
            let (lbl, gl) = if sw.is_running() {
                (t("Jeda", "Pause"), icon::PAUSE)
            } else {
                (t("Mulai", "Start"), icon::PLAY)
            };
            if accent_button(ui, &format!("{gl}  {lbl}")).clicked() {
                sw.toggle();
            }
            if ghost_button(ui, &format!("{}  {}", icon::FLAG, t("Putaran", "Lap"))).clicked() {
                sw.lap();
            }
            if ghost_button(ui, &format!("{}  Reset", icon::ARROW_COUNTER_CLOCKWISE)).clicked() {
                sw.reset();
            }
        });

        ui.add_space(12.0);

        if !sw.laps.is_empty() {
            if ghost_button(ui, &format!("{}  {}", icon::DOWNLOAD_SIMPLE, t("Ekspor .txt", "Export .txt"))).clicked() {
                export_laps(sw);
            }
            ui.add_space(8.0);
        }
    });

    // daftar lap (terbaru di atas)
    if sw.laps.is_empty() {
        return;
    }
    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for (idx, lap) in sw.laps.iter().enumerate().rev() {
            // selisih dari lap sebelumnya
            let split = if idx == 0 {
                *lap
            } else {
                lap.saturating_sub(sw.laps[idx - 1])
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Lap {}", idx + 1)).color(theme::muted()).size(12.0));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(fmt_sw(*lap)).color(theme::text()).monospace().size(13.0));
                    ui.label(
                        RichText::new(format!("+{}", fmt_sw(split)))
                            .color(theme::muted())
                            .monospace()
                            .size(11.0),
                    );
                });
            });
            ui.add_space(2.0);
        }
    });
}

fn export_laps(sw: &Stopwatch) {
    let text = sw.export_text();
    let dir = directories::UserDirs::new()
        .and_then(|u| u.document_dir().map(|d| d.to_path_buf()))
        .or_else(|| {
            directories::ProjectDirs::from("", "", "time-jutsu")
                .map(|p| p.config_dir().to_path_buf())
        });
    let Some(dir) = dir else {
        crate::utils::notifier::notify(
            t("Export gagal", "Export failed"),
            t("Folder tujuan tidak ditemukan", "Destination folder not found"),
        );
        return;
    };
    let _ = std::fs::create_dir_all(&dir);
    let fname = format!(
        "time-jutsu-laps-{}.txt",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    let path = dir.join(fname);
    match std::fs::write(&path, text) {
        Ok(_) => crate::utils::notifier::notify(
            t("Export berhasil", "Export successful"),
            &format!("{}", path.display()),
        ),
        Err(e) => crate::utils::notifier::notify(t("Export gagal", "Export failed"), &e.to_string()),
    }
}

// ════════════════════════════ Helpers ════════════════════════════

fn mode_button(ui: &mut Ui, timer: &mut TimerState, mode: TimerMode, text: &str) {
    let active = timer.mode == mode;
    let (fg, bg) = if active {
        (theme::on_accent(), theme::ACCENT)
    } else {
        (theme::text(), theme::surface_hi())
    };
    let btn = egui::Button::new(RichText::new(text).color(fg))
        .fill(bg)
        .corner_radius(6.0)
        .min_size(vec2(80.0, 26.0));
    if ui.add(btn).clicked() {
        timer.mode = mode;
    }
}

fn accent_button(ui: &mut Ui, text: &str) -> egui::Response {
    let btn = egui::Button::new(RichText::new(text).color(theme::on_accent()).strong())
        .fill(theme::ACCENT)
        .corner_radius(6.0)
        .min_size(vec2(96.0, 30.0));
    ui.add(btn)
}

fn ghost_button(ui: &mut Ui, text: &str) -> egui::Response {
    let btn = egui::Button::new(RichText::new(text).color(theme::text()))
        .fill(theme::surface_hi())
        .corner_radius(6.0)
        .min_size(vec2(0.0, 30.0));
    ui.add(btn)
}

fn icon_button(ui: &mut Ui, glyph: &str) -> egui::Response {
    let btn = egui::Button::new(RichText::new(glyph).color(theme::muted()).size(15.0))
        .frame(false)
        .min_size(vec2(22.0, 22.0));
    ui.add(btn)
}

fn progress_bar(ui: &mut Ui, frac: f32, finished: bool) {
    ui.add_space(6.0);
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 4.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 2.0, theme::surface_hi());
    let color = if finished { theme::ACCENT_2 } else { theme::ACCENT };
    let mut fg = rect;
    fg.set_width(rect.width() * frac.clamp(0.0, 1.0));
    painter.rect_filled(fg, 2.0, color);
}
