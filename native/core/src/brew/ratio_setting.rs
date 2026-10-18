use crate::model::BrewRatio;

/// The ratio the brew page shows for the open recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatioSetting {
    /// The recipe has no coffee, so it has no ratio to change.
    NoCoffee,
    /// The recipe's own ratio.
    Recipe(BrewRatio),
    /// A ratio changed on the brew page; it differs from the recipe's.
    Adjusted(BrewRatio),
}

impl RatioSetting {
    /// The ratio in effect; none without coffee.
    pub fn ratio(self) -> Option<BrewRatio> {
        match self {
            Self::NoCoffee => None,
            Self::Recipe(ratio) | Self::Adjusted(ratio) => Some(ratio),
        }
    }

    /// The ratio changed on the brew page, if any.
    pub fn adjusted(self) -> Option<BrewRatio> {
        match self {
            Self::Adjusted(ratio) => Some(ratio),
            Self::NoCoffee | Self::Recipe(_) => None,
        }
    }
}
