use std::fmt;

/// Why a brew could not start, or its ratio not change. The state is unchanged when one is returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrewError {
    /// No recipe is open.
    NoRecipe,
    /// The chosen first step does not exist (a recipe without steps has none).
    StepOutOfRange { index: usize, len: usize },
    /// The open recipe has no coffee, so it has no ratio to change.
    NoCoffee,
}

impl fmt::Display for BrewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRecipe => write!(f, "No recipe is open"),
            Self::StepOutOfRange { index, len } => {
                write!(f, "Step {index} is out of range for {len} steps")
            }
            Self::NoCoffee => write!(f, "The recipe has no coffee to set a ratio for"),
        }
    }
}

impl std::error::Error for BrewError {}
