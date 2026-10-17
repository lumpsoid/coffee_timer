//! `RecipeCatalog`: load, upsert and remove write through the store first.

use coffee_timer_core::catalog::{CatalogChange, RecipeCatalog, RecipeSummary};
use coffee_timer_core::fakes::FakeRecipeStore;
use coffee_timer_core::model::{
    BrewMetrics, LabelStyle, ModelError, Recipe, RecipeId, RecipeName, RecipeStep, StepDuration,
    StepId,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn recipe(id: &str, name: &str) -> Result<Recipe, ModelError> {
    Ok(Recipe::blank(
        RecipeId::try_parse(id)?,
        RecipeName::try_parse(name)?,
    ))
}

fn ids(catalog: &RecipeCatalog) -> Vec<&str> {
    catalog.state().iter().map(|r| r.id().as_str()).collect()
}

fn names(catalog: &RecipeCatalog) -> Vec<&str> {
    catalog.state().iter().map(|r| r.name().as_str()).collect()
}

fn loaded(store: FakeRecipeStore) -> Result<RecipeCatalog, Box<dyn std::error::Error>> {
    let mut catalog = RecipeCatalog::new(Box::new(store));
    catalog.load()?;
    Ok(catalog)
}

fn stored_ab() -> Result<FakeRecipeStore, ModelError> {
    Ok(FakeRecipeStore::with_recipes(vec![
        recipe("a", "Espresso")?,
        recipe("b", "Pour Over")?,
    ]))
}

// load

#[test]
fn new_starts_empty_until_loaded() -> TestResult {
    let catalog = RecipeCatalog::new(Box::new(stored_ab()?));
    assert!(catalog.state().is_empty());
    Ok(())
}

#[test]
fn load_replaces_the_state_in_stored_order() -> TestResult {
    let mut catalog = loaded(stored_ab()?)?;
    catalog.upsert(recipe("c", "Aero")?)?;
    catalog.load()?;
    assert_eq!(ids(&catalog), ["a", "b", "c"]);
    Ok(())
}

#[test]
fn failed_load_leaves_state_unchanged() -> TestResult {
    let mut catalog = RecipeCatalog::new(Box::new(stored_ab()?.failing_loads()));
    assert!(catalog.load().is_err());
    assert!(catalog.state().is_empty());
    Ok(())
}

// upsert

#[test]
fn upsert_new_id_appends_and_persists() -> TestResult {
    let mut catalog = loaded(stored_ab()?)?;
    let change = catalog.upsert(recipe("c", "Aero")?)?;
    assert_eq!(change, CatalogChange::Upserted(RecipeId::try_parse("c")?));
    assert_eq!(ids(&catalog), ["a", "b", "c"]);

    catalog.load()?;
    assert_eq!(ids(&catalog), ["a", "b", "c"], "store has it too");
    Ok(())
}

#[test]
fn upsert_known_id_replaces_in_place_and_persists() -> TestResult {
    let mut catalog = loaded(stored_ab()?)?;
    let change = catalog.upsert(recipe("a", "Ristretto")?)?;
    assert_eq!(change, CatalogChange::Upserted(RecipeId::try_parse("a")?));
    assert_eq!(names(&catalog), ["Ristretto", "Pour Over"]);

    catalog.load()?;
    assert_eq!(
        names(&catalog),
        ["Ristretto", "Pour Over"],
        "store has it too"
    );
    Ok(())
}

#[test]
fn upsert_into_an_empty_catalog() -> TestResult {
    let mut catalog = RecipeCatalog::new(Box::new(FakeRecipeStore::default()));
    catalog.upsert(recipe("r1", "V60")?)?;
    assert_eq!(catalog.state().len(), 1);
    Ok(())
}

#[test]
fn failed_upsert_leaves_state_unchanged() -> TestResult {
    let mut catalog = loaded(stored_ab()?.failing_writes())?;
    let before = catalog.state().to_vec();

    assert!(catalog.upsert(recipe("c", "Aero")?).is_err());
    assert!(catalog.upsert(recipe("a", "Ristretto")?).is_err());
    assert_eq!(catalog.state(), before.as_slice());
    Ok(())
}

// remove

#[test]
fn remove_known_id_drops_it_and_persists() -> TestResult {
    let mut catalog = loaded(stored_ab()?)?;
    let change = catalog.remove(RecipeId::try_parse("a")?)?;
    assert_eq!(change, CatalogChange::Removed(RecipeId::try_parse("a")?));
    assert_eq!(ids(&catalog), ["b"]);

    catalog.load()?;
    assert_eq!(ids(&catalog), ["b"], "store dropped it too");
    Ok(())
}

#[test]
fn remove_unknown_id_is_a_no_op() -> TestResult {
    // Legacy: SQL DELETE of a missing row and `removeWhere` both no-op.
    let mut catalog = loaded(stored_ab()?)?;
    let change = catalog.remove(RecipeId::try_parse("missing")?)?;
    assert_eq!(
        change,
        CatalogChange::Removed(RecipeId::try_parse("missing")?)
    );
    assert_eq!(ids(&catalog), ["a", "b"]);
    Ok(())
}

#[test]
fn failed_remove_leaves_state_unchanged() -> TestResult {
    let mut catalog = loaded(stored_ab()?.failing_writes())?;
    assert!(catalog.remove(RecipeId::try_parse("a")?).is_err());
    assert_eq!(ids(&catalog), ["a", "b"]);
    Ok(())
}

// get

#[test]
fn get_finds_a_recipe_by_id() -> TestResult {
    let catalog = loaded(stored_ab()?)?;
    let found = catalog.get(&RecipeId::try_parse("b")?).ok_or("missing b")?;
    assert_eq!(found.name().as_str(), "Pour Over");
    assert!(catalog.get(&RecipeId::try_parse("missing")?).is_none());
    Ok(())
}

#[test]
fn summary_carries_the_legacy_tile_values() -> TestResult {
    let step = |id: &str, seconds: i64| -> Result<RecipeStep, ModelError> {
        Ok(RecipeStep::new(
            StepId::try_parse(id)?,
            "step".to_owned(),
            StepDuration::try_parse(seconds)?,
        ))
    };
    let recipe = Recipe::new(
        RecipeId::try_parse("a")?,
        RecipeName::try_parse("Espresso")?,
        "Strong coffee".to_owned(),
        vec![step("s1", 30)?, step("s2", 65)?],
    );
    assert_eq!(
        RecipeSummary::of(&recipe, &BrewMetrics::default()),
        RecipeSummary {
            id: RecipeId::try_parse("a")?,
            name: "Espresso".to_owned(),
            description: "Strong coffee".to_owned(),
            coffee_label: "20 g".to_owned(),
            water_label: "100 g".to_owned(),
            time_label: "1:35".to_owned(),
            style: LabelStyle::of(&recipe, &BrewMetrics::default()),
        }
    );
    Ok(())
}
