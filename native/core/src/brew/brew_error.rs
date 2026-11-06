use std::fmt;

/// Why a brew could not start, pause or resume, or its dose not change. The state is unchanged when one is returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrewError {
    /// No recipe is open.
    NoRecipe,
    /// The chosen first step does not exist (a recipe without steps has none).
    StepOutOfRange { index: usize, len: usize },
    /// Only a running brew can pause.
    NotRunning,
    /// Only a paused brew can resume.
    NotPaused,
}

impl fmt::Display for BrewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRecipe => write!(f, "No recipe is open"),
            Self::StepOutOfRange { index, len } => {
                write!(f, "Step {index} is out of range for {len} steps")
            }
            Self::NotRunning => write!(f, "No brew is running"),
            Self::NotPaused => write!(f, "No brew is paused"),
        }
    }
}

impl std::error::Error for BrewError {}
