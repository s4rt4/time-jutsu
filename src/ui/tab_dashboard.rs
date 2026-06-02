use egui::{vec2, Align, Align2, FontId, Frame, Layout, Pos2, Rect, RichText, Sense, Ui};
use egui_phosphor::regular as icon;

use crate::core::alarm::AlarmState;
use crate::core::countdown::TimerState;
use crate::core::fmt_hms;
use crate::core::pomodoro::Pomodoro;
use crate::core::scheduler::Scheduler;
use crate::core::tracking::Tracker;
use crate::i18n::t;
use crate::ui::theme;

#[allow(clippy::too_many_arguments)]
pub fn render(
    ui: &mut Ui,
    pomo: &Pomodoro,
    timer: &TimerState,
    sched: &Scheduler,
    alarm: &AlarmState,
    tracker: &Tracker,
    today: (u32, u32),
    week: &[(String, u32)],
    streak: u32,
) {
    ui.label(RichText::new("Dashboard").color(theme::text()).strong().size(15.0));
    ui.add_space(12.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
    // ── Hari ini: dua stat card ────────────────────────────────────
    let (sessions, focus_min) = today;
    let focus = fmt_minutes(focus_min);
    ui.columns(2, |cols| {
        stat_card(&mut cols[0], icon::CHECK_CIRCLE, &sessions.to_string(), t("Sesi fokus", "Focus sessions"));
        stat_card(&mut cols[1], icon::CLOCK, &focus, t("Waktu fokus", "Focus time"));
    });

    ui.add_space(12.0);

    // ── Minggu ini: grafik + streak ────────────────────────────────
    ui.label(RichText::new(t("MINGGU INI", "THIS WEEK")).color(theme::muted()).size(10.0));
    ui.add_space(4.0);
    week_card(ui, week, streak);

    ui.add_space(12.0);
    ui.label(RichText::new(t("SEDANG AKTIF", "ACTIVE NOW")).color(theme::muted()).size(10.0));
    ui.add_space(6.0);

    let pomo_val = pomo
        .is_running()
        .then(|| format!("{}  {}", pomo.phase().label(), fmt_hms(pomo.remaining())));
    active_card(ui, icon::TIMER, "Pomodoro", pomo_val);

    let running = timer.countdowns.iter().filter(|c| c.is_running()).count();
    active_card(
        ui,
        icon::HOURGLASS,
        "Countdown",
        (running > 0).then(|| format!("{running} {}", t("berjalan", "running"))),
    );

    active_card(
        ui,
        icon::TIMER,
        "Stopwatch",
        timer.stopwatch.is_running().then(|| t("berjalan", "running").to_string()),
    );

    active_card(ui, icon::BRIEFCASE, "Tracking", tracker.active_name().map(|n| n.to_string()));

    let sch_val = sched
        .remaining()
        .map(|rem| format!("{}  {}", sched.action.label(), fmt_hms(rem)));
    active_card(ui, icon::POWER, "Scheduler", sch_val);

    active_card(ui, icon::BELL, t("Alarm berikutnya", "Next alarm"), alarm.next_active());

    let dl_val = timer
        .next_deadline()
        .map(|d| format!("{}  {}", d.label, d.remaining_text()));
    active_card(ui, icon::FLAG, "Deadline", dl_val);

            ui.add_space(8.0); // ruang bawah
        });
}

fn fmt_minutes(m: u32) -> String {
    let (h, mm) = (m / 60, m % 60);
    if h > 0 {
        format!("{h}{} {mm}{}", t("j", "h"), t("m", "m"))
    } else {
        format!("{mm}{}", t("m", "m"))
    }
}

/// Kartu grafik batang 7 hari + chip streak.
fn week_card(ui: &mut Ui, week: &[(String, u32)], streak: u32) {
    Frame::new()
        .fill(theme::surface())
        .corner_radius(10.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            // chip streak
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon::FIRE).color(theme::ACCENT).size(15.0));
                ui.label(
                    RichText::new(format!("{streak} {}", t("hari beruntun", "day streak")))
                        .color(theme::text())
                        .size(12.0),
                );
            });
            ui.add_space(10.0);

            // grafik batang
            let max = week.iter().map(|(_, m)| *m).max().unwrap_or(0).max(1) as f32;
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 64.0), Sense::hover());
            let painter = ui.painter();
            let n = week.len().max(1);
            let gap = 8.0;
            let bw = ((rect.width() - gap * (n as f32 - 1.0)) / n as f32).max(2.0);
            let label_h = 14.0;
            let chart_bottom = rect.bottom() - label_h;
            let chart_h = (chart_bottom - rect.top()).max(1.0);

            for (i, (lbl, mins)) in week.iter().enumerate() {
                let x = rect.left() + i as f32 * (bw + gap);
                let h = (*mins as f32 / max) * chart_h;
                let today = i == n - 1;
                // track (latar batang)
                let track = Rect::from_min_max(
                    Pos2::new(x, rect.top()),
                    Pos2::new(x + bw, chart_bottom),
                );
                painter.rect_filled(track, 3.0, theme::surface_hi());
                if *mins > 0 {
                    let bar = Rect::from_min_max(
                        Pos2::new(x, chart_bottom - h),
                        Pos2::new(x + bw, chart_bottom),
                    );
                    let color = if today { theme::ACCENT } else { theme::ACCENT_2 };
                    painter.rect_filled(bar, 3.0, color);
                }
                painter.text(
                    Pos2::new(x + bw / 2.0, rect.bottom() - label_h / 2.0),
                    Align2::CENTER_CENTER,
                    lbl,
                    FontId::proportional(9.0),
                    if today { theme::text() } else { theme::muted() },
                );
            }
        });
}

fn stat_card(ui: &mut Ui, glyph: &str, value: &str, label: &str) {
    Frame::new()
        .fill(theme::surface())
        .corner_radius(10.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(glyph).color(theme::ACCENT).size(16.0));
            ui.add_space(2.0);
            ui.label(RichText::new(value).color(theme::text()).strong().size(24.0));
            ui.label(RichText::new(label).color(theme::muted()).size(11.0));
        });
}

fn active_card(ui: &mut Ui, glyph: &str, name: &str, value: Option<String>) {
    let active = value.is_some();
    let icon_color = if active { theme::ACCENT } else { theme::muted() };
    let val_color = if active { theme::ACCENT_2 } else { theme::muted() };
    let val_text = value.unwrap_or_else(|| "—".to_string());

    Frame::new()
        .fill(theme::surface())
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(glyph).color(icon_color).size(15.0));
                ui.add_space(2.0);
                ui.label(RichText::new(name).color(theme::text()).size(12.0));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(val_text).color(val_color).size(12.0));
                });
            });
        });
    ui.add_space(6.0);
}
