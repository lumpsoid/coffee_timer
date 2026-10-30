//! `SqliteStore`: schema v6 and seed, recipe doses, pour speeds, grinds, step pours, recipe pins, recipe order and reordering, appending in one transaction, step replacement,
//! settings round trip and fallback, and the core controllers over it.

use std::path::Path;

use coffee_timer_core::catalog::{CatalogChange, RecipeCatalog};
use coffee_timer_core::model::{
    AccentColor, CoffeeDose, GrindClicks, Pin, PourSpeed, PreAlertLead, Recipe, RecipeId,
    RecipeName, RecipeStep, Settings, StepDuration, StepId, StoredSettings, ThemeSelection, Toggle,
    WaterAmount,
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
fn pour_speeds_round_trip_and_follow_an_upsert() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    let first = recipe("r1", "V60", vec![step("s1", "Pour", 90)?])?;
    store.upsert(&first)?;
    let speed = |store: &SqliteStore| -> Outcome<Vec<String>> {
        Ok(store
            .load_all()?
            .iter()
            .map(|r| r.pour_speed().text())
            .collect())
    };
    assert_eq!(speed(&store)?, ["5", "5", "5"], "seeds and new recipe");
    store.upsert(&first.with_pour_speed(PourSpeed::try_parse_text("6.5")?))?;
    assert_eq!(speed(&store)?, ["5", "5", "6.5"]);
    Ok(())
}

#[test]
fn stored_pour_speeds_out_of_range_are_clamped() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    drop(SqliteStore::open(&path)?);
    Connection::open(&path)?.execute_batch(
        "UPDATE recipe SET pour_speed_tenths = 0 WHERE id = 'espresso-uuid';
         UPDATE recipe SET pour_speed_tenths = 99999 WHERE id = 'pourover-uuid';",
    )?;
    let speeds: Vec<_> = SqliteStore::open(&path)?
        .load_all()?
        .iter()
        .map(Recipe::pour_speed)
        .collect();
    assert_eq!(speeds, [PourSpeed::MIN, PourSpeed::MAX]);
    Ok(())
}

#[test]
fn grinds_round_trip_and_follow_an_upsert() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    let first = recipe("r1", "V60", vec![step("s1", "Pour", 90)?])?;
    store.upsert(&first)?;
    let grinds = |store: &SqliteStore| -> Outcome<Vec<Option<u32>>> {
        Ok(store
            .load_all()?
            .iter()
            .map(|r| r.grind().map(GrindClicks::clicks))
            .collect())
    };
    assert_eq!(grinds(&store)?, [None, None, None], "seeds and new recipe");
    let ground = first.with_grind(Some(GrindClicks::try_parse(22)?));
    store.upsert(&ground)?;
    assert_eq!(grinds(&store)?, [None, None, Some(22)]);
    store.upsert(&ground.with_grind(None))?;
    assert_eq!(grinds(&store)?, [None, None, None], "cleared");
    Ok(())
}

#[test]
fn appended_and_reinserted_recipes_keep_their_grind() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    let ground = |id: &str| -> Outcome<Recipe> {
        Ok(recipe(id, "V60", Vec::new())?.with_grind(Some(GrindClicks::try_parse(18)?)))
    };
    store.append_all(&[ground("a")?])?;
    store.insert_at(&ground("b")?, 0, Pin::Unpinned)?;
    let grinds: Vec<_> = store
        .load_all()?
        .iter()
        .map(|r| r.grind().map(GrindClicks::clicks))
        .collect();
    assert_eq!(grinds, [Some(18), None, None, Some(18)]);
    Ok(())
}

#[test]
fn stored_grinds_out_of_range_are_clamped() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    drop(SqliteStore::open(&path)?);
    Connection::open(&path)?.execute_batch(
        "UPDATE recipe SET grind_clicks = -3 WHERE id = 'espresso-uuid';
         UPDATE recipe SET grind_clicks = 500 WHERE id = 'pourover-uuid';",
    )?;
    let grinds: Vec<_> = SqliteStore::open(&path)?
        .load_all()?
        .iter()
        .map(|r| r.grind().map(GrindClicks::clicks))
        .collect();
    assert_eq!(grinds, [Some(0), Some(99)]);
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
fn opened_file_is_at_schema_v6() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    SqliteStore::open(&path)?;
    let version: u32 =
        Connection::open(&path)?.pragma_query_value(None, "user_version", |row| row.get(0))?;
    assert_eq!(version, 6);
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

    store.insert_at(&removed, 1, Pin::Unpinned)?;

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
    store.insert_at(&recipe("first", "V60", Vec::new())?, 0, Pin::Unpinned)?;
    store.insert_at(&recipe("last", "Aeropress", Vec::new())?, 99, Pin::Unpinned)?;
    assert_eq!(
        ids(&store.load_all()?),
        ["first", "espresso-uuid", "pourover-uuid", "last"]
    );
    Ok(())
}

#[test]
fn append_all_adds_every_recipe_unpinned_at_the_end_with_its_steps() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    store.set_pin(&rid("pourover-uuid")?, Pin::Pinned)?;
    let v60 = recipe(
        "v60",
        "V60",
        vec![step("v1", "Bloom", 30)?, step("v2", "Pour", 90)?],
    )?;
    let aero = recipe("aero", "Aeropress", vec![step("a1", "Steep", 60)?])?;
    store.append_all(&[v60.clone(), aero.clone()])?;

    let loaded = store.load_all()?;
    assert_eq!(
        ids(&loaded),
        ["espresso-uuid", "pourover-uuid", "v60", "aero"]
    );
    assert_eq!(shape(&loaded[2..]), shape(&[v60, aero]));
    assert_eq!(pinned_ids(&store)?, ["pourover-uuid"]);
    Ok(())
}

#[test]
fn append_all_with_a_known_id_appends_none() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    let fresh = recipe("fresh", "V60", vec![step("f1", "Bloom", 30)?])?;
    let known = recipe("espresso-uuid", "Again", Vec::new())?;
    assert!(store.append_all(&[fresh, known]).is_err());
    assert_eq!(ids(&store.load_all()?), ["espresso-uuid", "pourover-uuid"]);
    Ok(())
}

fn rid(id: &str) -> Outcome<RecipeId> {
    Ok(RecipeId::try_parse(id)?)
}

fn pinned_ids(store: &SqliteStore) -> Outcome<Vec<String>> {
    Ok(store
        .load_pinned()?
        .iter()
        .map(|id| id.as_str().to_owned())
        .collect())
}

#[test]
fn fresh_database_pins_nothing() -> TestResult {
    let store = SqliteStore::open_in_memory()?;
    assert!(store.load_pinned()?.is_empty());
    Ok(())
}

#[test]
fn set_pin_round_trips_in_stored_order_and_keeps_positions() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    store.upsert(&recipe("r1", "V60", Vec::new())?)?;
    store.set_pin(&rid("r1")?, Pin::Pinned)?;
    store.set_pin(&rid("espresso-uuid")?, Pin::Pinned)?;
    assert_eq!(pinned_ids(&store)?, ["espresso-uuid", "r1"]);
    assert_eq!(
        ids(&store.load_all()?),
        ["espresso-uuid", "pourover-uuid", "r1"],
        "a pin moves no row"
    );

    store.set_pin(&rid("espresso-uuid")?, Pin::Unpinned)?;
    assert_eq!(pinned_ids(&store)?, ["r1"]);
    store.set_pin(&rid("missing")?, Pin::Pinned)?;
    assert_eq!(pinned_ids(&store)?, ["r1"], "an unknown id changes nothing");
    Ok(())
}

#[test]
fn upsert_keeps_a_pin_and_a_new_recipe_is_unpinned() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    store.set_pin(&rid("pourover-uuid")?, Pin::Pinned)?;
    store.upsert(&recipe("pourover-uuid", "Renamed", Vec::new())?)?;
    store.upsert(&recipe("r1", "V60", Vec::new())?)?;
    assert_eq!(pinned_ids(&store)?, ["pourover-uuid"]);
    Ok(())
}

#[test]
fn insert_at_stores_the_pin_it_is_given() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    store.insert_at(&recipe("r1", "V60", Vec::new())?, 1, Pin::Pinned)?;
    assert_eq!(pinned_ids(&store)?, ["r1"]);
    assert_eq!(
        ids(&store.load_all()?),
        ["espresso-uuid", "r1", "pourover-uuid"]
    );
    Ok(())
}

#[test]
fn a_removed_and_reinserted_recipe_takes_only_the_pin_it_is_given() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    store.set_pin(&rid("espresso-uuid")?, Pin::Pinned)?;
    let espresso = store.load_all()?.into_iter().next().ok_or("no espresso")?;
    store.remove(espresso.id())?;
    assert!(store.load_pinned()?.is_empty());
    store.insert_at(&espresso, 0, Pin::Unpinned)?;
    assert!(store.load_pinned()?.is_empty());
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

#[test]
fn the_catalog_lists_pins_first_after_a_reopen() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("coffee.db");
    let mut catalog = RecipeCatalog::new(Box::new(SqliteStore::open(&path)?));
    catalog.load()?;
    catalog.upsert(recipe("r1", "V60", Vec::new())?)?;
    catalog.set_pin(&rid("r1")?, Pin::Pinned)?;
    catalog.set_pin(&rid("pourover-uuid")?, Pin::Pinned)?;
    drop(catalog);

    let mut catalog = RecipeCatalog::new(Box::new(SqliteStore::open(&path)?));
    catalog.load()?;
    let listed: Vec<&str> = catalog.listed().iter().map(|r| r.id().as_str()).collect();
    assert_eq!(listed, ["pourover-uuid", "r1", "espresso-uuid"]);
    assert_eq!(
        ids(catalog.state()),
        ["espresso-uuid", "pourover-uuid", "r1"]
    );

    catalog.remove(rid("pourover-uuid")?)?;
    catalog.restore(&rid("pourover-uuid")?)?;
    drop(catalog);
    let mut catalog = RecipeCatalog::new(Box::new(SqliteStore::open(&path)?));
    catalog.load()?;
    assert_eq!(catalog.pin_of(&rid("pourover-uuid")?), Pin::Pinned);
    assert_eq!(catalog.index_of(&rid("pourover-uuid")?), Some(0));
    Ok(())
}

#[test]
fn reorder_rewrites_the_stored_order_and_keeps_pins_and_steps() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    store.upsert(&recipe("r1", "V60", vec![step("s1", "Bloom", 30)?])?)?;
    store.set_pin(&rid("pourover-uuid")?, Pin::Pinned)?;
    store.reorder(&[rid("r1")?, rid("pourover-uuid")?, rid("espresso-uuid")?])?;
    let recipes = store.load_all()?;
    assert_eq!(ids(&recipes), ["r1", "pourover-uuid", "espresso-uuid"]);
    assert_eq!(pinned_ids(&store)?, ["pourover-uuid"]);
    assert_eq!(recipes.first().map(|r| r.steps().len()), Some(1));

    store.upsert(&recipe("r2", "Aeropress", Vec::new())?)?;
    store.insert_at(&recipe("r3", "Chemex", Vec::new())?, 1, Pin::Unpinned)?;
    assert_eq!(
        ids(&store.load_all()?),
        ["r1", "r3", "pourover-uuid", "espresso-uuid", "r2"],
        "later writes still append and insert"
    );
    Ok(())
}

#[test]
fn reorder_puts_unnamed_rows_after_and_skips_unknown_ids() -> TestResult {
    let mut store = SqliteStore::open_in_memory()?;
    store.upsert(&recipe("r1", "V60", Vec::new())?)?;
    store.reorder(&[rid("r1")?, rid("missing")?])?;
    assert_eq!(
        ids(&store.load_all()?),
        ["r1", "espresso-uuid", "pourover-uuid"]
    );
    Ok(())
}

#[test]
fn a_moved_recipe_keeps_its_place_after_reopen() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("recipes.db");
    let mut catalog = RecipeCatalog::new(Box::new(SqliteStore::open(&path)?));
    catalog.load()?;
    catalog.move_to(&rid("pourover-uuid")?, 0)?;

    let mut reopened = RecipeCatalog::new(Box::new(SqliteStore::open(&path)?));
    reopened.load()?;
    assert_eq!(ids(reopened.state()), ["pourover-uuid", "espresso-uuid"]);
    Ok(())
}
