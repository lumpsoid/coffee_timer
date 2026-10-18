use std::fmt;

use crate::model::RecipeId;

/// Why nothing was exported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportError {
    /// The catalog holds no recipe with this id.
    UnknownRecipe(RecipeId),
    /// The document could not be written as JSON (serde's reason).
    Encode(String),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownRecipe(id) => write!(f, "No recipe with id {}", id.as_str()),
            Self::Encode(reason) => write!(f, "Failed to export recipes: {reason}"),
        }
    }
}

impl std::error::Error for ExportError {}
