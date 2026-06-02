//! Logic Scheduler — internal timer (bukan jadwal sistem). App tetap hidup
//! di tray, jadi countdown dihitung internal; saat target tercapai, command
//! sistem baru di-issue. Cancel sebelum target = cukup batalkan target.

use std::time::Duration;

use chrono::{DateTime, Local, Timelike};

use crate::utils::platform::{self, SystemAction};

pub struct Scheduler {
    pub action: SystemAction,
    /// false = "setelah X menit", true = "pada jam HH:MM".
    pub use_at_time: bool,
    pub after_minutes: u32,
    pub at_hour: u32,
    pub at_minute: u32,
    target: Option<DateTime<Local>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self {
            action: SystemAction::Shutdown,
            use_at_time: false,
            after_minutes: 30,
            at_hour: 23,
            at_minute: 0,
            target: None,
        }
    }
}

impl Scheduler {
    pub fn is_armed(&self) -> bool {
        self.target.is_some()
    }

    /// Hitung target dari input saat ini (None jika input tidak valid).
    fn compute_target(&self) -> Option<DateTime<Local>> {
        let now = Local::now();
        if self.use_at_time {
            let mut t = now
                .with_hour(self.at_hour)?
                .with_minute(self.at_minute)?
                .with_second(0)?
                .with_nanosecond(0)?;
            if t <= now {
                t += chrono::Duration::days(1);
            }
            Some(t)
        } else {
            Some(now + chrono::Duration::minutes(self.after_minutes.max(0) as i64))
        }
    }

    pub fn arm(&mut self) {
        self.target = self.compute_target();
    }

    pub fn cancel(&mut self) {
        self.target = None;
        // pengaman: batalkan jadwal sistem bila sempat ter-issue.
        platform::cancel_pending_system_action();
    }

    /// Sisa waktu menuju eksekusi (None bila tidak armed).
    pub fn remaining(&self) -> Option<Duration> {
        self.target
            .map(|t| (t - Local::now()).to_std().unwrap_or(Duration::ZERO))
    }

    /// Teks preview aksi.
    pub fn preview(&self) -> String {
        let target = if self.is_armed() {
            self.target
        } else {
            self.compute_target()
        };
        match target {
            Some(target) => format!(
                "{} {} {} {}",
                crate::i18n::t("Komputer akan", "Computer will"),
                self.action.label().to_lowercase(),
                crate::i18n::t("pukul", "at"),
                target.format("%H:%M")
            ),
            None => crate::i18n::t("Waktu tidak valid", "Invalid time").to_string(),
        }
    }

    /// Tick: jika target tercapai, jalankan aksi sistem. Kembalikan label aksi
    /// yang dijalankan (untuk notifikasi).
    pub fn tick(&mut self) -> Option<&'static str> {
        let target = self.target?;
        if Local::now() >= target {
            self.target = None;
            // beri grace 30 detik (untuk shutdown/restart) agar bisa dibatalkan.
            let _ = platform::build_action_command(self.action, 30).spawn();
            Some(self.action.label())
        } else {
            None
        }
    }
}
