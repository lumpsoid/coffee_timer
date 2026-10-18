//! Settings ↔ settings signals.

use coffee_timer_core::model::{AccentOption, PreAlertLead, Settings, ThemeSelection};

use crate::signals::{AccentChoice, PreAlertChoice, SettingsUpdate, ThemeChoice};

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
            sound: settings.sound().is_on(),
            vibration: settings.vibration().is_on(),
            pre_alert_seconds: settings.pre_alert().seconds(),
            pre_alerts: PreAlertLead::ALL
                .into_iter()
                .map(|lead| PreAlertChoice {
                    seconds: lead.seconds(),
                    label: lead.label(),
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
