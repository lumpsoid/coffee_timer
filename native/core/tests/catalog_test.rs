//! `RecipeCatalog`: load, upsert, remove, restore and pin write through the
//! store first; the list is in `PinnedFirst` order.

use coffee_timer_core::catalog::{
    CatalogChange, PinError, PinnedFirst, RecipeCatalog, RecipeSummary, RestoreError,
};
use coffee_timer_core::fakes::FakeRecipeStore;
use coffee_timer_core::model::{
    BrewMetrics, CoffeeDose, LabelStyle, ModelError, Pin, Recipe, RecipeId, RecipeName, RecipeStep,
    StepDuration, StepId, WaterAmount,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn recipe(id: &str, name: &str) -> Result<Recipe, ModelError> {
    Ok(Recipe::blank(
        RecipeId::try_parse(id)?,
        RecipeName::try_parse(name)?,
        CoffeeDose::try_parse(15)?,
        WaterAmount::try_parse(250)?,
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

// restore (undo the last removal)

fn rid(raw: &str) -> Result<RecipeId, ModelError> {
    RecipeId::try_parse(raw)
}

fn stored_abc() -> Result<FakeRecipeStore, ModelError> {
    Ok(FakeRecipeStore::with_recipes(vec![
        recipe("a", "Espresso")?,
        poured("b", "Pour Over")?,
        recipe("c", "Aero")?,
    ]))
}

/// A recipe with its own dose and a pouring step, so a restore can be
/// checked for the whole recipe.
fn poured(id: &str, name: &str) -> Result<Recipe, ModelError> {
    Ok(Recipe::new(
        rid(id)?,
        RecipeName::try_parse(name)?,
        "notes".to_owned(),
        CoffeeDose::try_parse(18)?,
        WaterAmount::try_parse(300)?,
        vec![
            RecipeStep::new(
                StepId::try_parse("s1")?,
                "Bloom".to_owned(),
                StepDuration::try_parse(30)?,
                Some(WaterAmount::try_parse(50)?),
            ),
            RecipeStep::new(
                StepId::try_parse("s2")?,
                "Wait".to_owned(),
                StepDuration::try_parse(90)?,
                None,
            ),
        ],
    ))
}

#[test]
fn remove_then_restore_puts_the_whole_recipe_back_in_place_and_persists() -> TestResult {
    let mut catalog = loaded(stored_abc()?)?;
    catalog.remove(rid("b")?)?;
    assert_eq!(ids(&catalog), ["a", "c"]);

    let change = catalog.restore(&rid("b")?)?;

    assert_eq!(change, CatalogChange::Upserted(rid("b")?));
    assert_eq!(ids(&catalog), ["a", "b", "c"]);
    assert_eq!(catalog.get(&rid("b")?), Some(&poured("b", "Pour Over")?));
    catalog.load()?;
    assert_eq!(ids(&catalog), ["a", "b", "c"], "store has it back in place");
    assert_eq!(catalog.get(&rid("b")?), Some(&poured("b", "Pour Over")?));
    Ok(())
}

#[test]
fn a_removed_recipe_is_offered_for_undo_with_its_notice() -> TestResult {
    let mut catalog = loaded(stored_abc()?)?;
    assert!(catalog.restorable(&rid("b")?).is_none());

    catalog.remove(rid("b")?)?;

    let removed = catalog.restorable(&rid("b")?).ok_or("no undo offered")?;
    assert_eq!(removed.notice(), "Deleted Pour Over");
    assert_eq!(removed.index(), 1);
    assert!(catalog.restorable(&rid("a")?).is_none());
    Ok(())
}

#[test]
fn a_second_undo_is_an_error_and_changes_nothing() -> TestResult {
    let mut catalog = loaded(stored_abc()?)?;
    catalog.remove(rid("b")?)?;
    catalog.restore(&rid("b")?)?;

    assert_eq!(
        catalog.restore(&rid("b")?),
        Err(RestoreError::NothingToUndo(rid("b")?))
    );
    assert!(catalog.restorable(&rid("b")?).is_none());
    assert_eq!(ids(&catalog), ["a", "b", "c"]);
    Ok(())
}

#[test]
fn only_the_last_removal_can_be_undone() -> TestResult {
    let mut catalog = loaded(stored_abc()?)?;
    catalog.remove(rid("a")?)?;
    catalog.remove(rid("b")?)?;

    assert_eq!(
        catalog.restore(&rid("a")?),
        Err(RestoreError::NothingToUndo(rid("a")?))
    );
    catalog.restore(&rid("b")?)?;
    assert_eq!(ids(&catalog), ["b", "c"], "b back first, a stays gone");
    Ok(())
}

#[test]
fn undo_of_a_recipe_never_removed_is_an_error() -> TestResult {
    let mut catalog = loaded(stored_abc()?)?;
    assert_eq!(
        catalog.restore(&rid("missing")?),
        Err(RestoreError::NothingToUndo(rid("missing")?))
    );
    assert_eq!(ids(&catalog), ["a", "b", "c"]);
    Ok(())
}

#[test]
fn removing_an_unknown_id_keeps_the_earlier_undo() -> TestResult {
    let mut catalog = loaded(stored_abc()?)?;
    catalog.remove(rid("c")?)?;
    catalog.remove(rid("missing")?)?;

    catalog.restore(&rid("c")?)?;
    assert_eq!(ids(&catalog), ["a", "b", "c"]);
    Ok(())
}

#[test]
fn a_failed_restore_leaves_the_catalog_and_the_undo_unchanged() -> TestResult {
    let mut catalog = loaded(stored_abc()?.failing_inserts())?;
    catalog.remove(rid("a")?)?;

    assert!(matches!(
        catalog.restore(&rid("a")?),
        Err(RestoreError::Store(_))
    ));
    assert_eq!(ids(&catalog), ["b", "c"]);
    assert!(catalog.restorable(&rid("a")?).is_some(), "still on offer");
    Ok(())
}

#[test]
fn index_of_is_the_place_in_catalog_order() -> TestResult {
    let catalog = loaded(stored_abc()?)?;
    assert_eq!(catalog.index_of(&rid("c")?), Some(2));
    assert_eq!(catalog.index_of(&rid("missing")?), None);
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
fn summary_carries_the_stored_dose_and_the_total_time() -> TestResult {
    let step = |id: &str, seconds: i64| -> Result<RecipeStep, ModelError> {
        Ok(RecipeStep::new(
            StepId::try_parse(id)?,
            "step".to_owned(),
            StepDuration::try_parse(seconds)?,
            None,
        ))
    };
    let recipe = Recipe::new(
        RecipeId::try_parse("a")?,
        RecipeName::try_parse("Espresso")?,
        "Strong coffee".to_owned(),
        CoffeeDose::try_parse(18)?,
        WaterAmount::try_parse(36)?,
        vec![step("s1", 30)?, step("s2", 65)?],
    );
    assert_eq!(
        RecipeSummary::of(&recipe, Pin::Pinned, &BrewMetrics::default()),
        RecipeSummary {
            id: RecipeId::try_parse("a")?,
            name: "Espresso".to_owned(),
            description: "Strong coffee".to_owned(),
            coffee_label: "18 g".to_owned(),
            water_label: "36 g".to_owned(),
            time_label: "1:35".to_owned(),
            style: LabelStyle::of(&recipe, &BrewMetrics::default()),
            pin: Pin::Pinned,
        }
    );
    Ok(())
}

// pin

fn listed(catalog: &RecipeCatalog) -> Vec<&str> {
    catalog.listed().iter().map(|r| r.id().as_str()).collect()
}

#[test]
fn pinned_first_keeps_each_group_in_catalog_order() -> TestResult {
    let recipes = [
        recipe("a", "A")?,
        recipe("b", "B")?,
        recipe("c", "C")?,
        recipe("d", "D")?,
    ];
    let pinned = |r: &Recipe| Pin::from(matches!(r.id().as_str(), "b" | "d"));
    let order: Vec<_> = PinnedFirst::order(&recipes, pinned)
        .iter()
        .map(|r| r.id().as_str())
        .collect();
    assert_eq!(order, ["b", "d", "a", "c"]);
    let none: Vec<_> = PinnedFirst::order(&recipes, |_| Pin::Unpinned)
        .iter()
        .map(|r| r.id().as_str())
        .collect();
    assert_eq!(none, ["a", "b", "c", "d"]);
    Ok(())
}

#[test]
fn load_reads_the_stored_pins() -> TestResult {
    let catalog = loaded(stored_abc()?.pinning(&rid("c")?))?;
    assert_eq!(catalog.pin_of(&rid("c")?), Pin::Pinned);
    assert_eq!(catalog.pin_of(&rid("a")?), Pin::Unpinned);
    assert_eq!(listed(&catalog), ["c", "a", "b"]);
    assert_eq!(ids(&catalog), ["a", "b", "c"], "stored order is kept");
    Ok(())
}

#[test]
fn pin_moves_the_recipe_to_the_top_and_persists() -> TestResult {
    let mut catalog = loaded(stored_abc()?)?;
    let change = catalog.set_pin(&rid("c")?, Pin::Pinned)?;
    assert_eq!(change, CatalogChange::Upserted(rid("c")?));
    assert_eq!(listed(&catalog), ["c", "a", "b"]);
    assert_eq!(catalog.index_of(&rid("c")?), Some(0));
    assert_eq!(catalog.index_of(&rid("a")?), Some(1));

    catalog.load()?;
    assert_eq!(listed(&catalog), ["c", "a", "b"], "store has the pin");
    Ok(())
}

#[test]
fn pinned_recipes_keep_their_relative_order() -> TestResult {
    let mut catalog = loaded(stored_abc()?)?;
    catalog.set_pin(&rid("c")?, Pin::Pinned)?;
    catalog.set_pin(&rid("b")?, Pin::Pinned)?;
    assert_eq!(listed(&catalog), ["b", "c", "a"]);
    Ok(())
}

#[test]
fn unpin_puts_the_recipe_back_in_its_place_among_the_unpinned() -> TestResult {
    let mut catalog = loaded(stored_abc()?.pinning(&rid("b")?).pinning(&rid("c")?))?;
    assert_eq!(listed(&catalog), ["b", "c", "a"]);
    let change = catalog.set_pin(&rid("c")?, Pin::Unpinned)?;
    assert_eq!(change, CatalogChange::Upserted(rid("c")?));
    assert_eq!(listed(&catalog), ["b", "a", "c"]);
    assert_eq!(catalog.index_of(&rid("c")?), Some(2));
    catalog.load()?;
    assert_eq!(listed(&catalog), ["b", "a", "c"], "store has it unpinned");
    Ok(())
}

#[test]
fn setting_the_pin_a_recipe_has_changes_nothing_but_still_reports_it() -> TestResult {
    let mut catalog = loaded(stored_abc()?)?;
    let change = catalog.set_pin(&rid("a")?, Pin::Unpinned)?;
    assert_eq!(change, CatalogChange::Upserted(rid("a")?));
    assert_eq!(listed(&catalog), ["a", "b", "c"]);
    Ok(())
}

#[test]
fn pinning_an_unknown_id_is_an_error() -> TestResult {
    let mut catalog = loaded(stored_abc()?)?;
    assert_eq!(
        catalog.set_pin(&rid("missing")?, Pin::Pinned),
        Err(PinError::UnknownRecipe(rid("missing")?))
    );
    assert_eq!(catalog.pin_of(&rid("missing")?), Pin::Unpinned);
    Ok(())
}

#[test]
fn a_failed_pin_leaves_the_catalog_unchanged() -> TestResult {
    let mut catalog = loaded(stored_abc()?.failing_writes())?;
    assert!(matches!(
        catalog.set_pin(&rid("c")?, Pin::Pinned),
        Err(PinError::Store(_))
    ));
    assert_eq!(catalog.pin_of(&rid("c")?), Pin::Unpinned);
    assert_eq!(listed(&catalog), ["a", "b", "c"]);
    Ok(())
}

#[test]
fn a_new_recipe_is_unpinned_at_the_end_of_the_list() -> TestResult {
    let mut catalog = loaded(stored_abc()?.pinning(&rid("c")?))?;
    catalog.upsert(recipe("d", "V60")?)?;
    assert_eq!(catalog.pin_of(&rid("d")?), Pin::Unpinned);
    assert_eq!(listed(&catalog), ["c", "a", "b", "d"]);
    assert_eq!(catalog.index_of(&rid("d")?), Some(3));
    Ok(())
}

#[test]
fn an_upsert_of_a_pinned_recipe_keeps_it_pinned_in_place() -> TestResult {
    let mut catalog = loaded(stored_abc()?.pinning(&rid("c")?))?;
    catalog.upsert(recipe("c", "Renamed")?)?;
    assert_eq!(catalog.pin_of(&rid("c")?), Pin::Pinned);
    assert_eq!(listed(&catalog), ["c", "a", "b"]);
    Ok(())
}

#[test]
fn undo_restores_a_pinned_recipe_pinned_at_its_place() -> TestResult {
    let mut catalog = loaded(stored_abc()?.pinning(&rid("a")?).pinning(&rid("c")?))?;
    assert_eq!(listed(&catalog), ["a", "c", "b"]);
    catalog.remove(rid("c")?)?;
    assert_eq!(
        catalog.pin_of(&rid("c")?),
        Pin::Unpinned,
        "gone, so not pinned"
    );
    assert_eq!(listed(&catalog), ["a", "b"]);
    let removed = catalog.restorable(&rid("c")?).ok_or("no undo")?;
    assert_eq!(removed.pin(), Pin::Pinned);

    catalog.restore(&rid("c")?)?;

    assert_eq!(catalog.pin_of(&rid("c")?), Pin::Pinned);
    assert_eq!(listed(&catalog), ["a", "c", "b"]);
    assert_eq!(catalog.index_of(&rid("c")?), Some(1));
    catalog.load()?;
    assert_eq!(listed(&catalog), ["a", "c", "b"], "store has it pinned");
    Ok(())
}

#[test]
fn undo_restores_an_unpinned_recipe_unpinned() -> TestResult {
    let mut catalog = loaded(stored_abc()?.pinning(&rid("c")?))?;
    catalog.remove(rid("a")?)?;
    catalog.restore(&rid("a")?)?;
    assert_eq!(catalog.pin_of(&rid("a")?), Pin::Unpinned);
    assert_eq!(listed(&catalog), ["c", "a", "b"]);
    Ok(())
}
