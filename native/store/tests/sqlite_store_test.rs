//! `SqliteStore`: schema v1 and seed, recipe order, step replacement,
//! settings round trip and fallback, and the core controllers over it.

use std::path::Path;

use coffee_timer_core::catalog::{CatalogChange, RecipeCatalog};
use coffee_timer_core::model::{
    AccentColor, Recipe, RecipeId, RecipeName, RecipeStep, Settings, StepDuration, StepId,
    ThemeSelection,
};
use coffee_timer_core::ports::{RecipeStore, SettingsStore};
use coffee_timer_core::settings::SettingsController;
use coffee_timer_store::SqliteStore;
use rusqlite::Connection;

type TestResult = Result<(), Box<dyn std::error::Error>>;
type Outcome<T> = Result<T, Box<dyn std::error::Error>>;

/// `(id, name, description, [(step id, step name, seconds)])`.
type Shape = (String, String, String, Vec<(String, String, u32)>);

fn shape(recipes: &[Recipe]) -> Vec<Shape> {
    recipes
        .iter()
        .map(|r| {
            (
                r.id().as_str().to_owned(),
                r.name().as_str().to_owned(),
                r.description().to_owned(),
                r.steps()
                    .iter()
                    .map(|s| {
                        (
                            s.id().as_str().to_owned(),
                            s.name().to_owned(),
                            s.duration().seconds(),
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

fn expected(id: &str, name: &str, description: &str, steps: &[(&str, &str, u32)]) -> Shape {
    (
        id.to_owned(),
        name.to_owned(),
        description.to_owned(),
        steps
            .iter()
            .map(|&(sid, sname, secs)| (sid.to_owned(), sname.to_owned(), secs))
            .collect(),
    )
}

fn seed() -> Vec<Shape> {
    vec![
        expected(
            "espresso-uuid",
            "Espresso",
            "Strong coffee",
            &[("step1", "Grind", 30), ("step2", "Brew", 25)],
        ),
        expected(
            "pourover-uuid",
            "Pour Over",
            "Light and clean",
            &[
                ("step3", "Grind", 40),
                ("step4", "Bloom", 30),
                ("step5", "Pour", 120),
            ],
        ),
    ]
}

fn step(id: &str, name: &str, seconds: i64) -> Outcome<RecipeStep> {
    Ok(RecipeStep::new(
        StepId::try_parse(id)?,
        name.to_owned(),
        StepDuration::try_parse(seconds)?,
    ))
}

fn recipe(id: &str, name: &str, steps: Vec<RecipeStep>) -> Outcome<Recipe> {
    Ok(Recipe::new(
        RecipeId::try_parse(id)?,
        RecipeName::try_parse(name)?,
        format!("{name} notes"),
        steps,
    ))
}

fn ids(recipes: &[Recipe]) -> Vec<&str> {
    recipes.iter().map(|r| r.id().as_str()).collect()
}

fn count(path: &Path, sql: &str) -> Outcome<i64> {
    Ok(Connection::open(path)?.query_row(sql, [], |row| row.get(0))?)
}

// --- schema and seed ---

#[test]
fn fresh_database_is_seeded_with_espresso_then_pour_over() -> TestResult {
    let store = SqliteStore::open_in_memory()?;
    assert_eq!(shape(&store.load_all()?), seed());
    Ok(())
}

#[test]
fn opened_file_is_at_schema_v1() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    SqliteStore::open(&path)?;
    let version: u32 =
        Connection::open(&path)?.pragma_query_value(None, "user_version", |row| row.get(0))?;
    assert_eq!(version, 1);
    Ok(())
}

#[test]
fn reopening_a_migrated_file_does_not_reseed() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    let mut first = SqliteStore::open(&path)?;
    first.upsert(&recipe("r1", "V60", vec![step("s1", "Pour", 90)?])?)?;
    drop(first);

    let reopened = SqliteStore::open(&path)?;
    assert_eq!(
        ids(&reopened.load_all()?),
        ["espresso-uuid", "pourover-uuid", "r1"]
    );
    assert_eq!(count(&path, "SELECT COUNT(*) FROM step")?, 6);
    Ok(())
}

#[test]
fn emptied_catalog_stays_empty_after_reopen() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    let mut first = SqliteStore::open(&path)?;
    first.remove(&RecipeId::try_parse("espresso-uuid")?)?;
    first.remove(&RecipeId::try_parse("pourover-uuid")?)?;
    drop(first);

    assert!(SqliteStore::open(&path)?.load_all()?.is_empty());
    Ok(())
}

// --- recipes ---

#[test]
fn upsert_of_a_new_recipe_appends() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    store.upsert(&recipe("r1", "V60", vec![step("s1", "Pour", 90)?])?)?;
    store.upsert(&recipe("r2", "Aeropress", Vec::new())?)?;

    let loaded = store.load_all()?;
    assert_eq!(ids(&loaded), ["espresso-uuid", "pourover-uuid", "r1", "r2"]);
    assert_eq!(
        shape(&loaded[2..]),
        [
            expected("r1", "V60", "V60 notes", &[("s1", "Pour", 90)]),
            expected("r2", "Aeropress", "Aeropress notes", &[]),
        ]
    );
    Ok(())
}

#[test]
fn upsert_of_a_known_recipe_keeps_its_position_and_replaces_its_steps() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    store.upsert(&recipe("r1", "V60", vec![step("s1", "Pour", 90)?])?)?;

    let edited = recipe(
        "espresso-uuid",
        "Ristretto",
        vec![
            step("n3", "Tamp", 5)?,
            step("step2", "Brew", 20)?,
            step("n1", "Rest", 0)?,
        ],
    )?;
    store.upsert(&edited)?;

    let loaded = store.load_all()?;
    assert_eq!(ids(&loaded), ["espresso-uuid", "pourover-uuid", "r1"]);
    assert_eq!(
        shape(&loaded[..1]),
        [expected(
            "espresso-uuid",
            "Ristretto",
            "Ristretto notes",
            &[("n3", "Tamp", 5), ("step2", "Brew", 20), ("n1", "Rest", 0)],
        )]
    );
    Ok(())
}

#[test]
fn upsert_after_removing_the_last_recipe_still_appends() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    store.remove(&RecipeId::try_parse("pourover-uuid")?)?;
    store.upsert(&recipe("r1", "V60", Vec::new())?)?;
    store.upsert(&recipe("espresso-uuid", "Espresso", Vec::new())?)?;
    assert_eq!(ids(&store.load_all()?), ["espresso-uuid", "r1"]);
    Ok(())
}

#[test]
fn failed_upsert_leaves_the_store_unchanged() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    let clashing = recipe(
        "pourover-uuid",
        "Changed",
        vec![step("fresh", "Pour", 60)?, step("step1", "Clash", 10)?],
    )?;

    assert!(store.upsert(&clashing).is_err());
    assert_eq!(shape(&store.load_all()?), seed());
    Ok(())
}

#[test]
fn remove_deletes_the_recipe_and_its_steps() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    let mut store = SqliteStore::open(&path)?;
    store.remove(&RecipeId::try_parse("pourover-uuid")?)?;

    assert_eq!(ids(&store.load_all()?), ["espresso-uuid"]);
    assert_eq!(
        count(
            &path,
            "SELECT COUNT(*) FROM step WHERE recipe_id = 'pourover-uuid'"
        )?,
        0
    );
    assert_eq!(count(&path, "SELECT COUNT(*) FROM step")?, 2);
    Ok(())
}

#[test]
fn remove_of_an_unknown_id_is_ok_and_changes_nothing() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    store.remove(&RecipeId::try_parse("nope")?)?;
    assert_eq!(shape(&store.load_all()?), seed());
    Ok(())
}

#[test]
fn stored_rows_are_read_through_the_core_constructors() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    let store = SqliteStore::open(&path)?;
    let raw = Connection::open(&path)?;
    raw.execute_batch(
        "INSERT INTO recipe (id, name, description) VALUES ('', 'Ghost', '');
         INSERT INTO recipe (id, name, description) VALUES ('blank', '   ', 'd');
         INSERT INTO step (id, recipe_id, position, name, duration_seconds) VALUES
             ('', 'blank', 0, 'Dropped', 10),
             ('big', 'blank', 1, 'Long', 99999),
             ('neg', 'blank', 2, 'Short', -5);",
    )?;

    let loaded = store.load_all()?;
    assert_eq!(ids(&loaded), ["espresso-uuid", "pourover-uuid", "blank"]);
    assert_eq!(
        shape(&loaded[2..]),
        [expected(
            "blank",
            "Untitled",
            "d",
            &[("big", "Long", 5999), ("neg", "Short", 0)],
        )]
    );
    Ok(())
}

// --- settings ---

#[test]
fn fresh_settings_load_the_fallback() -> TestResult {
    let store = SqliteStore::open_in_memory()?;
    assert_eq!(store.load()?, Settings::from_storage(None, None));
    Ok(())
}

#[test]
fn settings_round_trip() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    let first = Settings::new(ThemeSelection::Dark, AccentColor::from_argb32(0xFF4C_AF50));
    store.save(first)?;
    assert_eq!(store.load()?, first);

    let second = first.with_theme(ThemeSelection::Light);
    store.save(second)?;
    assert_eq!(store.load()?, second);
    Ok(())
}

#[test]
fn invalid_or_missing_settings_rows_fall_back_per_field() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    let store = SqliteStore::open(&path)?;
    let raw = Connection::open(&path)?;

    raw.execute(
        "INSERT INTO settings (key, value) VALUES ('theme', 'sepia')",
        [],
    )?;
    assert_eq!(store.load()?, Settings::from_storage(Some("sepia"), None));

    raw.execute(
        "INSERT INTO settings (key, value) VALUES ('accent_color', '-FF0000')",
        [],
    )?;
    assert_eq!(store.load()?, Settings::FALLBACK);

    raw.execute("UPDATE settings SET value = 'dark' WHERE key = 'theme'", [])?;
    assert_eq!(
        store.load()?,
        Settings::new(ThemeSelection::Dark, AccentColor::FALLBACK)
    );
    Ok(())
}

// --- controllers over the real store ---

#[test]
fn catalog_and_settings_controllers_persist_through_one_store() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    let store = SqliteStore::open(&path)?;

    let mut catalog = RecipeCatalog::new(Box::new(store.clone()));
    let mut settings = SettingsController::new(Box::new(store));
    catalog.load()?;
    settings.load()?;
    assert_eq!(shape(catalog.state()), seed());
    assert_eq!(*settings.state(), Settings::FALLBACK);

    let change = catalog.upsert(recipe("r1", "V60", vec![step("s1", "Pour", 90)?])?)?;
    assert_eq!(change, CatalogChange::Upserted(RecipeId::try_parse("r1")?));
    catalog.upsert(recipe("pourover-uuid", "Pour Over", Vec::new())?)?;
    catalog.remove(RecipeId::try_parse("espresso-uuid")?)?;
    settings.set_theme(ThemeSelection::Dark)?;
    settings.set_accent(AccentColor::from_argb32(0xFFE9_1E63))?;
    let expected_recipes = shape(catalog.state());
    let expected_settings = *settings.state();
    drop((catalog, settings));

    let reopened = SqliteStore::open(&path)?;
    let mut catalog = RecipeCatalog::new(Box::new(reopened.clone()));
    let mut settings = SettingsController::new(Box::new(reopened));
    catalog.load()?;
    settings.load()?;
    assert_eq!(ids(catalog.state()), ["pourover-uuid", "r1"]);
    assert_eq!(shape(catalog.state()), expected_recipes);
    assert_eq!(*settings.state(), expected_settings);
    Ok(())
}
