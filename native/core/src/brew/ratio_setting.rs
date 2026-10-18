use crate::model::BrewRatio;

/// The ratio the brew page shows for the open recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatioSetting {
    /// The recipe's own ratio.
    Recipe(BrewRatio),
    /// A ratio changed on the brew page; it differs from the recipe's.
    Adjusted(BrewRatio),
}

impl RatioSetting {
    /// The ratio in effect.
    pub fn ratio(self) -> BrewRatio {
        match self {
            Self::Recipe(ratio) | Self::Adjusted(ratio) => ratio,
        }
    }

    /// The ratio changed on the brew page, if any.
    pub fn adjusted(self) -> Option<BrewRatio> {
        match self {
            Self::Adjusted(ratio) => Some(ratio),
            Self::Recipe(_) => None,
        }
    }
}
