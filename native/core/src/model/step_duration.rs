use crate::model::ModelError;

/// A single step's duration in seconds, always within
/// `0..=MAX_STEP_SECONDS`.
///
/// Two entry points, one home for the bound: [`StepDuration::try_parse`] is
/// strict (user input), [`StepDuration::clamped`] is lenient (storage rows).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StepDuration(u32);

impl StepDuration {
    /// Inclusive upper bound for a single step.
    pub const MAX_STEP_SECONDS: u32 = 5999;

    pub const ZERO: Self = Self(0);

    /// Rejects `raw` outside `0..=MAX_STEP_SECONDS`.
    pub fn try_parse(raw: i64) -> Result<Self, ModelError> {
        match u32::try_from(raw) {
            Ok(seconds) if seconds <= Self::MAX_STEP_SECONDS => Ok(Self(seconds)),
            _ => Err(ModelError::StepDurationOutOfRange(raw)),
        }
    }

    /// Clamps `raw` into `0..=MAX_STEP_SECONDS`.
    pub fn clamped(raw: i64) -> Self {
        let bounded = raw.clamp(0, i64::from(Self::MAX_STEP_SECONDS));
        Self(u32::try_from(bounded).unwrap_or(Self::MAX_STEP_SECONDS))
    }

    pub fn seconds(self) -> u32 {
        self.0
    }

    /// `MM:SS`, each part zero-padded to two digits.
    pub fn mmss(self) -> String {
        format!("{:02}:{:02}", self.0 / 60, self.0 % 60)
    }

    /// `Ns`, the step-list trailing label.
    pub fn seconds_label(self) -> String {
        format!("{}s", self.0)
    }
}
