//! Settings ↔ settings signals.

use coffee_timer_core::model::{AccentOption, Settings, ThemeSelection};

use crate::signals::{AccentChoice, SettingsUpdate, ThemeChoice};

impl From<&Settings> for SettingsUpdate {
    fn from(settings: &Settings) -> Self {
        Self {
            theme: settings.theme().into(),
            accent: settings.accent().argb(),
            themes: ThemeSelection::ALL.into_iter().map(Into::into).collect(),
            accents: AccentOption::ALL
                .into_iter()
                .map(|option| AccentChoice {
                    name: option.name().to_owned(),
                    argb: option.color().argb(),
                })
                .collect(),
        }
    }
}

impl From<ThemeSelection> for ThemeChoice {
    fn from(theme: ThemeSelection) -> Self {
        match theme {
            ThemeSelection::Light => Self::Light,
            ThemeSelection::Dark => Self::Dark,
            ThemeSelection::System => Self::System,
        }
    }
}

impl From<ThemeChoice> for ThemeSelection {
    fn from(choice: ThemeChoice) -> Self {
        match choice {
            ThemeChoice::Light => Self::Light,
            ThemeChoice::Dark => Self::Dark,
            ThemeChoice::System => Self::System,
        }
    }
}
