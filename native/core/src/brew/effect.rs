use crate::brew::BrewNotice;
use crate::model::Sound;

/// A one-shot platform instruction, already decided by a controller. The
/// shell carries it out and keeps nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Vibrate {
        ms: u32,
    },
    /// A buzz pattern: alternating pause and buzz lengths in ms, starting
    /// with a pause (`[0, 400, 200, 400]` is two long buzzes), and the
    /// strength of each entry, 1–255 (0 for the pauses), as Android's
    /// waveform takes them. No strengths buzzes at the device's default; a
    /// device that cannot vary its strength ignores them.
    VibratePattern {
        pattern_ms: Vec<u32>,
        strengths: Vec<u8>,
    },
    PlaySound {
        sound: Sound,
    },
    /// Keep the screen awake (`true`) or let it sleep again (`false`).
    KeepScreenOn(bool),
    /// Show or refresh the ongoing notification of a running brew. While it
    /// shows, the platform keeps the brew ticking in the background.
    ShowBrewNotification(BrewNotice),
    /// Remove the brew notification; the brew no longer runs.
    HideBrewNotification,
}
