use std::fmt;

use crate::ports::StoreError;

/// Why a delete did nothing. The catalog is unchanged and the draft stays
/// open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteError {
    /// No draft is open.
    NoDraft,
    /// The draft is a new recipe: there is nothing to delete.
    NotSaved,
    /// The store could not delete the recipe.
    Store(StoreError),
}

impl fmt::Display for DeleteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDraft => write!(f, "No recipe is being edited"),
            Self::NotSaved => write!(f, "A new recipe has nothing to delete"),
            Self::Store(e) => write!(f, "Failed to delete recipe: {e}"),
        }
    }
}

impl std::error::Error for DeleteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Store(e) => Some(e),
            Self::NoDraft | Self::NotSaved => None,
        }
    }
}
