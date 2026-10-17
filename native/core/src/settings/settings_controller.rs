use crate::model::{AccentColor, Settings, ThemeSelection};
use crate::ports::{SettingsStore, StoreError};

/// Owner of the user's settings. Every change is written through the store
/// first; a failed write leaves the state unchanged.
pub struct SettingsController {
    state: Settings,
    store: Box<dyn SettingsStore>,
}

impl SettingsController {
    /// Starts at [`Settings::FALLBACK`]; call [`SettingsController::load`] to
    /// read the store.
    pub fn new(store: Box<dyn SettingsStore>) -> Self {
        Self {
            state: Settings::FALLBACK,
            store,
        }
    }

    /// Replaces the state with the stored settings.
    pub fn load(&mut self) -> Result<(), StoreError> {
        self.state = self.store.load()?;
        Ok(())
    }

    pub fn set_theme(&mut self, theme: ThemeSelection) -> Result<(), StoreError> {
        self.apply(self.state.with_theme(theme))
    }

    /// Any valid colour is accepted, not only an [`crate::model::AccentOption`]:
    /// the legacy app stored and loaded any ARGB value.
    pub fn set_accent(&mut self, accent: AccentColor) -> Result<(), StoreError> {
        self.apply(self.state.with_accent(accent))
    }

    pub fn state(&self) -> &Settings {
        &self.state
    }

    fn apply(&mut self, next: Settings) -> Result<(), StoreError> {
        self.store.save(next)?;
        self.state = next;
        Ok(())
    }
}
