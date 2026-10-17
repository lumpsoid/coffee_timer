//! Schema migrations keyed by `PRAGMA user_version`. Each entry takes the
//! database to its version from the one before; add a version by appending
//! one entry. A shipped entry is never edited.

use coffee_timer_core::ports::StoreError;
use rusqlite::Connection;

use crate::sqlite_store::db_error;

/// `(version, sql)`, in ascending version order starting at 1.
const MIGRATIONS: &[(u32, &str)] = &[(1, V1_SCHEMA_AND_SEED)];

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
    let latest = MIGRATIONS.last().map_or(0, |&(version, _)| version);
    if current > latest {
        return Err(StoreError::new(format!(
            "database schema v{current} is newer than this app (v{latest})"
        )));
    }
    for &(version, sql) in MIGRATIONS.iter().filter(|&&(version, _)| version > current) {
        tx.execute_batch(sql).map_err(db_error)?;
        tx.pragma_update(None, "user_version", version)
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
    fn fresh_database_ends_at_v1() -> TestResult {
        let mut conn = Connection::open_in_memory()?;
        migrate(&mut conn)?;
        assert_eq!(user_version(&conn)?, 1);
        Ok(())
    }

    #[test]
    fn migrating_twice_does_not_reseed() -> TestResult {
        let mut conn = Connection::open_in_memory()?;
        migrate(&mut conn)?;
        migrate(&mut conn)?;
        let recipes: i64 = conn.query_row("SELECT COUNT(*) FROM recipe", [], |r| r.get(0))?;
        assert_eq!(recipes, 2);
        assert_eq!(user_version(&conn)?, 1);
        Ok(())
    }

    #[test]
    fn newer_database_is_refused() -> TestResult {
        let mut conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "user_version", 2)?;
        assert!(migrate(&mut conn).is_err());
        Ok(())
    }
}
