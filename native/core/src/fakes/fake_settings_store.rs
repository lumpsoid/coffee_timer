use crate::model::{Settings, StoredSettings};
use crate::ports::{SettingsStore, StoreError};

/// In-memory [`SettingsStore`]. Empty, it loads what an empty real store
/// loads (`Settings::from_storage` of nothing). Can be told to fail loads or
/// writes.
#[derive(Debug, Clone, Default)]
pub struct FakeSettingsStore {
    stored: Option<Settings>,
    fail_loads: bool,
    fail_writes: bool,
}

impl FakeSettingsStore {
    /// A store already holding `settings`.
    pub fn with_settings(settings: Settings) -> Self {
        Self {
            stored: Some(settings),
            ..Self::default()
        }
    }

    /// Every `load` returns `Err`.
    pub fn failing_loads(self) -> Self {
        Self {
            fail_loads: true,
            ..self
        }
    }

    /// Every `save` returns `Err` and stores nothing.
    pub fn failing_writes(self) -> Self {
        Self {
            fail_writes: true,
            ..self
        }
    }
}

impl SettingsStore for FakeSettingsStore {
    fn load(&self) -> Result<Settings, StoreError> {
        if self.fail_loads {
            return Err(StoreError::new("fake load failure"));
        }
        Ok(self
            .stored
            .unwrap_or_else(|| Settings::from_storage(StoredSettings::default())))
    }

    fn save(&mut self, settings: Settings) -> Result<(), StoreError> {
        if self.fail_writes {
            return Err(StoreError::new("fake write failure"));
        }
        self.stored = Some(settings);
        Ok(())
    }
}
