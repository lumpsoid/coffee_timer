use std::fmt;

use crate::model::RecipeId;
use crate::ports::StoreError;

/// Why pinning or unpinning a recipe did nothing. The catalog is unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinError {
    /// The catalog holds no recipe with this id.
    UnknownRecipe(RecipeId),
    /// The store could not write the pin.
    Store(StoreError),
}

impl fmt::Display for PinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownRecipe(id) => write!(f, "No recipe with id {}", id.as_str()),
            Self::Store(e) => write!(f, "Failed to save the pin: {e}"),
        }
    }
}

impl std::error::Error for PinError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Store(e) => Some(e),
            Self::UnknownRecipe(_) => None,
        }
    }
}

impl From<StoreError> for PinError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}
