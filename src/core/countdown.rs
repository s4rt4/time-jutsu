//! Logic countdown multi-timer — murni, wall-clock based.

use std::time::{Duration, Instant};

use crate::core::deadline::Deadline;
use crate::core::stopwatch::Stopwatch;

/// Sub-mode Tab Timer.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TimerMode {
    Countdown,
    Stopwatch,
    Deadline,
}

pub struct Countdown {
    pub label: String,
    total: Duration,
    started_at: Option<Instant>,
    accumulated: Duration,
    pub finished: bool,
}

impl Countdown {
    pub fn new(label: String, total: Duration) -> Self {
        Self {
            label,
            total,
            started_at: Some(Instant::now()), // langsung jalan saat dibuat
            accumulated: Duration::ZERO,
            finished: false,
        }
    }

    fn elapsed(&self) -> Duration {
        self.accumulated + self.started_at.map(|t| t.elapsed()).unwrap_or(Duration::ZERO)
    }

    pub fn remaining(&self) -> Duration {
        self.total.saturating_sub(self.elapsed())
    }

    pub fn progress(&self) -> f32 {
        let t = self.total.as_secs_f32();
        if t <= 0.0 {
            return 1.0;
        }
        (self.elapsed().as_secs_f32() / t).clamp(0.0, 1.0)
    }

    pub fn is_running(&self) -> bool {
        self.started_at.is_some() && !self.finished
    }

    pub fn toggle(&mut self) {
        if self.finished {
            return;
        }
        if self.is_running() {
            self.accumulated = self.elapsed();
            self.started_at = None;
        } else {
            self.started_at = Some(Instant::now());
        }
    }

    /// Kembalikan true SEKALI saat countdown mencapai nol.
    fn tick(&mut self) -> bool {
        if self.finished {
            return false;
        }
        if self.is_running() && self.remaining().is_zero() {
            self.finished = true;
            self.started_at = None;
            true
        } else {
            false
        }
    }
}

/// State Tab Timer: countdown + stopwatch + field input.
pub struct TimerState {
    pub mode: TimerMode,
    pub countdowns: Vec<Countdown>,
    pub stopwatch: Stopwatch,
    pub input_label: String,
    pub input_h: u32,
    pub input_m: u32,
    pub input_s: u32,
    // deadline
    pub deadlines: Vec<Deadline>,
    pub dl_label: String,
    pub dl_days: i64,
    pub dl_hours: i64,
}

impl Default for TimerState {
    fn default() -> Self {
        Self {
            mode: TimerMode::Countdown,
            countdowns: Vec::new(),
            stopwatch: Stopwatch::default(),
            input_label: String::new(),
            input_h: 0,
            input_m: 5,
            input_s: 0,
            deadlines: Vec::new(),
            dl_label: String::new(),
            dl_days: 1,
            dl_hours: 0,
        }
    }
}

impl TimerState {
    /// Total detik dari input.
    pub fn input_total_secs(&self) -> u64 {
        self.input_h as u64 * 3600 + self.input_m as u64 * 60 + self.input_s as u64
    }

    /// Tambah countdown dari input (abaikan jika durasi 0).
    pub fn add(&mut self) {
        let secs = self.input_total_secs();
        if secs == 0 {
            return;
        }
        let label = if self.input_label.trim().is_empty() {
            format!("Timer {}", self.countdowns.len() + 1)
        } else {
            self.input_label.trim().to_string()
        };
        self.countdowns
            .push(Countdown::new(label, Duration::from_secs(secs)));
        self.input_label.clear();
    }

    /// Tick semua countdown; kembalikan label yang baru saja selesai.
    pub fn tick(&mut self) -> Vec<String> {
        let mut done = Vec::new();
        for c in &mut self.countdowns {
            if c.tick() {
                done.push(c.label.clone());
            }
        }
        done
    }

    /// Tambah deadline relatif: sekarang + dl_days hari + dl_hours jam.
    pub fn add_deadline(&mut self) {
        if self.dl_days <= 0 && self.dl_hours <= 0 {
            return;
        }
        let target = chrono::Local::now().naive_local()
            + chrono::Duration::days(self.dl_days.max(0))
            + chrono::Duration::hours(self.dl_hours.max(0));
        let label = if self.dl_label.trim().is_empty() {
            "Deadline".to_string()
        } else {
            self.dl_label.trim().to_string()
        };
        self.deadlines.push(Deadline { label, target });
        self.dl_label.clear();
    }

    /// Deadline terdekat yang belum lewat (untuk Dashboard).
    pub fn next_deadline(&self) -> Option<&Deadline> {
        self.deadlines
            .iter()
            .filter(|d| !d.is_past())
            .min_by_key(|d| d.target)
    }
}
