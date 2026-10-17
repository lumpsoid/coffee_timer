use coffee_timer_core::model::{RecipeId, StepId};
use coffee_timer_core::ports::IdGen;
use uuid::Uuid;

/// The production [`IdGen`]: every id is a fresh UUID v7 (time-ordered),
/// as hyphenated lowercase text. Lives here, beside the store, so both
/// heads (hub and CLI) share it.
#[derive(Debug, Clone, Copy, Default)]
pub struct UuidIds;

impl IdGen for UuidIds {
    fn recipe_id(&mut self) -> RecipeId {
        RecipeId::from_u128(Uuid::now_v7().as_u128())
    }

    fn step_id(&mut self) -> StepId {
        StepId::from_u128(Uuid::now_v7().as_u128())
    }
}
