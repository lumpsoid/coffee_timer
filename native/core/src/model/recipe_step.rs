use crate::model::{StepDuration, StepId};

/// One step of a brew: an id, a name, and how long it runs. Immutable — a
/// step is replaced, never edited in place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeStep {
    id: StepId,
    name: String,
    duration: StepDuration,
}

impl RecipeStep {
    pub fn new(id: StepId, name: String, duration: StepDuration) -> Self {
        Self { id, name, duration }
    }

    /// The name to store for a step at `position` (1-based), given the raw
    /// text a user typed. Blank input falls back to `Step N`.
    pub fn label_or_default(raw: &str, position: usize) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            format!("Step {position}")
        } else {
            trimmed.to_owned()
        }
    }

    pub fn id(&self) -> &StepId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn duration(&self) -> StepDuration {
        self.duration
    }
}
