/// A one-shot platform instruction, already decided by a controller. The
/// shell carries it out and keeps nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Vibrate {
        ms: u32,
    },
    PlayStepSound,
    /// Keep the screen awake (`true`) or let it sleep again (`false`).
    KeepScreenOn(bool),
}
