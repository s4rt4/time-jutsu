//! Abstraksi command OS-specific.
//!
//! ATURAN: semua percabangan OS hidup di file ini. UI/core hanya memanggil
//! fungsi publik di sini, tidak boleh tahu OS apa yang berjalan.
//! Windows & Linux didukung. Linux memakai D-Bus (deteksi host tray + idle)
//! dan systemctl (scheduler); diuji di GNOME/Wayland, portabel ke distro lain.

use std::process::Command;

/// Aksi sistem yang bisa dijadwalkan oleh Scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemAction {
    Shutdown,
    Restart,
    Sleep,
    Hibernate,
}

impl SystemAction {
    pub const ALL: [SystemAction; 4] = [
        SystemAction::Shutdown,
        SystemAction::Restart,
        SystemAction::Sleep,
        SystemAction::Hibernate,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SystemAction::Shutdown => crate::i18n::t("Matikan", "Shutdown"),
            SystemAction::Restart => crate::i18n::t("Mulai Ulang", "Restart"),
            SystemAction::Sleep => crate::i18n::t("Tidur", "Sleep"),
            SystemAction::Hibernate => crate::i18n::t("Hibernasi", "Hibernate"),
        }
    }
}

/// Bangun `Command` untuk aksi sistem tertentu, dengan jeda `delay_secs`
/// (hanya dipakai shutdown/restart di Windows; sleep/hibernate langsung).
#[cfg(target_os = "windows")]
pub fn build_action_command(action: SystemAction, delay_secs: u32) -> Command {
    let mut cmd;
    match action {
        SystemAction::Shutdown => {
            cmd = Command::new("shutdown");
            cmd.args(["/s", "/t", &delay_secs.to_string()]);
        }
        SystemAction::Restart => {
            cmd = Command::new("shutdown");
            cmd.args(["/r", "/t", &delay_secs.to_string()]);
        }
        SystemAction::Sleep => {
            cmd = Command::new("rundll32.exe");
            cmd.args(["powrprof.dll,SetSuspendState", "0,1,0"]);
        }
        SystemAction::Hibernate => {
            cmd = Command::new("shutdown");
            cmd.arg("/h");
        }
    }
    cmd
}

/// Cabang Linux — belum diuji, disiapkan agar kode tetap compile lintas-OS.
/// Memakai systemctl (polkit, tanpa sudo) alih-alih `shutdown -h` klasik.
#[cfg(target_os = "linux")]
pub fn build_action_command(action: SystemAction, _delay_secs: u32) -> Command {
    let mut cmd = Command::new("systemctl");
    match action {
        SystemAction::Shutdown => cmd.arg("poweroff"),
        SystemAction::Restart => cmd.arg("reboot"),
        SystemAction::Sleep => cmd.arg("suspend"),
        SystemAction::Hibernate => cmd.arg("hibernate"),
    };
    cmd
}

/// Batalkan jadwal sistem yang sudah terlanjur di-issue (Windows: `shutdown /a`).
/// Pengaman bila aksi sudah dikirim dengan delay tapi user menekan Cancel.
#[cfg(target_os = "windows")]
pub fn cancel_pending_system_action() {
    let _ = Command::new("shutdown").arg("/a").status();
}

#[cfg(target_os = "linux")]
pub fn cancel_pending_system_action() {
    // Dengan pendekatan internal-timer, di Linux tidak ada jadwal sistem
    // yang perlu dibatalkan (command di-issue saat timer mencapai nol).
}

/// Bulatkan sudut window via DWM (Windows 11). Tanpa transparansi —
/// OS yang meng-clip region window jadi rounded. Kembalikan true jika sukses.
#[cfg(target_os = "windows")]
pub fn round_window_corners(title: &str) -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    };
    use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

    let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let Ok(hwnd) = FindWindowW(PCWSTR::null(), PCWSTR(wide.as_ptr())) else {
            return false;
        };
        if hwnd.is_invalid() {
            return false;
        }
        let pref = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &pref as *const _ as *const core::ffi::c_void,
            std::mem::size_of_val(&pref) as u32,
        );
        true
    }
}

/// Linux: rounding ditangani compositor / nanti via transparansi. No-op sukses.
#[cfg(target_os = "linux")]
pub fn round_window_corners(_title: &str) -> bool {
    true
}

/// Berapa detik sejak input terakhir user (untuk auto-pause saat idle).
#[cfg(target_os = "windows")]
pub fn idle_seconds() -> u64 {
    use windows::Win32::System::SystemInformation::GetTickCount;
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    unsafe {
        let mut lii = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if GetLastInputInfo(&mut lii).as_bool() {
            let now = GetTickCount();
            (now.wrapping_sub(lii.dwTime) / 1000) as u64
        } else {
            0
        }
    }
}

/// Linux: idle via D-Bus. Coba GNOME (Mutter IdleMonitor) lalu freedesktop
/// ScreenSaver (KDE/lainnya). Bila tak satupun tersedia → 0 (anggap aktif).
#[cfg(target_os = "linux")]
pub fn idle_seconds() -> u64 {
    use zbus::blocking::Connection;
    let Ok(conn) = Connection::session() else {
        return 0;
    };

    // GNOME: org.gnome.Mutter.IdleMonitor.GetIdletime → u64 milidetik.
    let reply = conn.call_method(
        Some("org.gnome.Mutter.IdleMonitor"),
        "/org/gnome/Mutter/IdleMonitor/Core",
        Some("org.gnome.Mutter.IdleMonitor"),
        "GetIdletime",
        &(),
    );
    if let Ok(msg) = reply {
        if let Ok(ms) = msg.body().deserialize::<u64>() {
            return ms / 1000;
        }
    }

    // Fallback portabel: org.freedesktop.ScreenSaver.GetSessionIdleTime → u32 detik.
    let reply = conn.call_method(
        Some("org.freedesktop.ScreenSaver"),
        "/org/freedesktop/ScreenSaver",
        Some("org.freedesktop.ScreenSaver"),
        "GetSessionIdleTime",
        &(),
    );
    if let Ok(msg) = reply {
        if let Ok(secs) = msg.body().deserialize::<u32>() {
            return secs as u64;
        }
    }

    0
}

/// Apakah ada host system-tray sungguhan (StatusNotifierHost) di session bus?
/// GNOME default TANPA ekstensi AppIndicator = tidak ada → app tetap di dock.
/// Tanpa cek ini, ikon tray ter-register di D-Bus tapi tak terlihat, sehingga
/// "hide to tray" menyembunyikan window tanpa cara memunculkannya kembali.
#[cfg(target_os = "linux")]
pub fn tray_host_available() -> bool {
    use zbus::blocking::Connection;
    let Ok(conn) = Connection::session() else {
        return false;
    };
    let reply = conn.call_method(
        Some("org.kde.StatusNotifierWatcher"),
        "/StatusNotifierWatcher",
        Some("org.freedesktop.DBus.Properties"),
        "Get",
        &(
            "org.kde.StatusNotifierWatcher",
            "IsStatusNotifierHostRegistered",
        ),
    );
    let Ok(msg) = reply else {
        return false; // watcher tak ada di bus → tak ada tray
    };
    match msg.body().deserialize::<zbus::zvariant::Value>() {
        Ok(v) => bool::try_from(v).unwrap_or(false),
        Err(_) => false,
    }
}

/// Windows selalu punya tray (notification area).
#[cfg(target_os = "windows")]
pub fn tray_host_available() -> bool {
    true
}

/// Tampilkan & fokuskan window app via Win32 langsung (by title). Dipakai tray
/// & single-instance — bekerja walau eframe sedang tidak repaint (window hidden).
#[cfg(target_os = "windows")]
pub fn show_window(title: &str) {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, SetForegroundWindow, ShowWindow, SW_SHOW,
    };
    let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        if let Ok(hwnd) = FindWindowW(PCWSTR::null(), PCWSTR(wide.as_ptr())) {
            if !hwnd.is_invalid() {
                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = SetForegroundWindow(hwnd);
            }
        }
    }
}

/// Linux: un-hide & fokus ditangani lintas-platform di `app::handle_tray` via
/// `ViewportCommand::Visible(true)` + `Focus` (egui membangunkan eframe lewat
/// `request_repaint` dari handler tray, walau window sedang hidden). Wayland
/// melarang aplikasi memaksa raise window lain, jadi tak ada jalur Win32 di sini.
#[cfg(target_os = "linux")]
pub fn show_window(_title: &str) {}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn linux_actions_map_to_systemctl() {
        let cases = [
            (SystemAction::Shutdown, "poweroff"),
            (SystemAction::Restart, "reboot"),
            (SystemAction::Sleep, "suspend"),
            (SystemAction::Hibernate, "hibernate"),
        ];
        for (action, want) in cases {
            let cmd = build_action_command(action, 0);
            assert_eq!(cmd.get_program(), "systemctl");
            let args: Vec<String> = cmd
                .get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect();
            assert_eq!(args, vec![want.to_string()], "aksi {action:?}");
        }
    }
}
