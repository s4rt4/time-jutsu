//! System tray: icon + menu (Show / Quit).
//!
//! tray-icon mengekspos event lewat channel global (`MenuEvent::receiver()`).
//! App mem-poll channel ini tiap frame di `update()`.

use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

/// Apa yang diminta user lewat menu tray.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayCommand {
    Show,
    Quit,
}

/// Memegang TrayIcon (harus tetap hidup selama app jalan) + id menu item.
pub struct TrayState {
    _tray: TrayIcon,
    show_id: tray_icon::menu::MenuId,
    quit_id: tray_icon::menu::MenuId,
}

impl TrayState {
    pub fn new() -> Option<Self> {
        let menu = Menu::new();
        let show_item = MenuItem::new("Show", true, None);
        let quit_item = MenuItem::new("Quit", true, None);
        let show_id = show_item.id().clone();
        let quit_id = quit_item.id().clone();
        menu.append(&show_item).ok()?;
        menu.append(&quit_item).ok()?;

        let icon = build_icon();
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Time-Jutsu")
            .with_icon(icon)
            .build()
            .ok()?;

        Some(Self {
            _tray: tray,
            show_id,
            quit_id,
        })
    }

    /// Ambil command tray yang tertunda (non-blocking). None jika tidak ada.
    pub fn poll(&self) -> Option<TrayCommand> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.show_id {
                return Some(TrayCommand::Show);
            }
            if event.id == self.quit_id {
                return Some(TrayCommand::Quit);
            }
        }
        None
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
