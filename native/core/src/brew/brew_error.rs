use std::fmt;

/// Why a brew could not start. The state is unchanged when one is returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrewError {
    /// No recipe is open.
    NoRecipe,
    /// The chosen first step does not exist (a recipe without steps has none).
    StepOutOfRange { index: usize, len: usize },
}

impl fmt::Display for BrewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRecipe => write!(f, "No recipe is open"),
            Self::StepOutOfRange { index, len } => {
                write!(f, "Step {index} is out of range for {len} steps")
            }
        }
    }
}

impl std::error::Error for BrewError {}
