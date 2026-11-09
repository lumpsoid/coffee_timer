//! Settings ↔ settings signals.

use coffee_timer_core::model::{AccentOption, PreAlertLead, Settings, Sound, ThemeSelection};
use coffee_timer_core::version::APP_VERSION;

use crate::signals::{AccentChoice, PreAlertChoice, SettingsUpdate, SoundChoice, ThemeChoice};

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
            step_sound: settings.step_sound().id().to_owned(),
            finish_sound: settings.finish_sound().id().to_owned(),
            sounds: Sound::ALL
                .into_iter()
                .map(|sound| SoundChoice {
                    id: sound.id().to_owned(),
                    label: sound.label().to_owned(),
                    length_label: sound.length_label(),
                })
                .collect(),
            app_version: APP_VERSION.to_owned(),
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

#[cfg(test)]
mod tests {
    use coffee_timer_core::model::{Settings, Sound};

    use crate::signals::SettingsUpdate;

    #[test]
    fn carries_the_chosen_sounds_and_every_option() {
        let settings = Settings::FALLBACK
            .with_step_sound(Sound::Kalimba)
            .with_finish_sound(Sound::Tingsha);
        let update = SettingsUpdate::from(&settings);
        assert_eq!(update.step_sound, "kalimba");
        assert_eq!(update.finish_sound, "tingsha");
        let rows: Vec<(&str, &str, &str)> = update
            .sounds
            .iter()
            .map(|row| {
                (
                    row.id.as_str(),
                    row.label.as_str(),
                    row.length_label.as_str(),
                )
            })
            .collect();
        assert_eq!(rows.len(), Sound::ALL.len());
        assert_eq!(rows[0], ("temple-block", "Temple block", "1.0 s"));
        assert_eq!(rows[7], ("singing-bowl", "Singing bowl", "2.0 s"));
    }
}
