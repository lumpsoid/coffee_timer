use crate::model::Settings;
use crate::ports::StoreError;

/// Persistent settings. Owned by `SettingsController`, which writes through
/// it before changing its own state.
pub trait SettingsStore {
    /// Missing or invalid stored values are not an error: build the result
    /// with [`Settings::from_storage`], which decides the per-field fallback.
    fn load(&self) -> Result<Settings, StoreError>;

    fn save(&mut self, settings: Settings) -> Result<(), StoreError>;
}
