//! Logic deadline — hitung mundur ke tanggal/waktu target (berbasis jam dinding).

use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Deadline {
    pub label: String,
    pub target: NaiveDateTime,
}

impl Deadline {
    pub fn remaining(&self) -> Option<chrono::Duration> {
        let now = Local::now().naive_local();
        let d = self.target - now;
        (d.num_seconds() > 0).then_some(d)
    }

    pub fn is_past(&self) -> bool {
        self.remaining().is_none()
    }

    /// "3 hari 4 jam" / "5 jam 12 mnt" / "20 menit" / "Lewat".
    pub fn remaining_text(&self) -> String {
        match self.remaining() {
            Some(d) => {
                let (days, h, m) = (d.num_days(), d.num_hours() % 24, d.num_minutes() % 60);
                if days > 0 {
                    format!("{days} hari {h} jam")
                } else if h > 0 {
                    format!("{h} jam {m} mnt")
                } else {
                    format!("{m} menit")
                }
            }
            None => "Lewat".to_string(),
        }
    }

    pub fn target_text(&self) -> String {
        self.target.format("%d %b %Y · %H:%M").to_string()
    }
}
