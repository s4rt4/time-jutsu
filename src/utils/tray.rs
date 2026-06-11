//! System tray: icon + menu yang lebih kaya (Tampilkan / quick-jump tab / Keluar).
//!
//! Saat window di-hide ke tray, eframe berhenti `update()`. Maka handler tray
//! langsung memunculkan window via Win32 (`platform::show_window`) — itu juga
//! membangunkan eframe (event focus) sehingga command di-antrian ikut diproses.

use std::sync::Mutex;

use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    Show,
    OpenPomodoro,
    OpenTracking,
    OpenDashboard,
    Quit,
}

/// Antrian command dari handler tray → diproses di `update()`.
static QUEUE: Mutex<Vec<TrayCommand>> = Mutex::new(Vec::new());

fn push(cmd: TrayCommand) {
    if let Ok(mut q) = QUEUE.lock() {
        q.push(cmd);
    }
}

pub struct TrayState {
    _tray: TrayIcon,
    // simpan item agar teksnya bisa diperbarui saat bahasa berganti
    i_show: MenuItem,
    i_track: MenuItem,
    i_dash: MenuItem,
    i_quit: MenuItem,
}

fn t(id: &'static str, en: &'static str) -> &'static str {
    crate::i18n::t(id, en)
}

impl TrayState {
    pub fn new(ctx: &egui::Context) -> Option<Self> {
        // Tanpa host tray sungguhan (mis. GNOME default), JANGAN buat tray:
        // app akan tetap tampil di dock & perilaku hide-to-tray nonaktif.
        if !crate::utils::platform::tray_host_available() {
            return None;
        }

        let menu = Menu::new();
        let i_show = MenuItem::new(t("Tampilkan", "Show"), true, None);
        let i_pomo = MenuItem::new("Pomodoro", true, None);
        let i_track = MenuItem::new(t("Pelacak Waktu", "Time Tracking"), true, None);
        let i_dash = MenuItem::new(t("Dasbor", "Dashboard"), true, None);
        let i_quit = MenuItem::new(t("Keluar", "Quit"), true, None);

        let _ = menu.append(&i_show);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&i_pomo);
        let _ = menu.append(&i_track);
        let _ = menu.append(&i_dash);
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&i_quit);

        let (show_id, pomo_id, track_id, dash_id, quit_id) = (
            i_show.id().clone(),
            i_pomo.id().clone(),
            i_track.id().clone(),
            i_dash.id().clone(),
            i_quit.id().clone(),
        );

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Time-Jutsu")
            .with_icon(build_icon())
            .build()
            .ok()?;

        // Menu klik kanan.
        let menu_ctx = ctx.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let cmd = if event.id == show_id {
                TrayCommand::Show
            } else if event.id == pomo_id {
                TrayCommand::OpenPomodoro
            } else if event.id == track_id {
                TrayCommand::OpenTracking
            } else if event.id == dash_id {
                TrayCommand::OpenDashboard
            } else if event.id == quit_id {
                TrayCommand::Quit
            } else {
                return;
            };
            push(cmd);
            // munculkan window (kecuali Quit, tetap dipanggil utk bangunkan eframe).
            crate::utils::platform::show_window("Time-Jutsu");
            menu_ctx.request_repaint();
        }));

        // Klik kiri icon → Show.
        let click_ctx = ctx.clone();
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
            if let TrayIconEvent::Click {
                button: tray_icon::MouseButton::Left,
                button_state: tray_icon::MouseButtonState::Up,
                ..
            } = event
            {
                push(TrayCommand::Show);
                crate::utils::platform::show_window("Time-Jutsu");
                click_ctx.request_repaint();
            }
        }));

        Some(Self {
            _tray: tray,
            i_show,
            i_track,
            i_dash,
            i_quit,
        })
    }

    /// Perbarui label menu sesuai bahasa aktif (menu OS dibuat sekali).
    pub fn update_lang(&self) {
        self.i_show.set_text(t("Tampilkan", "Show"));
        self.i_track.set_text(t("Pelacak Waktu", "Time Tracking"));
        self.i_dash.set_text(t("Dasbor", "Dashboard"));
        self.i_quit.set_text(t("Keluar", "Quit"));
    }

    /// Ambil semua command tertunda (FIFO).
    pub fn drain() -> Vec<TrayCommand> {
        match QUEUE.lock() {
            Ok(mut q) => std::mem::take(&mut *q),
            Err(_) => Vec::new(),
        }
    }
}

/// Icon tray = logo brand (PNG 32×32 transparan di-embed).
fn build_icon() -> tray_icon::Icon {
    let bytes = include_bytes!("../../assets/logo-32.png");
    let img = image::load_from_memory(bytes)
        .expect("decode logo png")
        .to_rgba8();
    let (w, h) = img.dimensions();
    tray_icon::Icon::from_rgba(img.into_raw(), w, h).expect("icon rgba valid")
}
