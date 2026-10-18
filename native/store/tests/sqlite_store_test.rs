//! `SqliteStore`: schema v3 and seed, recipe doses, step pours, recipe order, step replacement,
//! settings round trip and fallback, and the core controllers over it.

use std::path::Path;

use coffee_timer_core::catalog::{CatalogChange, RecipeCatalog};
use coffee_timer_core::model::{
    AccentColor, CoffeeDose, PreAlertLead, Recipe, RecipeId, RecipeName, RecipeStep, Settings,
    StepDuration, StepId, StoredSettings, ThemeSelection, Toggle, WaterAmount,
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
        None,
    ))
}

fn recipe(id: &str, name: &str, steps: Vec<RecipeStep>) -> Outcome<Recipe> {
    Ok(Recipe::new(
        RecipeId::try_parse(id)?,
        RecipeName::try_parse(name)?,
        format!("{name} notes"),
        CoffeeDose::try_parse(20)?,
        WaterAmount::try_parse(300)?,
        steps,
    ))
}

/// `(id, coffee grams, water grams)` of each recipe.
fn doses(recipes: &[Recipe]) -> Vec<(&str, u32, u32)> {
    recipes
        .iter()
        .map(|r| (r.id().as_str(), r.coffee().grams(), r.water().grams()))
        .collect()
}

/// `(step id, water grams)` of every step, in catalog order.
fn pours(recipes: &[Recipe]) -> Vec<(&str, Option<u32>)> {
    recipes
        .iter()
        .flat_map(Recipe::steps)
        .map(|s| (s.id().as_str(), s.water().map(WaterAmount::grams)))
        .collect()
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
fn fresh_database_seeds_real_doses() -> TestResult {
    let store = SqliteStore::open_in_memory()?;
    assert_eq!(
        doses(&store.load_all()?),
        [("espresso-uuid", 18, 36), ("pourover-uuid", 15, 250)]
    );
    Ok(())
}

#[test]
fn doses_round_trip_and_follow_an_upsert() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    let first = recipe("r1", "V60", vec![step("s1", "Pour", 90)?])?;
    store.upsert(&first)?;
    let edited = Recipe::new(
        first.id().clone(),
        first.name().clone(),
        first.description().to_owned(),
        CoffeeDose::try_parse(16)?,
        WaterAmount::try_parse(256)?,
        first.steps().to_vec(),
    );
    store.upsert(&edited)?;
    let loaded = store.load_all()?;
    assert_eq!(doses(&loaded).last(), Some(&("r1", 16, 256)));
    Ok(())
}

#[test]
fn fresh_database_seeds_the_pour_over_pours() -> TestResult {
    let store = SqliteStore::open_in_memory()?;
    assert_eq!(
        pours(&store.load_all()?),
        [
            ("step1", None),
            ("step2", None),
            ("step3", None),
            ("step4", Some(50)),
            ("step5", Some(200)),
        ]
    );
    Ok(())
}

#[test]
fn step_pours_round_trip() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    let bloom = RecipeStep::new(
        StepId::try_parse("s1")?,
        "Bloom".to_owned(),
        StepDuration::try_parse(30)?,
        Some(WaterAmount::try_parse(45)?),
    );
    store.upsert(&recipe("r1", "V60", vec![bloom, step("s2", "Wait", 60)?])?)?;
    let loaded = store.load_all()?;
    assert_eq!(pours(&loaded[2..]), [("s1", Some(45)), ("s2", None)]);
    Ok(())
}

#[test]
fn stored_pours_out_of_range_are_clamped() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    drop(SqliteStore::open(&path)?);
    Connection::open(&path)?.execute_batch(
        "UPDATE step SET water_grams = 0 WHERE id = 'step4';
         UPDATE step SET water_grams = 99999 WHERE id = 'step5';",
    )?;
    let store = SqliteStore::open(&path)?;
    let loaded = store.load_all()?;
    assert_eq!(
        pours(&loaded[1..]),
        [("step3", None), ("step4", Some(1)), ("step5", Some(2000))]
    );
    Ok(())
}

#[test]
fn stored_doses_out_of_range_are_clamped() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    drop(SqliteStore::open(&path)?);
    Connection::open(&path)?.execute_batch(
        "UPDATE recipe SET coffee_grams = 0, water_grams = 0 WHERE id = 'espresso-uuid';
         UPDATE recipe SET coffee_grams = 999, water_grams = 99999 WHERE id = 'pourover-uuid';",
    )?;
    let store = SqliteStore::open(&path)?;
    assert_eq!(
        doses(&store.load_all()?),
        [("espresso-uuid", 1, 1), ("pourover-uuid", 200, 2000)]
    );
    Ok(())
}

#[test]
fn opened_file_is_at_schema_v3() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    SqliteStore::open(&path)?;
    let version: u32 =
        Connection::open(&path)?.pragma_query_value(None, "user_version", |row| row.get(0))?;
    assert_eq!(version, 3);
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
fn insert_at_puts_a_recipe_back_in_its_place_with_its_steps() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    store.upsert(&recipe("r1", "V60", Vec::new())?)?;
    let removed = store
        .load_all()?
        .into_iter()
        .find(|r| r.id().as_str() == "pourover-uuid")
        .ok_or("no pour over")?;
    store.remove(removed.id())?;

    store.insert_at(&removed, 1)?;

    let loaded = store.load_all()?;
    assert_eq!(ids(&loaded), ["espresso-uuid", "pourover-uuid", "r1"]);
    assert_eq!(loaded.get(1), Some(&removed));
    store.upsert(&recipe("r2", "Aeropress", Vec::new())?)?;
    assert_eq!(
        ids(&store.load_all()?),
        ["espresso-uuid", "pourover-uuid", "r1", "r2"],
        "later upserts still append"
    );
    Ok(())
}

#[test]
fn insert_at_the_front_or_past_the_end() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    store.insert_at(&recipe("first", "V60", Vec::new())?, 0)?;
    store.insert_at(&recipe("last", "Aeropress", Vec::new())?, 99)?;
    assert_eq!(
        ids(&store.load_all()?),
        ["first", "espresso-uuid", "pourover-uuid", "last"]
    );
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
    assert_eq!(
        store.load()?,
        Settings::from_storage(StoredSettings::default())
    );
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

    for sound in Toggle::ALL {
        for vibration in Toggle::ALL {
            for lead in PreAlertLead::ALL {
                let cues = second
                    .with_sound(sound)
                    .with_vibration(vibration)
                    .with_pre_alert(lead);
                store.save(cues)?;
                assert_eq!(store.load()?, cues);
            }
        }
    }
    Ok(())
}

#[test]
fn cue_settings_are_stored_as_key_value_rows() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    let mut store = SqliteStore::open(&path)?;
    store.save(
        Settings::FALLBACK
            .with_sound(Toggle::Off)
            .with_pre_alert(PreAlertLead::Seconds10),
    )?;
    let raw = Connection::open(&path)?;
    let row = |key: &str| -> rusqlite::Result<String> {
        raw.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
            row.get(0)
        })
    };
    assert_eq!(row("sound")?, "off");
    assert_eq!(row("vibration")?, "on");
    assert_eq!(row("pre_alert_seconds")?, "10");
    Ok(())
}

#[test]
fn malformed_cue_rows_fall_back_per_field() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    let store = SqliteStore::open(&path)?;
    let raw = Connection::open(&path)?;
    raw.execute(
        "INSERT INTO settings (key, value) VALUES ('theme', 'dark'), ('sound', 'loud'),
         ('vibration', 'off'), ('pre_alert_seconds', '7')",
        [],
    )?;
    assert_eq!(
        store.load()?,
        Settings::new(ThemeSelection::Dark, AccentColor::FALLBACK).with_vibration(Toggle::Off)
    );
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
    assert_eq!(
        store.load()?,
        Settings::from_storage(StoredSettings {
            theme: Some("sepia"),
            ..StoredSettings::default()
        })
    );

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
