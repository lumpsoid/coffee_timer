//! `SequentialIds`, the deterministic `IdGen` fake.

use coffee_timer_core::fakes::SequentialIds;
use coffee_timer_core::model::{RecipeId, StepId};
use coffee_timer_core::ports::IdGen;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn sequential_ids_count_recipes_and_steps_separately() -> TestResult {
    let mut ids = SequentialIds::default();
    assert_eq!(ids.recipe_id(), RecipeId::try_parse("r1")?);
    assert_eq!(ids.step_id(), StepId::try_parse("s1")?);
    assert_eq!(ids.step_id(), StepId::try_parse("s2")?);
    assert_eq!(ids.recipe_id(), RecipeId::try_parse("r2")?);
    Ok(())
}
