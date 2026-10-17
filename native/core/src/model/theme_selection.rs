use crate::model::ModelError;

/// The theme options the user can choose. A closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeSelection {
    Light,
    Dark,
    System,
}

impl ThemeSelection {
    /// The string persisted in storage.
    pub fn storage_value(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        }
    }

    /// Every option, in the legacy picker's order.
    pub const ALL: [Self; 3] = [Self::Light, Self::Dark, Self::System];

    /// Parses a [`ThemeSelection::storage_value`] strictly: anything else is
    /// [`ModelError::UnknownTheme`]. For user input; storage reads use
    /// [`ThemeSelection::from_storage`].
    pub fn try_parse(raw: &str) -> Result<Self, ModelError> {
        Self::ALL
            .into_iter()
            .find(|theme| theme.storage_value() == raw)
            .ok_or_else(|| ModelError::UnknownTheme(raw.to_owned()))
    }

    /// Parses a stored value, falling back to `System` for anything
    /// unrecognised or missing.
    pub fn from_storage(value: Option<&str>) -> Self {
        value
            .and_then(|raw| Self::try_parse(raw).ok())
            .unwrap_or(Self::System)
    }
}
