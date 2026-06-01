//! Left icon-only tab bar (~44px). Placeholder pakai huruf/emoji sampai
//! asset PNG 24×24 tersedia. Tab aktif diberi accent strip kiri.

use crate::app::Tab;
use crate::ui::theme;
use egui::{Align, Color32, Layout, Sense, Ui, Vec2};

const BAR_W: f32 = 44.0;
const ITEM_H: f32 = 48.0;

/// Render sidebar; kembalikan true jika tab berubah.
pub fn render(ui: &mut Ui, current: &mut Tab) -> bool {
    let mut changed = false;
    ui.allocate_ui_with_layout(
        Vec2::new(BAR_W, ui.available_height()),
        Layout::top_down(Align::Center),
        |ui| {
            ui.add_space(6.0);
            for tab in Tab::ALL {
                if item(ui, tab, *current == tab) {
                    if *current != tab {
                        *current = tab;
                        changed = true;
                    }
                }
            }
            // About ditempel di bawah sidebar.
            let push = (ui.available_height() - ITEM_H).max(0.0);
            ui.add_space(push);
            if item(ui, Tab::About, *current == Tab::About) && *current != Tab::About {
                *current = Tab::About;
                changed = true;
            }
        },
    );
    changed
}

fn item(ui: &mut Ui, tab: Tab, active: bool) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(BAR_W, ITEM_H), Sense::click());
    let painter = ui.painter();

    if active {
        // accent strip kiri
        let strip = egui::Rect::from_min_size(rect.left_top(), Vec2::new(3.0, ITEM_H));
        painter.rect_filled(strip, 0.0, theme::ACCENT);
    } else if resp.hovered() {
        painter.rect_filled(rect, 4.0, theme::surface_hi());
    }

    let color: Color32 = if active {
        theme::ACCENT
    } else if resp.hovered() {
        theme::text()
    } else {
        theme::muted()
    };

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        tab.glyph(),
        egui::FontId::proportional(18.0),
        color,
    );

    let resp = resp.on_hover_text(tab.title());
    resp.clicked()
}
