use std::fmt;

use crate::model::{RecipeId, StepId};

/// Why a draft could not be opened or changed. The draft is unchanged when
/// one is returned, except that, like every action on it, the previous
/// notice is cleared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    /// No draft is open.
    NoDraft,
    /// The catalog holds no recipe with this id.
    UnknownRecipe(RecipeId),
    /// The draft has no step row with this id.
    UnknownStep(StepId),
    /// A step row cannot move to position `to` (0-based): the draft has
    /// only `rows` rows.
    StepPositionOutOfRange { to: usize, rows: usize },
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDraft => write!(f, "No recipe is being edited"),
            Self::UnknownRecipe(id) => write!(f, "No recipe with id {}", id.as_str()),
            Self::UnknownStep(id) => write!(f, "No step with id {}", id.as_str()),
            Self::StepPositionOutOfRange { to, rows } => {
                write!(f, "No step position {to} in a recipe of {rows} steps")
            }
        }
    }
}

impl std::error::Error for EditError {}
