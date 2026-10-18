/// The quick buttons on a step card that shorten or lengthen its duration
/// by a few seconds, instead of retyping it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationNudge {
    MinusLarge,
    MinusSmall,
    PlusSmall,
    PlusLarge,
}

impl DurationNudge {
    /// The small nudge, in seconds.
    pub const SMALL_SECONDS: i64 = 5;
    /// The large nudge, in seconds.
    pub const LARGE_SECONDS: i64 = 15;

    /// Every nudge, in button order: −15 s, −5 s, +5 s, +15 s.
    pub const ALL: [Self; 4] = [
        Self::MinusLarge,
        Self::MinusSmall,
        Self::PlusSmall,
        Self::PlusLarge,
    ];

    /// Seconds added to the duration; negative shortens it.
    pub fn delta_seconds(self) -> i64 {
        match self {
            Self::MinusLarge => -Self::LARGE_SECONDS,
            Self::MinusSmall => -Self::SMALL_SECONDS,
            Self::PlusSmall => Self::SMALL_SECONDS,
            Self::PlusLarge => Self::LARGE_SECONDS,
        }
    }

    /// The button text, `−15` or `+5`: signed seconds, a true minus sign.
    pub fn button_label(self) -> String {
        let delta = self.delta_seconds();
        let sign = if delta < 0 { '−' } else { '+' };
        format!("{sign}{}", delta.unsigned_abs())
    }

    /// What a screen reader says for the button, `Subtract 15 seconds`.
    pub fn description(self) -> String {
        let delta = self.delta_seconds();
        let verb = if delta < 0 { "Subtract" } else { "Add" };
        format!("{verb} {} seconds", delta.unsigned_abs())
    }
}
