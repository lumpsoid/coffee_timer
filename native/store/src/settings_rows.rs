use coffee_timer_core::model::Settings;
use coffee_timer_core::ports::{SettingsStore, StoreError};
use rusqlite::{Connection, OptionalExtension};

use crate::sqlite_store::{SqliteStore, db_error};

/// Key/value rows, as legacy `sqlite_settings_repository.dart:18-20`: the
/// theme's storage value and the accent as `#AARRGGBB`.
const THEME_KEY: &str = "theme";
const ACCENT_KEY: &str = "accent_color";

impl SettingsStore for SqliteStore {
    fn load(&self) -> Result<Settings, StoreError> {
        let conn = self.conn()?;
        let theme = value(&conn, THEME_KEY).map_err(db_error)?;
        let accent = value(&conn, ACCENT_KEY).map_err(db_error)?;
        Ok(Settings::from_storage(theme.as_deref(), accent.as_deref()))
    }

    fn save(&mut self, settings: Settings) -> Result<(), StoreError> {
        save(&mut *self.conn()?, settings).map_err(db_error)
    }
}

fn value(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
        row.get(0)
    })
    .optional()
}

fn save(conn: &mut Connection, settings: Settings) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    {
        let mut put = tx.prepare(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        )?;
        put.execute([THEME_KEY, settings.theme().storage_value()])?;
        put.execute([ACCENT_KEY, settings.accent().to_hex().as_str()])?;
    }
    tx.commit()
}
