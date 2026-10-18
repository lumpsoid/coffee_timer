use std::fmt;

use crate::model::ModelError;

/// Why a text is not a recipe document this app can import. Each value
/// error names where it is: `Recipe 2, step 3, seconds: 0–5999 seconds`
/// (1-based, in document order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentError {
    /// The text is blank.
    Empty,
    /// Not JSON, or not in the document's shape; serde's reason, with its
    /// line and column.
    Malformed(String),
    /// JSON, but its `format` is not [`super::RecipeDocument::FORMAT`].
    WrongFormat,
    /// The document has no `version`.
    MissingVersion,
    /// A `version` this app does not read, such as a newer app's.
    UnsupportedVersion(u64),
    /// `recipes` is empty.
    NoRecipes,
    /// The recipe at `recipe` has no steps.
    NoSteps { recipe: usize },
    /// A value its model type refuses: `field` (the JSON key) of the
    /// recipe at `recipe`, or of its step at `step`.
    Invalid {
        recipe: usize,
        step: Option<usize>,
        field: &'static str,
        error: ModelError,
    },
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "The text is empty"),
            Self::Malformed(reason) => write!(f, "Not a recipe document: {reason}"),
            Self::WrongFormat => write!(f, "Not a Coffee Timer recipe document"),
            Self::MissingVersion => write!(f, "The recipe document has no version"),
            Self::UnsupportedVersion(version) => write!(
                f,
                "Recipe document version {version} is not supported; this app reads version {}",
                super::RecipeDocument::VERSION
            ),
            Self::NoRecipes => write!(f, "The recipe document has no recipes"),
            Self::NoSteps { recipe } => write!(f, "Recipe {recipe}: no steps"),
            Self::Invalid {
                recipe,
                step: None,
                field,
                error,
            } => write!(f, "Recipe {recipe}, {field}: {error}"),
            Self::Invalid {
                recipe,
                step: Some(step),
                field,
                error,
            } => write!(f, "Recipe {recipe}, step {step}, {field}: {error}"),
        }
    }
}

impl std::error::Error for DocumentError {}
