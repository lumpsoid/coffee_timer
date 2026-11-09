//! Settings ↔ settings signals.

use coffee_timer_core::model::{
    AccentOption, FinishSound, PreAlertLead, Settings, Sound, StepSound, ThemeSelection,
};
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
            step_sounds: StepSound::ALL.into_iter().map(Into::into).collect(),
            finish_sounds: FinishSound::ALL.into_iter().map(Into::into).collect(),
            app_version: APP_VERSION.to_owned(),
        }
    }
}

impl<S: Into<Sound>> From<S> for SoundChoice {
    fn from(sound: S) -> Self {
        let sound = sound.into();
        Self {
            id: sound.id().to_owned(),
            label: sound.label().to_owned(),
            length_label: sound.length_label(),
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
    use coffee_timer_core::model::{FinishSound, Settings, StepSound};

    use crate::signals::{SettingsUpdate, SoundChoice};

    fn rows(choices: &[SoundChoice]) -> Vec<(&str, &str, &str)> {
        choices
            .iter()
            .map(|row| {
                (
                    row.id.as_str(),
                    row.label.as_str(),
                    row.length_label.as_str(),
                )
            })
            .collect()
    }

    #[test]
    fn carries_the_chosen_sounds_and_each_pickers_options() {
        let settings = Settings::FALLBACK
            .with_step_sound(StepSound::Kalimba)
            .with_finish_sound(FinishSound::Gong);
        let update = SettingsUpdate::from(&settings);
        assert_eq!(update.step_sound, "kalimba");
        assert_eq!(update.finish_sound, "gong");
        let steps = rows(&update.step_sounds);
        assert_eq!(steps.len(), StepSound::ALL.len());
        assert_eq!(steps[0], ("temple-block", "Temple block", "1.0 s"));
        let finishes = rows(&update.finish_sounds);
        assert_eq!(finishes.len(), FinishSound::ALL.len());
        assert_eq!(finishes[0], ("singing-bowl", "Singing bowl", "3.0 s"));
        assert_eq!(finishes[1], ("bowl-twice", "Bowl struck twice", "4.6 s"));
    }
}
