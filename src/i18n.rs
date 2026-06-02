//! Lokalisasi sederhana: Bahasa Indonesia / English.
//! Bahasa aktif disimpan di state global (atomic) dan dipilih lewat `t(id, en)`.

use std::sync::atomic::{AtomicU8, Ordering};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    #[default]
    Id,
    En,
}

impl Lang {
    pub fn is_id(self) -> bool {
        matches!(self, Lang::Id)
    }
}

static LANG: AtomicU8 = AtomicU8::new(0); // 0 = Id, 1 = En

pub fn set_lang(l: Lang) {
    LANG.store(if l.is_id() { 0 } else { 1 }, Ordering::Relaxed);
}

fn is_id() -> bool {
    LANG.load(Ordering::Relaxed) == 0
}

/// Pilih string sesuai bahasa aktif: `t("Indonesia", "English")`.
pub fn t(id: &'static str, en: &'static str) -> &'static str {
    if is_id() {
        id
    } else {
        en
    }
}
