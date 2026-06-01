//! Logic murni (tanpa dependensi egui) supaya bisa di-unit-test terpisah.

pub mod alarm;
pub mod countdown;
pub mod deadline;
pub mod pomodoro;
pub mod scheduler;
pub mod stopwatch;
pub mod tracking;

use std::time::Duration;

/// Format durasi: HH:MM:SS bila ≥ 1 jam, selain itu MM:SS.
pub fn fmt_hms(d: Duration) -> String {
    let s = d.as_secs();
    let (h, m, sec) = (s / 3600, (s % 3600) / 60, s % 60);
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, sec)
    } else {
        format!("{:02}:{:02}", m, sec)
    }
}
