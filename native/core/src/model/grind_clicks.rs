use crate::model::ModelError;

/// Where to set the grinder for a recipe, in whole clicks from zero (fully
/// closed), always within `MIN..=MAX`. A recipe without one has no grind
/// setting.
///
/// Strict entry for typed text ([`GrindClicks::try_parse_optional_text`]),
/// lenient entry for stored rows ([`GrindClicks::clamped`]); the bounds live
/// here only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GrindClicks(u32);

impl GrindClicks {
    pub const MIN: u32 = 0;
    /// Two digits: every click a hand grinder's dial has.
    pub const MAX: u32 = 99;

    /// Rejects `clicks` outside `MIN..=MAX`.
    pub fn try_parse(clicks: i64) -> Result<Self, ModelError> {
        match u32::try_from(clicks) {
            Ok(c) if (Self::MIN..=Self::MAX).contains(&c) => Ok(Self(c)),
            _ => Err(ModelError::GrindClicksOutOfRange(clicks)),
        }
    }

    /// Parses typed text strictly: trimmed, then blank is none, anything else
    /// a decimal integer within the bounds. Not an integer is
    /// [`ModelError::GrindClicksNotANumber`] (`Enter clicks`); out of range
    /// is [`ModelError::GrindClicksOutOfRange`] (`0–99 clicks`).
    pub fn try_parse_optional_text(raw: &str) -> Result<Option<Self>, ModelError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        let clicks = trimmed
            .parse()
            .map_err(|_| ModelError::GrindClicksNotANumber(raw.to_owned()))?;
        Self::try_parse(clicks).map(Some)
    }

    /// Clamps `clicks` into `MIN..=MAX`: bad storage never blocks the app.
    pub fn clamped(clicks: i64) -> Self {
        let bounded = clicks.clamp(i64::from(Self::MIN), i64::from(Self::MAX));
        Self(u32::try_from(bounded).unwrap_or(Self::MAX))
    }

    pub fn clicks(self) -> u32 {
        self.0
    }

    /// The bare count, `22`: shown under a `Clicks` caption.
    pub fn label(self) -> String {
        self.0.to_string()
    }
}
