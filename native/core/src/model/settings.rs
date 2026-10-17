use crate::model::{AccentColor, ThemeSelection};

/// User settings. Change only through the named operations below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    theme: ThemeSelection,
    accent: AccentColor,
}

impl Settings {
    /// Defaults used before anything is loaded. The theme matches
    /// `ThemeSelection::from_storage(None)`.
    pub const FALLBACK: Self = Self {
        theme: ThemeSelection::System,
        accent: AccentColor::FALLBACK,
    };

    pub fn new(theme: ThemeSelection, accent: AccentColor) -> Self {
        Self { theme, accent }
    }

    pub fn with_theme(self, theme: ThemeSelection) -> Self {
        Self { theme, ..self }
    }

    pub fn with_accent(self, accent: AccentColor) -> Self {
        Self { accent, ..self }
    }

    pub fn theme(self) -> ThemeSelection {
        self.theme
    }

    pub fn accent(self) -> AccentColor {
        self.accent
    }
}
