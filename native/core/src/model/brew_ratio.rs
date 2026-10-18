/// Grams of water per gram of coffee, kept in tenths: `1:15.5` is 155.
///
/// A recipe's own ratio can be any value. A ratio adjusted on the brew page
/// moves in [`BrewRatio::STEP_TENTHS`] steps and stays within
/// `MIN_TENTHS..=MAX_TENTHS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrewRatio(u32);

impl BrewRatio {
    /// `1:1`, the most coffee an adjustment reaches.
    pub const MIN_TENTHS: u32 = 10;
    /// `1:30`, the most water an adjustment reaches.
    pub const MAX_TENTHS: u32 = 300;
    /// One adjustment step, `0.5`.
    pub const STEP_TENTHS: u32 = 5;
    /// What stands in for the ratio of a recipe without coffee.
    pub const NONE_LABEL: &'static str = "—";

    /// `water` per gram of `coffee`, rounded half up to a tenth. None
    /// without coffee.
    pub fn of(coffee_grams: u32, water_grams: u32) -> Option<Self> {
        let coffee = u64::from(coffee_grams);
        if coffee == 0 {
            return None;
        }
        let tenths = (u64::from(water_grams) * 20 + coffee) / (coffee * 2);
        Some(Self(u32::try_from(tenths).unwrap_or(u32::MAX)))
    }

    /// One step more water: up to the next multiple of the step, at most
    /// [`BrewRatio::MAX_TENTHS`].
    pub fn more_water(self) -> Self {
        let next = (self.0 / Self::STEP_TENTHS).saturating_add(1) * Self::STEP_TENTHS;
        Self(next.clamp(Self::MIN_TENTHS, Self::MAX_TENTHS))
    }

    /// One step less water: down to the previous multiple of the step, at
    /// least [`BrewRatio::MIN_TENTHS`].
    pub fn less_water(self) -> Self {
        let previous = self.0.div_ceil(Self::STEP_TENTHS).saturating_sub(1) * Self::STEP_TENTHS;
        Self(previous.clamp(Self::MIN_TENTHS, Self::MAX_TENTHS))
    }

    /// The water for `coffee_grams` at this ratio, rounded half up to a gram.
    pub fn water_grams(self, coffee_grams: u32) -> u32 {
        let tenth_grams = u64::from(coffee_grams) * u64::from(self.0);
        u32::try_from((tenth_grams + 5) / 10).unwrap_or(u32::MAX)
    }

    /// `1:5`, or to one decimal when it is not whole, `1:2.5`.
    pub fn label(self) -> String {
        match self.0 % 10 {
            0 => format!("1:{}", self.0 / 10),
            fraction => format!("1:{}.{fraction}", self.0 / 10),
        }
    }
}
