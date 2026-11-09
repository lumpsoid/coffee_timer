use coffee_timer_core::model::{Settings, StoredSettings};
use coffee_timer_core::ports::{SettingsStore, StoreError};
use rusqlite::{Connection, OptionalExtension};

use crate::sqlite_store::{SqliteStore, db_error};

/// Key/value rows, as legacy `sqlite_settings_repository.dart:18-20`: the
/// theme's storage value and the accent as `#AARRGGBB`. The brew's cues,
/// new since the port, follow the same scheme: sound and vibration as
/// `on`/`off`, the pre-alert lead in seconds, the step and finish sounds as
/// their ids. A database without their rows (one from before them) loads
/// their fallbacks, so no migration is needed.
const THEME_KEY: &str = "theme";
const ACCENT_KEY: &str = "accent_color";
const SOUND_KEY: &str = "sound";
const VIBRATION_KEY: &str = "vibration";
const PRE_ALERT_KEY: &str = "pre_alert_seconds";
const STEP_SOUND_KEY: &str = "step_sound";
const FINISH_SOUND_KEY: &str = "finish_sound";

impl SettingsStore for SqliteStore {
    fn load(&self) -> Result<Settings, StoreError> {
        let conn = self.conn()?;
        let read = |key| value(&conn, key).map_err(db_error);
        let theme = read(THEME_KEY)?;
        let accent = read(ACCENT_KEY)?;
        let sound = read(SOUND_KEY)?;
        let vibration = read(VIBRATION_KEY)?;
        let pre_alert = read(PRE_ALERT_KEY)?;
        let step_sound = read(STEP_SOUND_KEY)?;
        let finish_sound = read(FINISH_SOUND_KEY)?;
        Ok(Settings::from_storage(StoredSettings {
            theme: theme.as_deref(),
            accent: accent.as_deref(),
            sound: sound.as_deref(),
            vibration: vibration.as_deref(),
            pre_alert: pre_alert.as_deref(),
            step_sound: step_sound.as_deref(),
            finish_sound: finish_sound.as_deref(),
        }))
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
        put.execute([SOUND_KEY, settings.sound().storage_value()])?;
        put.execute([VIBRATION_KEY, settings.vibration().storage_value()])?;
        put.execute([PRE_ALERT_KEY, settings.pre_alert().storage_value().as_str()])?;
        put.execute([STEP_SOUND_KEY, settings.step_sound().id()])?;
        put.execute([FINISH_SOUND_KEY, settings.finish_sound().id()])?;
    }
    tx.commit()
}
