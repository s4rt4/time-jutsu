use egui::{RichText, Ui};
use egui_phosphor::regular as icon;

use crate::ui::theme;

pub fn render(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(24.0);

        // Logo (PNG di-embed, di-decode oleh egui_extras image loader).
        ui.add(
            egui::Image::new(egui::include_image!("../../assets/logo-256.png"))
                .max_width(92.0),
        );

        ui.add_space(12.0);
        ui.label(
            RichText::new("Time-Jutsu")
                .color(theme::text())
                .strong()
                .size(22.0),
        );
        ui.label(
            RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                .color(theme::ACCENT)
                .strong()
                .size(12.0),
        );

        ui.add_space(12.0);
        ui.label(
            RichText::new("Master your time.")
                .color(theme::muted())
                .italics()
                .size(12.0),
        );
        ui.label(
            RichText::new("Strike with precision.")
                .color(theme::muted())
                .italics()
                .size(12.0),
        );

        ui.add_space(22.0);

        info_row(ui, icon::CODE, "Rust + egui");
        info_row(ui, icon::DESKTOP, "Windows · Linux");
        info_row(ui, icon::PALETTE, "Navy · Coral");

        ui.add_space(22.0);
        ui.label(
            RichText::new("© 2026 Time-Jutsu")
                .color(theme::muted())
                .size(10.0),
        );
    });
}

fn info_row(ui: &mut Ui, glyph: &str, text: &str) {
    ui.horizontal(|ui| {
        // pusatkan baris icon+teks
        let content_w = 150.0;
        ui.add_space((ui.available_width() - content_w).max(0.0) / 2.0);
        ui.label(RichText::new(glyph).color(theme::ACCENT).size(14.0));
        ui.label(RichText::new(text).color(theme::muted()).size(12.0));
    });
    ui.add_space(4.0);
}
