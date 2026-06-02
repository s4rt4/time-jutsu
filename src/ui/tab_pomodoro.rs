use std::f32::consts::{FRAC_PI_2, TAU};

use egui::{
    vec2, Align, Align2, Color32, CursorIcon, FontId, Layout, Pos2, RichText, Sense, Shape, Stroke,
    Ui, Vec2,
};
use egui_phosphor::regular as icon;

use crate::core::pomodoro::{Phase, Pomodoro, PRESETS};
use crate::i18n::t;
use crate::ui::theme;

const ARC_SIZE: f32 = 168.0;
const ARC_WIDTH: f32 = 10.0;

pub fn render(ui: &mut Ui, pomo: &mut Pomodoro) {
    let phase = pomo.phase();
    let arc_color = match phase {
        Phase::Focus => theme::ACCENT,
        Phase::Break | Phase::LongBreak => theme::ACCENT_2,
    };

    ui.vertical_centered(|ui| {
        // ── Header: fase + nomor sesi ──────────────────────────────
        ui.add_space(2.0);
        ui.label(
            RichText::new(phase.label().to_uppercase())
                .color(arc_color)
                .strong()
                .size(13.0),
        );
        ui.label(
            RichText::new(format!(
                "{}{} {}",
                t("Sesi ke-", "Session #"),
                pomo.current_session(),
                t("hari ini", "today")
            ))
            .color(theme::muted())
            .size(11.0),
        );

        ui.add_space(10.0);
        draw_arc(ui, pomo, arc_color);
        ui.add_space(16.0);

        // ── Start/Pause (di tengah) ────────────────────────────────
        let (label, glyph) = if pomo.is_running() {
            (t("Jeda", "Pause"), icon::PAUSE)
        } else {
            (t("Mulai", "Start"), icon::PLAY)
        };
        if start_button(ui, &format!("{glyph}  {label}"), arc_color).clicked() {
            pomo.toggle();
        }

        // ── Reset (teks bisa diklik, di bawah Start) ───────────────
        ui.add_space(8.0);
        if reset_link(ui) {
            pomo.reset();
        }

        ui.add_space(22.0);

        // ── Preset ─────────────────────────────────────────────────
        ui.label(RichText::new("PRESET").color(theme::muted()).size(10.0));
        ui.add_space(6.0);
        centered_row(ui, 192.0, 30.0, |ui| {
            for (w, b) in PRESETS {
                let active = pomo.work_minutes == w && pomo.break_minutes == b;
                if preset_button(ui, &format!("{w} / {b}"), active).clicked() {
                    pomo.set_preset(w, b);
                }
            }
        });

        ui.add_space(8.0);
        ui.add_enabled_ui(!pomo.is_running(), |ui| {
            centered_row(ui, 192.0, 26.0, |ui| {
                ui.label(RichText::new(t("Kustom", "Custom")).color(theme::muted()).size(12.0));
                let mut w = pomo.work_minutes;
                let mut b = pomo.break_minutes;
                let r1 = ui.add(egui::DragValue::new(&mut w).range(1..=180).suffix("m"));
                ui.label(RichText::new("/").color(theme::muted()));
                let r2 = ui.add(egui::DragValue::new(&mut b).range(1..=120).suffix("m"));
                if r1.changed() || r2.changed() {
                    pomo.set_preset(w, b);
                }
            });
        });

        ui.add_space(16.0);
        ui.checkbox(
            &mut pomo.sound_enabled,
            RichText::new(t("Suara notifikasi", "Notification sound")).color(theme::text()),
        );
    });
}

/// Alokasikan satu baris selebar `width` lalu isi kiri→kanan. Karena induknya
/// `vertical_centered`, blok selebar `width` ini otomatis ter-tengah.
fn centered_row<R>(ui: &mut Ui, width: f32, height: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.allocate_ui_with_layout(vec2(width, height), Layout::left_to_right(Align::Center), add)
        .inner
}

/// Titik-titik di sepanjang busur lingkaran (radius `r`) mulai sudut `start`
/// sejauh `sweep`, dibagi `seg` segmen.
fn arc_points(center: Pos2, r: f32, start: f32, sweep: f32, seg: usize) -> Vec<Pos2> {
    (0..=seg)
        .map(|i| {
            let a = start + sweep * (i as f32 / seg as f32);
            center + vec2(a.cos(), a.sin()) * r
        })
        .collect()
}

/// Gambar ring background + arc progress + teks MM:SS di tengah.
/// KEDUA ring digambar dengan metode polyline yang sama (radius & tebal
/// identik) agar progress menempel persis di atas ring belakang.
fn draw_arc(ui: &mut Ui, pomo: &Pomodoro, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(ARC_SIZE), Sense::hover());
    let painter = ui.painter();
    let center = rect.center();
    let radius = ARC_SIZE / 2.0 - ARC_WIDTH;
    let start = -FRAC_PI_2;

    // Ring belakang: polyline lingkaran penuh (bukan circle_stroke) supaya
    // radius efektifnya sama persis dengan arc progress.
    let bg = arc_points(center, radius, start, TAU, 256);
    painter.add(Shape::line(bg, Stroke::new(ARC_WIDTH, theme::surface_hi())));

    let frac = pomo.progress();
    if frac > 0.0 {
        let sweep = frac * TAU;
        let seg = ((256.0 * frac).ceil() as usize).max(2);
        let pts = arc_points(center, radius, start, sweep, seg);
        painter.add(Shape::line(pts.clone(), Stroke::new(ARC_WIDTH, color)));
        // Rounded cap manual di kedua ujung (egui Stroke tidak punya line-cap).
        let cap = ARC_WIDTH / 2.0;
        painter.circle_filled(pts[0], cap, color);
        painter.circle_filled(pts[pts.len() - 1], cap, color);
    }

    let secs = pomo.remaining().as_secs();
    let txt = format!("{:02}:{:02}", secs / 60, secs % 60);
    painter.text(
        center,
        Align2::CENTER_CENTER,
        txt,
        FontId::monospace(38.0),
        theme::text(),
    );
}

fn start_button(ui: &mut Ui, text: &str, fill: Color32) -> egui::Response {
    let btn = egui::Button::new(RichText::new(text).color(theme::on_accent()).strong().size(14.0))
        .fill(fill)
        .corner_radius(6.0)
        .min_size(vec2(150.0, 34.0));
    ui.add(btn)
}

/// "Reset" sebagai teks bisa diklik (bukan tombol). Redup → putih saat hover.
fn reset_link(ui: &mut Ui) -> bool {
    let text = format!("{}  Reset", icon::ARROW_COUNTER_CLOCKWISE);
    let font = FontId::proportional(13.0);
    let size = ui
        .painter()
        .layout_no_wrap(text.clone(), font.clone(), theme::muted())
        .size();
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let color = if resp.hovered() {
        theme::text()
    } else {
        theme::muted()
    };
    let galley = ui.painter().layout_no_wrap(text, font, color);
    ui.painter().galley(rect.min, galley, color);
    resp.on_hover_cursor(CursorIcon::PointingHand).clicked()
}

fn preset_button(ui: &mut Ui, text: &str, active: bool) -> egui::Response {
    let (fg, bg) = if active {
        (theme::on_accent(), theme::ACCENT)
    } else {
        (theme::text(), theme::surface_hi())
    };
    let btn = egui::Button::new(RichText::new(text).color(fg))
        .fill(bg)
        .corner_radius(6.0)
        .min_size(vec2(88.0, 28.0));
    ui.add(btn)
}
