//! Logic Alarm, Reminder & Break Alert.
//!
//! Alarm dicek terhadap jam dinding (`chrono::Local`). Field runtime
//! (`last_fire_key`, `last`) di-skip dari serialisasi.

use std::time::{Duration, Instant};

use chrono::{DateTime, Datelike, Local, Timelike, Weekday};
use serde::{Deserialize, Serialize};

use crate::utils::sound::AlarmSound;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Repeat {
    Once,
    Daily,
    Weekdays,
}

impl Repeat {
    pub fn label(self) -> &'static str {
        match self {
            Repeat::Once => "Sekali",
            Repeat::Daily => "Harian",
            Repeat::Weekdays => "Hari kerja",
        }
    }
    pub const ALL: [Repeat; 3] = [Repeat::Once, Repeat::Daily, Repeat::Weekdays];
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Alarm {
    pub label: String,
    pub hour: u32,
    pub minute: u32,
    pub enabled: bool,
    pub repeat: Repeat,
    /// Kunci "YYYY-MM-DD HH:MM" terakhir berbunyi (anti dobel dalam 1 menit).
    #[serde(skip)]
    last_fire_key: Option<String>,
}

impl Alarm {
    pub fn new(label: String, hour: u32, minute: u32, repeat: Repeat) -> Self {
        Self {
            label,
            hour,
            minute,
            enabled: true,
            repeat,
            last_fire_key: None,
        }
    }

    pub fn time_label(&self) -> String {
        format!("{:02}:{:02}", self.hour, self.minute)
    }

    /// Cek apakah alarm berbunyi sekarang. Kembalikan label bila ya.
    pub fn check(&mut self, now: DateTime<Local>) -> Option<String> {
        if !self.enabled {
            return None;
        }
        if now.hour() != self.hour || now.minute() != self.minute {
            return None;
        }
        let key = format!("{} {:02}:{:02}", now.date_naive(), self.hour, self.minute);
        if self.last_fire_key.as_deref() == Some(key.as_str()) {
            return None; // sudah berbunyi menit ini
        }
        let day_ok = match self.repeat {
            Repeat::Once | Repeat::Daily => true,
            Repeat::Weekdays => !matches!(now.weekday(), Weekday::Sat | Weekday::Sun),
        };
        if !day_ok {
            return None;
        }
        self.last_fire_key = Some(key);
        if self.repeat == Repeat::Once {
            self.enabled = false;
        }
        Some(self.label.clone())
    }
}

/// Break Alert: ingatkan istirahat tiap X menit kerja.
#[derive(Clone)]
pub struct BreakAlert {
    pub enabled: bool,
    pub interval_minutes: u32,
    last: Option<Instant>,
}

impl Default for BreakAlert {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_minutes: 60,
            last: None,
        }
    }
}

impl BreakAlert {
    /// Kembalikan true bila interval tercapai (waktunya istirahat).
    pub fn check(&mut self) -> bool {
        if !self.enabled {
            self.last = None;
            return false;
        }
        let now = Instant::now();
        match self.last {
            None => {
                self.last = Some(now);
                false
            }
            Some(t) => {
                if t.elapsed() >= Duration::from_secs(self.interval_minutes.max(1) as u64 * 60) {
                    self.last = Some(now);
                    true
                } else {
                    false
                }
            }
        }
    }
}

/// State Tab Alarm: daftar alarm + break alert + field input.
pub struct AlarmState {
    pub alarms: Vec<Alarm>,
    pub break_alert: BreakAlert,
    /// Nada yang dipakai untuk alarm/break/notifikasi.
    pub sound: AlarmSound,
    pub input_label: String,
    pub input_hour: u32,
    pub input_minute: u32,
    pub input_repeat: Repeat,
}

impl Default for AlarmState {
    fn default() -> Self {
        Self {
            alarms: Vec::new(),
            break_alert: BreakAlert::default(),
            sound: AlarmSound::default(),
            input_label: String::new(),
            input_hour: 7,
            input_minute: 0,
            input_repeat: Repeat::Daily,
        }
    }
}

impl AlarmState {
    pub fn add(&mut self) {
        let label = if self.input_label.trim().is_empty() {
            "Alarm".to_string()
        } else {
            self.input_label.trim().to_string()
        };
        self.alarms.push(Alarm::new(
            label,
            self.input_hour,
            self.input_minute,
            self.input_repeat,
        ));
        self.input_label.clear();
    }

    /// Tick: cek semua alarm + break alert. Kembalikan (label_alarm_berbunyi, break_due).
    pub fn tick(&mut self) -> (Vec<String>, bool) {
        let now = Local::now();
        let mut fired = Vec::new();
        for a in &mut self.alarms {
            if let Some(l) = a.check(now) {
                fired.push(l);
            }
        }
        (fired, self.break_alert.check())
    }

    /// Deskripsi alarm aktif berikutnya (untuk Dashboard).
    pub fn next_active(&self) -> Option<String> {
        self.alarms
            .iter()
            .filter(|a| a.enabled)
            .min_by_key(|a| a.hour * 60 + a.minute)
            .map(|a| format!("{}  {}", a.time_label(), a.label))
    }
}
