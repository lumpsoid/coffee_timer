use crate::model::ModelError;

/// A cue that is on or off: the brew's sound, or its vibration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Toggle {
    On,
    Off,
}

impl Toggle {
    /// Both values, in the CLI's order.
    pub const ALL: [Self; 2] = [Self::On, Self::Off];

    pub fn is_on(self) -> bool {
        self == Self::On
    }

    /// The string persisted in storage, and typed on the CLI.
    pub fn storage_value(self) -> &'static str {
        match self {
            Self::On => "on",
            Self::Off => "off",
        }
    }

    /// Parses a [`Toggle::storage_value`] strictly: anything else is
    /// [`ModelError::UnknownToggle`]. For user input; storage reads go
    /// through [`crate::model::Settings::from_storage`].
    pub fn try_parse(raw: &str) -> Result<Self, ModelError> {
        Self::ALL
            .into_iter()
            .find(|toggle| toggle.storage_value() == raw)
            .ok_or_else(|| ModelError::UnknownToggle(raw.to_owned()))
    }
}

impl From<bool> for Toggle {
    fn from(on: bool) -> Self {
        if on { Self::On } else { Self::Off }
    }
}
