//! Config & persistence ke `config.json`.
//!
//! Lokasi via crate `directories` (lintas-OS, tidak hardcode):
//!   Windows: %APPDATA%\time-jutsu\   Linux: ~/.config/time-jutsu/
//! Daily log dirotasi 90 hari terakhir agar file tetap ramping.

use std::collections::BTreeMap;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::core::alarm::Alarm;
use crate::core::deadline::Deadline;
use crate::core::tracking::Project;
use crate::i18n::Lang;
use crate::ui::theme::ThemeMode;
use crate::utils::sound::AlarmSound;

#[derive(Serialize, Deserialize, Clone)]
pub struct PomodoroCfg {
    pub work_minutes: u32,
    pub break_minutes: u32,
    pub long_break_minutes: u32,
    pub sound_enabled: bool,
}

impl Default for PomodoroCfg {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            break_minutes: 5,
            long_break_minutes: 15,
            sound_enabled: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BreakAlertCfg {
    pub enabled: bool,
    pub interval_minutes: u32,
}

impl Default for BreakAlertCfg {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_minutes: 60,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct DayLog {
    pub pomodoro_sessions: u32,
    pub focus_minutes: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub theme: ThemeMode,
    pub lang: Lang,
    pub pomodoro: PomodoroCfg,
    pub alarms: Vec<Alarm>,
    pub break_alert: BreakAlertCfg,
    pub alarm_sound: AlarmSound,
    /// Sembunyikan ke tray saat tombol close ditekan (bukan keluar app).
    pub tray_on_close: bool,
    /// Sembunyikan ke tray saat di-minimize (bukan ke taskbar).
    pub tray_on_minimize: bool,
    /// Auto-pause Pomodoro saat user idle (AFK).
    pub idle_autopause: bool,
    pub projects: Vec<Project>,
    pub deadlines: Vec<Deadline>,
    /// tanggal "YYYY-MM-DD" → ringkasan hari itu.
    pub daily_log: BTreeMap<String, DayLog>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: ThemeMode::default(),
            lang: Lang::default(),
            pomodoro: PomodoroCfg::default(),
            alarms: Vec::new(),
            break_alert: BreakAlertCfg::default(),
            alarm_sound: AlarmSound::default(),
            tray_on_close: true,
            tray_on_minimize: false,
            idle_autopause: true,
            projects: Vec::new(),
            deadlines: Vec::new(),
            daily_log: BTreeMap::new(),
        }
    }
}

impl Config {
    fn path() -> Option<PathBuf> {
        let dirs = ProjectDirs::from("", "", "time-jutsu")?;
        Some(dirs.config_dir().join("config.json"))
    }

    /// Muat config dari disk; jika gagal/belum ada → default.
    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Simpan ke disk (buat folder bila perlu). Rotasi daily_log 90 hari.
    pub fn save(&mut self) {
        self.rotate_log();
        let Some(path) = Self::path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, json);
        }
    }

    /// Simpan hanya 90 entry tanggal terakhir.
    fn rotate_log(&mut self) {
        const KEEP: usize = 90;
        while self.daily_log.len() > KEEP {
            // BTreeMap terurut menaik → buang yang paling lama (paling kecil).
            if let Some(oldest) = self.daily_log.keys().next().cloned() {
                self.daily_log.remove(&oldest);
            } else {
                break;
            }
        }
    }
}
