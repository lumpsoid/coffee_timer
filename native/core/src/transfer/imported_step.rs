use crate::model::{RecipeStep, StepDuration, WaterAmount};
use crate::ports::IdGen;

/// One step read from a document, valid but without an id yet. Its name
/// is already the saved one (a blank name is `Step N`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedStep {
    name: String,
    duration: StepDuration,
    water: Option<WaterAmount>,
}

impl ImportedStep {
    pub(crate) fn new(name: String, duration: StepDuration, water: Option<WaterAmount>) -> Self {
        Self {
            name,
            duration,
            water,
        }
    }

    /// The step, with a fresh id from `ids`.
    pub fn into_step(self, ids: &mut dyn IdGen) -> RecipeStep {
        RecipeStep::new(ids.step_id(), self.name, self.duration, self.water)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn duration(&self) -> StepDuration {
        self.duration
    }

    pub fn water(&self) -> Option<WaterAmount> {
        self.water
    }
}
