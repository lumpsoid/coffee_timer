use std::fmt;

use crate::model::RecipeId;
use crate::ports::StoreError;

/// Why moving a recipe in the list did nothing. The catalog is unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveError {
    /// The catalog holds no recipe with this id.
    UnknownRecipe(RecipeId),
    /// The store could not write the new order.
    Store(StoreError),
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownRecipe(id) => write!(f, "No recipe with id {}", id.as_str()),
            Self::Store(e) => write!(f, "Failed to save the order: {e}"),
        }
    }
}

impl std::error::Error for MoveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Store(e) => Some(e),
            Self::UnknownRecipe(_) => None,
        }
    }
}

impl From<StoreError> for MoveError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}
