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

    /// Parses a stored value, falling back to `System` for anything
    /// unrecognised or missing.
    pub fn from_storage(value: Option<&str>) -> Self {
        match value {
            Some("light") => Self::Light,
            Some("dark") => Self::Dark,
            _ => Self::System,
        }
    }
}
