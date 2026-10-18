use std::fmt;

use crate::ports::StoreError;
use crate::transfer::DocumentError;

/// Why an import added nothing. The catalog is unchanged either way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    /// The text is not an importable document.
    Document(DocumentError),
    /// The store could not write the recipes; none were added.
    Store(StoreError),
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Document(e) => write!(f, "Can't import: {e}"),
            Self::Store(e) => write!(f, "Failed to import recipes: {e}"),
        }
    }
}

impl std::error::Error for ImportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Document(e) => Some(e),
            Self::Store(e) => Some(e),
        }
    }
}

impl From<DocumentError> for ImportError {
    fn from(e: DocumentError) -> Self {
        Self::Document(e)
    }
}

impl From<StoreError> for ImportError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}
