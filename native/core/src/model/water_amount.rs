use crate::model::ModelError;

/// An amount of brew water, in whole grams, always within
/// `MIN_GRAMS..=MAX_GRAMS`. A recipe's total water is one; it is its own
/// type so other amounts of water can reuse its bounds and parsing.
///
/// Strict entry for typed text ([`WaterAmount::try_parse_text`]), lenient
/// entry for stored rows ([`WaterAmount::clamped`]); the bounds live here
/// only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WaterAmount(u32);

impl WaterAmount {
    /// The least water an amount can be.
    pub const MIN_GRAMS: u32 = 1;
    /// The most water an amount can be: a two-litre batch brew. It also
    /// holds every amount the step-count formula of schema v1 gave (up to
    /// 40 steps).
    pub const MAX_GRAMS: u32 = 2000;

    /// Rejects `grams` outside `MIN_GRAMS..=MAX_GRAMS`.
    pub fn try_parse(grams: i64) -> Result<Self, ModelError> {
        match u32::try_from(grams) {
            Ok(g) if (Self::MIN_GRAMS..=Self::MAX_GRAMS).contains(&g) => Ok(Self(g)),
            _ => Err(ModelError::WaterAmountOutOfRange(grams)),
        }
    }

    /// Parses typed text strictly: trimmed, then a decimal integer, then the
    /// bounds. Not an integer is [`ModelError::WaterAmountNotANumber`]
    /// (`Enter grams`); out of range is
    /// [`ModelError::WaterAmountOutOfRange`] (`1–2000 g`).
    pub fn try_parse_text(raw: &str) -> Result<Self, ModelError> {
        let grams = raw
            .trim()
            .parse()
            .map_err(|_| ModelError::WaterAmountNotANumber(raw.to_owned()))?;
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
