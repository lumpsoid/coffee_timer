//! Schema migrations keyed by `PRAGMA user_version`. Each entry takes the
//! database to its version from the one before; add a version by appending
//! one entry. A shipped entry is never edited.
//!
//! An entry may also carry `fresh` SQL, run right after its own only on a
//! database this same migration created (from version 0). That lets a new
//! database's seed differ from what an upgraded database keeps, while every
//! statement still runs against the schema of its own version.

use coffee_timer_core::ports::StoreError;
use rusqlite::Connection;

use crate::sqlite_store::db_error;

/// One schema version: `sql` takes the database to `version`; `fresh`, if
/// any, then runs only on a database created in this migration.
struct Migration {
    version: u32,
    sql: &'static str,
    fresh: Option<&'static str>,
}

/// In ascending version order starting at 1.
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        sql: V1_SCHEMA_AND_SEED,
        fresh: None,
    },
    Migration {
        version: 2,
        sql: V2_RECIPE_DOSE,
        fresh: Some(V2_FRESH_SEED_DOSE),
    },
    Migration {
        version: 3,
        sql: V3_STEP_WATER,
        fresh: Some(V3_FRESH_SEED_POURS),
    },
    Migration {
        version: 4,
        sql: V4_RECIPE_PIN,
        fresh: None,
    },
];

/// v1: tables, plus the two example recipes. Seeding is part of creating the
/// database, so it happens once: emptying the catalog does not re-seed
/// (legacy `app_migrations.dart` v2).
const V1_SCHEMA_AND_SEED: &str = "
CREATE TABLE recipe (
    position    INTEGER PRIMARY KEY,
    id          TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE step (
    id               TEXT PRIMARY KEY NOT NULL,
    recipe_id        TEXT NOT NULL REFERENCES recipe (id) ON DELETE CASCADE,
    position         INTEGER NOT NULL,
    name             TEXT NOT NULL,
    duration_seconds INTEGER NOT NULL,
    UNIQUE (recipe_id, position)
);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

INSERT INTO recipe (position, id, name, description) VALUES
    (1, 'espresso-uuid', 'Espresso', 'Strong coffee'),
    (2, 'pourover-uuid', 'Pour Over', 'Light and clean');

INSERT INTO step (id, recipe_id, position, name, duration_seconds) VALUES
    ('step1', 'espresso-uuid', 0, 'Grind', 30),
    ('step2', 'espresso-uuid', 1, 'Brew', 25),
    ('step3', 'pourover-uuid', 0, 'Grind', 40),
    ('step4', 'pourover-uuid', 1, 'Bloom', 30),
    ('step5', 'pourover-uuid', 2, 'Pour', 120);
";

/// v2: a recipe stores its coffee dose and total water in grams. Until v1
/// they were derived from the step count (10 g coffee and 50 g water per
/// step), so an upgraded recipe gets exactly that and shows what it showed.
/// The core reads the values leniently (clamped into its bounds), so the
/// bounds are not repeated here.
const V2_RECIPE_DOSE: &str = "
ALTER TABLE recipe ADD COLUMN coffee_grams INTEGER NOT NULL DEFAULT 0;
ALTER TABLE recipe ADD COLUMN water_grams INTEGER NOT NULL DEFAULT 0;

UPDATE recipe SET
    coffee_grams = 10 * (SELECT COUNT(*) FROM step WHERE step.recipe_id = recipe.id),
    water_grams  = 50 * (SELECT COUNT(*) FROM step WHERE step.recipe_id = recipe.id);
";

/// v2 on a new database: the seeded recipes get real doses instead.
const V2_FRESH_SEED_DOSE: &str = "
UPDATE recipe SET coffee_grams = 18, water_grams = 36 WHERE id = 'espresso-uuid';
UPDATE recipe SET coffee_grams = 15, water_grams = 250 WHERE id = 'pourover-uuid';
";

/// v3: a step may store the water poured in it, in grams. NULL is a step
/// that pours nothing, which is what every step before v3 was. The core
/// reads the value leniently (clamped into its bounds).
const V3_STEP_WATER: &str = "
ALTER TABLE step ADD COLUMN water_grams INTEGER;
";

/// v3 on a new database: the seeded Pour Over pours its 250 g in a bloom
/// and one main pour.
const V3_FRESH_SEED_POURS: &str = "
UPDATE step SET water_grams = 50 WHERE id = 'step4';
UPDATE step SET water_grams = 200 WHERE id = 'step5';
";

/// v4: a recipe may be pinned to the top of the list. Every recipe before
/// v4 is unpinned, so the list shows what it showed; positions are kept.
const V4_RECIPE_PIN: &str = "
ALTER TABLE recipe ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;
";

/// Applies every pending migration in one transaction: the database ends at
/// the latest version or is left as it was. A database newer than this build
/// is refused rather than written with an older schema's assumptions.
///
/// Runs with foreign keys off (see `SqliteStore::ready`), so it checks them
/// itself before committing: a migration that leaves a dangling reference
/// fails and is rolled back.
pub(crate) fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    let tx = conn.transaction().map_err(db_error)?;
    let current: u32 = tx
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(db_error)?;
    let latest = MIGRATIONS.last().map_or(0, |m| m.version);
    if current > latest {
        return Err(StoreError::new(format!(
            "database schema v{current} is newer than this app (v{latest})"
        )));
    }
    let created = current == 0;
    for m in MIGRATIONS.iter().filter(|m| m.version > current) {
        tx.execute_batch(m.sql).map_err(db_error)?;
        if let (true, Some(fresh)) = (created, m.fresh) {
            tx.execute_batch(fresh).map_err(db_error)?;
        }
        tx.pragma_update(None, "user_version", m.version)
            .map_err(db_error)?;
    }
    let violations: i64 = tx
        .query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .map_err(db_error)?;
    if violations > 0 {
        return Err(StoreError::new(format!(
            "migration left {violations} foreign key violation(s)"
        )));
    }
    tx.commit().map_err(db_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn user_version(conn: &Connection) -> rusqlite::Result<u32> {
        conn.pragma_query_value(None, "user_version", |row| row.get(0))
    }

    #[test]
    fn fresh_database_ends_at_v4() -> TestResult {
        let mut conn = Connection::open_in_memory()?;
        migrate(&mut conn)?;
        assert_eq!(user_version(&conn)?, 4);
        Ok(())
    }

    fn doses(conn: &Connection) -> rusqlite::Result<Vec<(String, i64, i64)>> {
        conn.prepare("SELECT id, coffee_grams, water_grams FROM recipe ORDER BY position")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect()
    }

    #[test]
    fn fresh_database_seeds_real_doses() -> TestResult {
        let mut conn = Connection::open_in_memory()?;
        migrate(&mut conn)?;
        assert_eq!(
            doses(&conn)?,
            [
                ("espresso-uuid".to_owned(), 18, 36),
                ("pourover-uuid".to_owned(), 15, 250),
            ]
        );
        Ok(())
    }

    #[test]
    fn v1_recipes_keep_the_dose_their_step_count_gave() -> TestResult {
        let mut conn = Connection::open_in_memory()?;
        conn.execute_batch(V1_SCHEMA_AND_SEED)?;
        conn.execute_batch(
            "INSERT INTO recipe (id, name, description) VALUES ('none', 'No steps', '');
             UPDATE recipe SET name = 'Mine' WHERE id = 'espresso-uuid';",
        )?;
        conn.pragma_update(None, "user_version", 1)?;
        migrate(&mut conn)?;
        assert_eq!(user_version(&conn)?, 4);
        assert_eq!(
            doses(&conn)?,
            [
                ("espresso-uuid".to_owned(), 20, 100),
                ("pourover-uuid".to_owned(), 30, 150),
                ("none".to_owned(), 0, 0),
            ]
        );
        Ok(())
    }

    fn pours(conn: &Connection) -> rusqlite::Result<Vec<(String, Option<i64>)>> {
        conn.prepare("SELECT id, water_grams FROM step ORDER BY id")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect()
    }

    #[test]
    fn fresh_database_seeds_the_pour_over_pours() -> TestResult {
        let mut conn = Connection::open_in_memory()?;
        migrate(&mut conn)?;
        assert_eq!(
            pours(&conn)?,
            [
                ("step1".to_owned(), None),
                ("step2".to_owned(), None),
                ("step3".to_owned(), None),
                ("step4".to_owned(), Some(50)),
                ("step5".to_owned(), Some(200)),
            ]
        );
        Ok(())
    }

    #[test]
    fn v2_steps_pour_nothing() -> TestResult {
        let mut conn = Connection::open_in_memory()?;
        conn.execute_batch(V1_SCHEMA_AND_SEED)?;
        conn.execute_batch(V2_RECIPE_DOSE)?;
        conn.pragma_update(None, "user_version", 2)?;
        migrate(&mut conn)?;
        assert_eq!(user_version(&conn)?, 4);
        assert!(pours(&conn)?.iter().all(|(_, water)| water.is_none()));
        Ok(())
    }

    fn pins(conn: &Connection) -> rusqlite::Result<Vec<(i64, String, i64)>> {
        conn.prepare("SELECT position, id, pinned FROM recipe ORDER BY position")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
            .collect()
    }

    #[test]
    fn v3_recipes_are_unpinned_and_keep_their_positions() -> TestResult {
        let mut conn = Connection::open_in_memory()?;
        conn.execute_batch(V1_SCHEMA_AND_SEED)?;
        conn.execute_batch(V2_RECIPE_DOSE)?;
        conn.execute_batch(V3_STEP_WATER)?;
        conn.execute_batch(
            "INSERT INTO recipe (position, id, name, description) VALUES (7, 'mine', 'Mine', '');",
        )?;
        conn.pragma_update(None, "user_version", 3)?;
        migrate(&mut conn)?;
        assert_eq!(user_version(&conn)?, 4);
        assert_eq!(
            pins(&conn)?,
            [
                (1, "espresso-uuid".to_owned(), 0),
                (2, "pourover-uuid".to_owned(), 0),
                (7, "mine".to_owned(), 0),
            ]
        );
        Ok(())
    }

    #[test]
    fn migrating_twice_does_not_reseed() -> TestResult {
        let mut conn = Connection::open_in_memory()?;
        migrate(&mut conn)?;
        migrate(&mut conn)?;
        let recipes: i64 = conn.query_row("SELECT COUNT(*) FROM recipe", [], |r| r.get(0))?;
        assert_eq!(recipes, 2);
        assert_eq!(user_version(&conn)?, 4);
        Ok(())
    }

    #[test]
    fn newer_database_is_refused() -> TestResult {
        let mut conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "user_version", 5)?;
        assert!(migrate(&mut conn).is_err());
        Ok(())
    }
}
