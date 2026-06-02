//! Logic Pomodoro — murni, tanpa egui. Timing berbasis wall-clock (`Instant`),
//! bukan akumulasi per-frame, supaya akurat walau frame rate tidak konsisten.

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Focus,
    Break,
    LongBreak,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Phase::Focus => crate::i18n::t("Fokus", "Focus"),
            Phase::Break => crate::i18n::t("Istirahat", "Break"),
            Phase::LongBreak => crate::i18n::t("Istirahat Panjang", "Long Break"),
        }
    }
}

/// Istirahat panjang tiap N sesi fokus (teknik Pomodoro klasik).
const LONG_BREAK_EVERY: u32 = 4;

/// Event saat fase berganti — dipakai untuk memicu notifikasi & suara.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseSwitch {
    ToBreak,
    ToFocus,
}

/// Preset durasi (menit fokus / menit break).
pub const PRESETS: [(u32, u32); 2] = [(25, 5), (50, 10)];

pub struct Pomodoro {
    pub work_minutes: u32,
    pub break_minutes: u32,
    pub long_break_minutes: u32,
    pub sound_enabled: bool,

    phase: Phase,
    running: bool,
    /// Waktu mulai segmen berjalan saat ini (None = sedang pause).
    started_at: Option<Instant>,
    /// Durasi yang sudah terkumpul sebelum pause terakhir.
    accumulated: Duration,
    sessions_completed: u32,
}

impl Default for Pomodoro {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            break_minutes: 5,
            long_break_minutes: 15,
            sound_enabled: true,
            phase: Phase::Focus,
            running: false,
            started_at: None,
            accumulated: Duration::ZERO,
            sessions_completed: 0,
        }
    }
}

impl Pomodoro {
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Nomor sesi fokus yang sedang/akan berjalan hari ini (1-based).
    pub fn current_session(&self) -> u32 {
        self.sessions_completed + 1
    }

    fn phase_total(&self) -> Duration {
        let mins = match self.phase {
            Phase::Focus => self.work_minutes,
            Phase::Break => self.break_minutes,
            Phase::LongBreak => self.long_break_minutes,
        };
        Duration::from_secs(mins as u64 * 60)
    }

    fn elapsed(&self) -> Duration {
        self.accumulated
            + self
                .started_at
                .map(|t| t.elapsed())
                .unwrap_or(Duration::ZERO)
    }

    /// Sisa waktu fase saat ini.
    pub fn remaining(&self) -> Duration {
        self.phase_total().saturating_sub(self.elapsed())
    }

    /// Fraksi progress 0.0..=1.0 (untuk arc).
    pub fn progress(&self) -> f32 {
        let total = self.phase_total().as_secs_f32();
        if total <= 0.0 {
            return 0.0;
        }
        (self.elapsed().as_secs_f32() / total).clamp(0.0, 1.0)
    }

    pub fn start(&mut self) {
        if !self.running {
            self.started_at = Some(Instant::now());
            self.running = true;
        }
    }

    pub fn pause(&mut self) {
        if self.running {
            self.accumulated = self.elapsed();
            self.started_at = None;
            self.running = false;
        }
    }

    pub fn toggle(&mut self) {
        if self.running {
            self.pause();
        } else {
            self.start();
        }
    }

    /// Reset ke awal fase Fokus (sesi hari ini tidak dihapus).
    pub fn reset(&mut self) {
        self.phase = Phase::Focus;
        self.running = false;
        self.started_at = None;
        self.accumulated = Duration::ZERO;
    }

    pub fn set_preset(&mut self, work: u32, brk: u32) {
        self.work_minutes = work.max(1);
        self.break_minutes = brk.max(1);
        self.reset();
    }

    /// Dipanggil tiap frame. Kembalikan event bila fase berganti (auto-switch).
    pub fn tick(&mut self) -> Option<PhaseSwitch> {
        if !self.running {
            return None;
        }
        if self.elapsed() < self.phase_total() {
            return None;
        }
        let ev = match self.phase {
            Phase::Focus => {
                self.sessions_completed += 1;
                // tiap LONG_BREAK_EVERY sesi → istirahat panjang
                self.phase = if self.sessions_completed % LONG_BREAK_EVERY == 0 {
                    Phase::LongBreak
                } else {
                    Phase::Break
                };
                PhaseSwitch::ToBreak
            }
            Phase::Break | Phase::LongBreak => {
                self.phase = Phase::Focus;
                PhaseSwitch::ToFocus
            }
        };
        // auto-start fase berikutnya
        self.accumulated = Duration::ZERO;
        self.started_at = Some(Instant::now());
        Some(ev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaining_starts_full() {
        let p = Pomodoro::default();
        assert_eq!(p.remaining(), Duration::from_secs(25 * 60));
        assert_eq!(p.progress(), 0.0);
    }

    #[test]
    fn pause_freezes_elapsed() {
        let mut p = Pomodoro::default();
        p.start();
        std::thread::sleep(Duration::from_millis(20));
        p.pause();
        let a = p.elapsed();
        std::thread::sleep(Duration::from_millis(20));
        // setelah pause, elapsed tidak bertambah
        assert_eq!(a, p.elapsed());
    }

    #[test]
    fn preset_resets_to_focus() {
        let mut p = Pomodoro::default();
        p.start();
        p.set_preset(50, 10);
        assert_eq!(p.phase(), Phase::Focus);
        assert!(!p.is_running());
        assert_eq!(p.work_minutes, 50);
    }
}
