//! Wrapper audio (rodio): beep sintetik + nada alarm bundled (.wav di-embed).

use std::io::Cursor;
use std::time::Duration;

use rodio::source::SineWave;
use rodio::{Decoder, OutputStream, Sink, Source};
use serde::{Deserialize, Serialize};

/// Pilihan nada notifikasi/alarm.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AlarmSound {
    #[default]
    Classic,
    Alert,
    Buzzer,
    Rooster,
    Win,
    Beep,
}

impl AlarmSound {
    pub const ALL: [AlarmSound; 6] = [
        AlarmSound::Classic,
        AlarmSound::Alert,
        AlarmSound::Buzzer,
        AlarmSound::Rooster,
        AlarmSound::Win,
        AlarmSound::Beep,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AlarmSound::Classic => "Classic",
            AlarmSound::Alert => "Alert",
            AlarmSound::Buzzer => "Buzzer",
            AlarmSound::Rooster => "Rooster",
            AlarmSound::Win => "Win",
            AlarmSound::Beep => "Beep (sintetik)",
        }
    }

    /// Byte .wav bundled (None untuk Beep → pakai nada sintetik).
    fn bytes(self) -> Option<&'static [u8]> {
        Some(match self {
            AlarmSound::Classic => include_bytes!("../../assets/sounds/classic.ogg"),
            AlarmSound::Alert => include_bytes!("../../assets/sounds/alert.ogg"),
            AlarmSound::Buzzer => include_bytes!("../../assets/sounds/buzzer.ogg"),
            AlarmSound::Rooster => include_bytes!("../../assets/sounds/rooster.ogg"),
            AlarmSound::Win => include_bytes!("../../assets/sounds/win.ogg"),
            AlarmSound::Beep => return None,
        })
    }
}

/// Putar nada terpilih di thread terpisah (non-blocking). Gagal diam-diam.
pub fn play(sound: AlarmSound) {
    std::thread::spawn(move || {
        let Ok((_stream, handle)) = OutputStream::try_default() else {
            return;
        };
        let Ok(sink) = Sink::try_new(&handle) else {
            return;
        };
        match sound.bytes() {
            Some(bytes) => {
                if let Ok(src) = Decoder::new(Cursor::new(bytes)) {
                    sink.append(src);
                }
            }
            None => {
                // beep sintetik dua nada (ding-ding)
                sink.append(
                    SineWave::new(880.0)
                        .take_duration(Duration::from_millis(140))
                        .amplify(0.20),
                );
                sink.append(
                    SineWave::new(1175.0)
                        .take_duration(Duration::from_millis(160))
                        .amplify(0.20),
                );
            }
        }
        sink.sleep_until_end();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_sounds_decode() {
        for s in AlarmSound::ALL {
            if let Some(bytes) = s.bytes() {
                assert!(
                    Decoder::new(Cursor::new(bytes)).is_ok(),
                    "gagal decode nada {:?}",
                    s
                );
            }
        }
    }
}
