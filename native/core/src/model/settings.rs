use crate::model::{AccentColor, PreAlertLead, StoredSettings, ThemeSelection, Toggle};

/// User settings. Change only through the named operations below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    theme: ThemeSelection,
    accent: AccentColor,
    sound: Toggle,
    vibration: Toggle,
    pre_alert: PreAlertLead,
}

impl Settings {
    /// Defaults used before anything is loaded, and what
    /// [`Settings::from_storage`] gives for nothing stored: the system
    /// theme, Material blue, sound and vibration on, a 3 s pre-alert.
    pub const FALLBACK: Self = Self {
        theme: ThemeSelection::System,
        accent: AccentColor::FALLBACK,
        sound: Toggle::On,
        vibration: Toggle::On,
        pre_alert: PreAlertLead::DEFAULT,
    };

    /// `theme` and `accent`, with the fallback cues.
    pub fn new(theme: ThemeSelection, accent: AccentColor) -> Self {
        Self {
            theme,
            accent,
            ..Self::FALLBACK
        }
    }

    /// Builds settings from raw stored values, field by field: a missing or
    /// unrecognised value falls back to its [`Settings::FALLBACK`] field (a
    /// theme through [`ThemeSelection::from_storage`]). Never fails — bad
    /// storage never blocks the app.
    pub fn from_storage(stored: StoredSettings<'_>) -> Self {
        let fallback = Self::FALLBACK;
        let toggle = |raw: Option<&str>, fallback: Toggle| {
            raw.and_then(|raw| Toggle::try_parse(raw).ok())
                .unwrap_or(fallback)
        };
        Self {
            theme: ThemeSelection::from_storage(stored.theme),
            accent: stored
                .accent
                .and_then(|raw| AccentColor::try_from_hex(raw).ok())
                .unwrap_or(fallback.accent),
            sound: toggle(stored.sound, fallback.sound),
            vibration: toggle(stored.vibration, fallback.vibration),
            pre_alert: stored
                .pre_alert
                .and_then(|raw| PreAlertLead::try_parse(raw).ok())
                .unwrap_or(fallback.pre_alert),
        }
    }

    pub fn with_theme(self, theme: ThemeSelection) -> Self {
        Self { theme, ..self }
    }

    pub fn with_accent(self, accent: AccentColor) -> Self {
        Self { accent, ..self }
    }

    pub fn with_sound(self, sound: Toggle) -> Self {
        Self { sound, ..self }
    }

    pub fn with_vibration(self, vibration: Toggle) -> Self {
        Self { vibration, ..self }
    }

    pub fn with_pre_alert(self, pre_alert: PreAlertLead) -> Self {
        Self { pre_alert, ..self }
    }

    pub fn theme(self) -> ThemeSelection {
        self.theme
    }

    pub fn accent(self) -> AccentColor {
        self.accent
    }

    /// Whether a brew plays its step sound.
    pub fn sound(self) -> Toggle {
        self.sound
    }

    /// Whether a brew buzzes: step boundaries, the pre-alert and the
    /// finish.
    pub fn vibration(self) -> Toggle {
        self.vibration
    }

    pub fn pre_alert(self) -> PreAlertLead {
        self.pre_alert
    }
}
