use crate::brew::BrewNotice;

/// A one-shot platform instruction, already decided by a controller. The
/// shell carries it out and keeps nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    Vibrate {
        ms: u32,
    },
    PlayStepSound,
    /// Keep the screen awake (`true`) or let it sleep again (`false`).
    KeepScreenOn(bool),
    /// Show or refresh the ongoing notification of a running brew. While it
    /// shows, the platform keeps the brew ticking in the background.
    ShowBrewNotification(BrewNotice),
    /// Remove the brew notification; the brew no longer runs.
    HideBrewNotification,
}
