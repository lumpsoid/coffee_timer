use std::fmt;

use crate::model::RecipeId;
use crate::ports::StoreError;

/// Why an undo of a removal did nothing. The catalog is unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestoreError {
    /// The recipe is not the one removed last, or is already back.
    NothingToUndo(RecipeId),
    /// The store could not put the recipe back; the undo stays on offer.
    Store(StoreError),
}

impl fmt::Display for RestoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NothingToUndo(_) => write!(f, "Nothing to undo"),
            Self::Store(e) => write!(f, "Failed to restore recipe: {e}"),
        }
    }
}

impl std::error::Error for RestoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Store(e) => Some(e),
            Self::NothingToUndo(_) => None,
        }
    }
}

impl From<StoreError> for RestoreError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}
