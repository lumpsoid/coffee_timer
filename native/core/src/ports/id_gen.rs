use crate::model::{RecipeId, StepId};

/// Mints fresh ids for new recipes and new steps. One generator serves both:
/// they are minted by the same feature (the editor) from the same source.
pub trait IdGen {
    fn recipe_id(&mut self) -> RecipeId;
    fn step_id(&mut self) -> StepId;
}
