use crate::model::{StepDuration, StepId, WaterAmount};

/// One step of a brew: an id, a name, how long it runs, and the water
/// poured in it, if any. Immutable — a step is replaced, never edited in
/// place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeStep {
    id: StepId,
    name: String,
    duration: StepDuration,
    water: Option<WaterAmount>,
}

impl RecipeStep {
    /// `water` is what the step pours; none for a step that pours nothing.
    pub fn new(
        id: StepId,
        name: String,
        duration: StepDuration,
        water: Option<WaterAmount>,
    ) -> Self {
        Self {
            id,
            name,
            duration,
            water,
        }
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

    /// The same step pouring `water` instead; none pours nothing.
    pub fn pouring(self, water: Option<WaterAmount>) -> Self {
        Self { water, ..self }
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

    /// The water poured in this step, at the recipe's own ratio; none for a
    /// step that pours nothing.
    pub fn water(&self) -> Option<WaterAmount> {
        self.water
    }
}
