use crate::model::ModelError;

/// How much ground coffee a recipe uses, in whole grams, always within
/// `MIN_GRAMS..=MAX_GRAMS`.
///
/// Strict entry for typed text ([`CoffeeDose::try_parse_text`]), lenient
/// entry for stored rows ([`CoffeeDose::clamped`]); the bounds live here
/// only. Never zero, so every recipe has a ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CoffeeDose(u32);

impl CoffeeDose {
    /// The least coffee a recipe can use.
    pub const MIN_GRAMS: u32 = 1;
    /// The most coffee a recipe can use: a large batch brew. It also holds
    /// every dose the step-count formula of schema v1 gave (up to 20 steps).
    pub const MAX_GRAMS: u32 = 200;

    /// Rejects `grams` outside `MIN_GRAMS..=MAX_GRAMS`.
    pub fn try_parse(grams: i64) -> Result<Self, ModelError> {
        match u32::try_from(grams) {
            Ok(g) if (Self::MIN_GRAMS..=Self::MAX_GRAMS).contains(&g) => Ok(Self(g)),
            _ => Err(ModelError::CoffeeDoseOutOfRange(grams)),
        }
    }

    /// Parses typed text strictly: trimmed, then a decimal integer, then the
    /// bounds. Not an integer is [`ModelError::CoffeeDoseNotANumber`]
    /// (`Enter grams`); out of range is [`ModelError::CoffeeDoseOutOfRange`]
    /// (`1–200 g`).
    pub fn try_parse_text(raw: &str) -> Result<Self, ModelError> {
        let grams = raw
            .trim()
            .parse()
            .map_err(|_| ModelError::CoffeeDoseNotANumber(raw.to_owned()))?;
        Self::try_parse(grams)
    }

    /// Clamps `grams` into `MIN_GRAMS..=MAX_GRAMS`: bad storage never blocks
    /// the app.
    pub fn clamped(grams: i64) -> Self {
        let bounded = grams.clamp(i64::from(Self::MIN_GRAMS), i64::from(Self::MAX_GRAMS));
        Self(u32::try_from(bounded).unwrap_or(Self::MAX_GRAMS))
    }

    pub fn grams(self) -> u32 {
        self.0
    }
}
