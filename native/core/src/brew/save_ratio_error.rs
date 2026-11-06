use std::fmt;

use crate::model::WaterScaleError;
use crate::ports::StoreError;

/// Why the brew's changed dose (coffee, ratio) was not saved to its recipe. The brew and
/// the catalog are unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveRatioError {
    /// No recipe is open.
    NoRecipe,
    /// The brew shows the recipe's own coffee and ratio: nothing to save.
    NotAdjusted,
    /// The recipe cannot hold the changed water or a scaled pour.
    OutOfRange(WaterScaleError),
    /// The store could not write the recipe.
    Store(StoreError),
}

impl fmt::Display for SaveRatioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRecipe => write!(f, "No recipe is open"),
            Self::NotAdjusted => write!(f, "The dose is already the recipe's own"),
            Self::OutOfRange(e) => write!(f, "Can't save to the recipe: {e}"),
            Self::Store(e) => write!(f, "Failed to save recipe: {e}"),
        }
    }
}

impl std::error::Error for SaveRatioError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::OutOfRange(e) => Some(e),
            Self::Store(e) => Some(e),
            Self::NoRecipe | Self::NotAdjusted => None,
        }
    }
}

impl From<WaterScaleError> for SaveRatioError {
    fn from(e: WaterScaleError) -> Self {
        Self::OutOfRange(e)
    }
}

impl From<StoreError> for SaveRatioError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}
