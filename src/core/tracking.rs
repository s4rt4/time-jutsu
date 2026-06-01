//! Logic time-tracking per proyek — wall-clock, akumulasi per hari.

use std::collections::BTreeMap;
use std::time::Instant;

use chrono::{Duration, Local};
use serde::{Deserialize, Serialize};

fn today_key() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Project {
    pub name: String,
    /// tanggal "YYYY-MM-DD" → detik tercatat.
    pub log: BTreeMap<String, u64>,
}

impl Project {
    pub fn today_secs(&self) -> u64 {
        self.log.get(&today_key()).copied().unwrap_or(0)
    }
    pub fn week_secs(&self) -> u64 {
        let today = Local::now().date_naive();
        (0..7)
            .map(|i| {
                let k = (today - Duration::days(i)).format("%Y-%m-%d").to_string();
                self.log.get(&k).copied().unwrap_or(0)
            })
            .sum()
    }
}

/// State Tab Tracking: daftar proyek + sesi berjalan (satu aktif).
pub struct Tracker {
    pub projects: Vec<Project>,
    pub input_name: String,
    active: Option<usize>,
    started: Option<Instant>,
}

impl Default for Tracker {
    fn default() -> Self {
        Self {
            projects: Vec::new(),
            input_name: String::new(),
            active: None,
            started: None,
        }
    }
}

impl Tracker {
    pub fn add(&mut self) {
        let n = self.input_name.trim();
        if n.is_empty() {
            return;
        }
        self.projects.push(Project {
            name: n.to_string(),
            log: BTreeMap::new(),
        });
        self.input_name.clear();
    }

    pub fn is_active(&self, i: usize) -> bool {
        self.active == Some(i)
    }
    pub fn any_active(&self) -> bool {
        self.active.is_some()
    }

    /// Commit detik sesi berjalan ke log hari ini, lalu reset titik mulai.
    pub fn flush(&mut self) {
        if let (Some(i), Some(t)) = (self.active, self.started) {
            let secs = t.elapsed().as_secs();
            if secs > 0 {
                *self.projects[i].log.entry(today_key()).or_insert(0) += secs;
            }
            self.started = Some(Instant::now());
        }
    }

    pub fn toggle(&mut self, i: usize) {
        if self.active == Some(i) {
            self.flush();
            self.active = None;
            self.started = None;
        } else {
            self.flush(); // commit proyek sebelumnya (jika ada)
            self.active = Some(i);
            self.started = Some(Instant::now());
        }
    }

    /// Detik hari ini untuk proyek i, termasuk sesi yang sedang berjalan.
    pub fn live_today(&self, i: usize) -> u64 {
        let base = self.projects[i].today_secs();
        if self.active == Some(i) {
            base + self.started.map(|t| t.elapsed().as_secs()).unwrap_or(0)
        } else {
            base
        }
    }

    pub fn remove(&mut self, i: usize) {
        match self.active {
            Some(a) if a == i => {
                self.active = None;
                self.started = None;
            }
            Some(a) if a > i => self.active = Some(a - 1),
            _ => {}
        }
        self.projects.remove(i);
    }

    /// Nama proyek yang sedang ditrack (untuk Dashboard).
    pub fn active_name(&self) -> Option<&str> {
        self.active.map(|i| self.projects[i].name.as_str())
    }
}
