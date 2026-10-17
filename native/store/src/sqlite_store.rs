use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use coffee_timer_core::ports::StoreError;
use rusqlite::Connection;

use crate::migrations;

/// One migrated SQLite connection that implements both `RecipeStore` and
/// `SettingsStore`. Clones share the connection, so the hub can hand one
/// clone to each controller; an in-memory database stays a single database.
#[derive(Clone)]
pub struct SqliteStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStore {
    /// Opens (creating if needed) the database file at `path` and brings its
    /// schema up to date.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        Self::ready(Connection::open(path).map_err(db_error)?)
    }

    /// A fresh, migrated database that lives as long as this store and its
    /// clones.
    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::ready(Connection::open_in_memory().map_err(db_error)?)
    }

    /// Migrates with foreign keys off, then turns them on. A future migration
    /// that rebuilds `recipe` (create, copy, drop, rename) would otherwise
    /// cascade-delete every step when the old table is dropped; `migrate`
    /// runs `foreign_key_check` before committing instead.
    fn ready(mut conn: Connection) -> Result<Self, StoreError> {
        migrations::migrate(&mut conn)?;
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(db_error)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub(crate) fn conn(&self) -> Result<MutexGuard<'_, Connection>, StoreError> {
        self.conn
            .lock()
            .map_err(|_| StoreError::new("database connection lock poisoned"))
    }
}

pub(crate) fn db_error(error: rusqlite::Error) -> StoreError {
    StoreError::new(error.to_string())
}
