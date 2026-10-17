use crate::model::{RecipeId, StepId};
use crate::ports::IdGen;

/// Deterministic [`IdGen`]: recipe ids `r1`, `r2`, … and step ids `s1`,
/// `s2`, …, each from its own counter.
#[derive(Debug, Clone, Default)]
pub struct SequentialIds {
    recipes: u64,
    steps: u64,
}

impl IdGen for SequentialIds {
    fn recipe_id(&mut self) -> RecipeId {
        self.recipes = self.recipes.saturating_add(1);
        RecipeId::sequential(self.recipes)
    }

    fn step_id(&mut self) -> StepId {
        self.steps = self.steps.saturating_add(1);
        StepId::sequential(self.steps)
    }
}
