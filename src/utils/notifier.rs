//! Wrapper notifikasi sistem (notify-rust, cross-platform).

use notify_rust::Notification;

/// Tampilkan notifikasi sistem. Gagal diam-diam (mis. service notif mati)
/// agar tidak meng-crash app.
pub fn notify(title: &str, body: &str) {
    let _ = Notification::new()
        .summary(title)
        .body(body)
        .appname("Time-Jutsu")
        .show();
}
