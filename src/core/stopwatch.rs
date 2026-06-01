//! Logic stopwatch + lap — murni, wall-clock based.

use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Stopwatch {
    started_at: Option<Instant>,
    accumulated: Duration,
    pub laps: Vec<Duration>,
}

impl Stopwatch {
    pub fn elapsed(&self) -> Duration {
        self.accumulated + self.started_at.map(|t| t.elapsed()).unwrap_or(Duration::ZERO)
    }

    pub fn is_running(&self) -> bool {
        self.started_at.is_some()
    }

    pub fn toggle(&mut self) {
        if self.is_running() {
            self.accumulated = self.elapsed();
            self.started_at = None;
        } else {
            self.started_at = Some(Instant::now());
        }
    }

    pub fn lap(&mut self) {
        if self.is_running() || !self.accumulated.is_zero() {
            self.laps.push(self.elapsed());
        }
    }

    pub fn reset(&mut self) {
        self.started_at = None;
        self.accumulated = Duration::ZERO;
        self.laps.clear();
    }

    /// Format teks lap log untuk di-export.
    pub fn export_text(&self) -> String {
        let mut s = String::from("Time-Jutsu — Stopwatch Lap Log\n\n");
        for (i, lap) in self.laps.iter().enumerate() {
            s.push_str(&format!("Lap {:>2}: {}\n", i + 1, fmt_sw(*lap)));
        }
        s.push_str(&format!("\nTotal: {}\n", fmt_sw(self.elapsed())));
        s
    }
}

/// Format stopwatch MM:SS.cs (centisecond) atau HH:MM:SS.cs bila ≥ 1 jam.
pub fn fmt_sw(d: Duration) -> String {
    let total_cs = d.as_millis() / 10;
    let cs = total_cs % 100;
    let s = total_cs / 100;
    let (h, m, sec) = (s / 3600, (s % 3600) / 60, s % 60);
    if h > 0 {
        format!("{:02}:{:02}:{:02}.{:02}", h, m, sec, cs)
    } else {
        format!("{:02}:{:02}.{:02}", m, sec, cs)
    }
}
