use std::fmt;

use crate::ports::StoreError;

/// Why a save did nothing. The catalog is unchanged and the draft stays open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveError {
    /// No draft is open.
    NoDraft,
    /// A field is invalid; the draft now shows its field errors.
    Invalid,
    /// The store could not write the recipe.
    Store(StoreError),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDraft => write!(f, "No recipe is being edited"),
            Self::Invalid => write!(f, "Fix the highlighted fields"),
            Self::Store(e) => write!(f, "Failed to save recipe: {e}"),
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Store(e) => Some(e),
            Self::NoDraft | Self::Invalid => None,
        }
    }
}
