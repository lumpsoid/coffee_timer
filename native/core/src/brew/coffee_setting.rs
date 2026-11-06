use crate::model::CoffeeDose;

/// The coffee the brew page shows for the open recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoffeeSetting {
    /// The recipe's own coffee.
    Recipe(CoffeeDose),
    /// Coffee changed on the brew page; it differs from the recipe's.
    Adjusted(CoffeeDose),
}

impl CoffeeSetting {
    /// The coffee in effect.
    pub fn dose(self) -> CoffeeDose {
        match self {
            Self::Recipe(dose) | Self::Adjusted(dose) => dose,
        }
    }

    /// The coffee changed on the brew page, if any.
    pub fn adjusted(self) -> Option<CoffeeDose> {
        match self {
            Self::Adjusted(dose) => Some(dose),
            Self::Recipe(_) => None,
        }
    }
}
