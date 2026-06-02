pub mod sidebar;
pub mod theme;

pub mod tab_about;
pub mod tab_pomodoro;
pub mod tab_timer;
pub mod tab_scheduler;
pub mod tab_alarm;
pub mod tab_dashboard;
pub mod tab_settings;
pub mod tab_tracking;

use egui::{ComboBox, RichText, Ui};
use egui_phosphor::regular as icon;

use crate::utils::sound::{self, AlarmSound};

/// Picker nada + tombol preview (dipakai bersama Tab Pomodoro & Alarm).
/// Kembalikan true bila pilihan berubah (untuk disimpan).
pub fn sound_picker(ui: &mut Ui, sound: &mut AlarmSound) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{}  {}", icon::MUSIC_NOTES, crate::i18n::t("Nada", "Sound")))
                .color(theme::muted())
                .size(12.0),
        );
        ComboBox::from_id_salt("sound_picker")
            .selected_text(sound.label())
            .show_ui(ui, |ui| {
                for s in AlarmSound::ALL {
                    if ui.selectable_value(sound, s, s.label()).clicked() {
                        changed = true;
                    }
                }
            });
        let play = egui::Button::new(RichText::new(icon::PLAY).color(theme::on_accent()))
            .fill(theme::ACCENT)
            .corner_radius(6.0)
            .min_size(egui::vec2(28.0, 22.0));
        if ui.add(play).on_hover_text(crate::i18n::t("Preview nada", "Preview sound")).clicked() {
            sound::play(*sound);
        }
    });
    changed
}
