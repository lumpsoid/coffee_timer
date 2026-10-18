use crate::model::ModelError;

/// How long before each step ends the brew gives its pre-alert buzz: off,
/// or one of the few leads the Settings page offers. A closed set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreAlertLead {
    Off,
    Seconds3,
    Seconds5,
    Seconds10,
}

impl PreAlertLead {
    /// Every option, in the picker's order.
    pub const ALL: [Self; 4] = [Self::Off, Self::Seconds3, Self::Seconds5, Self::Seconds10];

    /// The lead before anything is stored, and for a stored value that is
    /// not an option.
    pub const DEFAULT: Self = Self::Seconds3;

    /// Seconds left in a step when the pre-alert fires; 0 when off.
    pub fn seconds(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Seconds3 => 3,
            Self::Seconds5 => 5,
            Self::Seconds10 => 10,
        }
    }

    /// The picker's text: `Off`, `3 s`, `5 s`, `10 s`.
    pub fn label(self) -> String {
        match self {
            Self::Off => "Off".to_owned(),
            lead => format!("{} s", lead.seconds()),
        }
    }

    /// The option of `seconds`; any other number is
    /// [`ModelError::UnknownPreAlert`].
    pub fn try_from_seconds(seconds: u32) -> Result<Self, ModelError> {
        Self::ALL
            .into_iter()
            .find(|lead| lead.seconds() == seconds)
            .ok_or_else(|| ModelError::UnknownPreAlert(seconds.to_string()))
    }

    /// Parses the seconds as typed or stored (`0`, `3`, `5`, `10`)
    /// strictly. For user input; storage reads go through
    /// [`crate::model::Settings::from_storage`].
    pub fn try_parse(raw: &str) -> Result<Self, ModelError> {
        raw.parse::<u32>()
            .ok()
            .and_then(|seconds| Self::try_from_seconds(seconds).ok())
            .ok_or_else(|| ModelError::UnknownPreAlert(raw.to_owned()))
    }

    /// The string persisted in storage: the seconds.
    pub fn storage_value(self) -> String {
        self.seconds().to_string()
    }
}
