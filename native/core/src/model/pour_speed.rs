use crate::model::{ModelError, StepDuration, WaterAmount};

/// How fast a recipe pours, in grams of water per second, kept in tenths:
/// `4.5 g/s` is 45. Always within `MIN_TENTHS..=MAX_TENTHS`.
///
/// The editor uses it to work out how long a step takes to pour its water
/// ([`PourSpeed::duration_of`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PourSpeed(u32);

impl PourSpeed {
    /// `0.5 g/s`, a slow trickle.
    pub const MIN_TENTHS: u32 = 5;
    /// `30 g/s`, faster than any kettle pours.
    pub const MAX_TENTHS: u32 = 300;
    /// The slowest speed, `0.5 g/s`.
    pub const MIN: Self = Self(Self::MIN_TENTHS);
    /// The fastest speed, `30 g/s`.
    pub const MAX: Self = Self(Self::MAX_TENTHS);
    /// `5 g/s`: a steady gooseneck-kettle pour. What a recipe pours at until
    /// it is given its own speed.
    pub const DEFAULT: Self = Self(50);

    /// `grams_per_second` rounded to a tenth, then the bounds. Not a finite
    /// number is out of range.
    pub fn try_parse(grams_per_second: f64) -> Result<Self, ModelError> {
        let tenths = (grams_per_second * 10.0).round();
        let in_range = f64::from(Self::MIN_TENTHS)..=f64::from(Self::MAX_TENTHS);
        if !in_range.contains(&tenths) {
            return Err(ModelError::PourSpeedOutOfRange(
                grams_per_second.to_string(),
            ));
        }
        // In range, so a whole number of tenths that fits a `u32`.
        Ok(Self(tenths as u32))
    }

    /// Parses typed text strictly: trimmed, then a decimal number (`5`,
    /// `4.5`), rounded to a tenth, then the bounds. Not a number is
    /// [`ModelError::PourSpeedNotANumber`] (`Enter g/s`); out of range is
    /// [`ModelError::PourSpeedOutOfRange`] (`0.5–30 g/s`).
    pub fn try_parse_text(raw: &str) -> Result<Self, ModelError> {
        let grams_per_second = raw
            .trim()
            .parse()
            .map_err(|_| ModelError::PourSpeedNotANumber(raw.to_owned()))?;
        Self::try_parse(grams_per_second)
    }

    /// Clamps `tenths` into `MIN_TENTHS..=MAX_TENTHS`: bad storage never
    /// blocks the app.
    pub fn clamped_tenths(tenths: i64) -> Self {
        let bounded = tenths.clamp(i64::from(Self::MIN_TENTHS), i64::from(Self::MAX_TENTHS));
        Self(u32::try_from(bounded).unwrap_or(Self::MAX_TENTHS))
    }

    pub fn tenths(self) -> u32 {
        self.0
    }

    /// Grams per second, `4.5`.
    pub fn grams_per_second(self) -> f64 {
        f64::from(self.0) / 10.0
    }

    /// How long pouring `water` takes at this speed, rounded up to a whole
    /// second (a pour is never cut short), at most a step's longest.
    pub fn duration_of(self, water: WaterAmount) -> StepDuration {
        let seconds = (u64::from(water.grams()) * 10).div_ceil(u64::from(self.0));
        StepDuration::clamped(i64::try_from(seconds).unwrap_or(i64::MAX))
    }

    /// The speed as typed text: `5`, or to one decimal when it is not
    /// whole, `4.5`.
    pub fn text(self) -> String {
        match self.0 % 10 {
            0 => format!("{}", self.0 / 10),
            fraction => format!("{}.{fraction}", self.0 / 10),
        }
    }

    /// `5 g/s`, `4.5 g/s`.
    pub fn label(self) -> String {
        format!("{} g/s", self.text())
    }
}

impl Default for PourSpeed {
    fn default() -> Self {
        Self::DEFAULT
    }
}
