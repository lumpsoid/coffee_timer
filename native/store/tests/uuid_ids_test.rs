//! `UuidIds`, the production `IdGen`.

use coffee_timer_core::model::RecipeId;
use coffee_timer_core::ports::IdGen;
use coffee_timer_store::UuidIds;
use uuid::Uuid;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn ids_are_distinct_v7_uuids_in_uuid_text_form() -> TestResult {
    let mut ids = UuidIds;
    let recipe = ids.recipe_id();
    let step = ids.step_id();
    for text in [recipe.as_str(), step.as_str()] {
        let parsed = Uuid::try_parse(text)?;
        assert_eq!(parsed.get_version_num(), 7);
        assert_eq!(parsed.hyphenated().to_string(), text);
    }
    assert_ne!(recipe.as_str(), step.as_str());
    Ok(())
}

#[test]
fn from_u128_matches_the_uuid_crate_text() {
    let uuid = Uuid::now_v7();
    assert_eq!(
        RecipeId::from_u128(uuid.as_u128()).as_str(),
        uuid.hyphenated().to_string()
    );
}
