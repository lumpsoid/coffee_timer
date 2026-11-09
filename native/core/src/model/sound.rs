use crate::model::{FinishSound, ModelError, StepSound};

/// Any sound the app ships: a [`StepSound`] or a [`FinishSound`]. What a
/// brew cue or a preview plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sound {
    Step(StepSound),
    Finish(FinishSound),
}

impl Sound {
    /// The id: the asset's file stem.
    pub fn id(self) -> &'static str {
        match self {
            Self::Step(sound) => sound.id(),
            Self::Finish(sound) => sound.id(),
        }
    }

    /// The picker's name: `Temple block`.
    pub fn label(self) -> &'static str {
        match self {
            Self::Step(sound) => sound.label(),
            Self::Finish(sound) => sound.label(),
        }
    }

    /// How long the asset plays, in ms.
    pub fn duration_ms(self) -> u32 {
        match self {
            Self::Step(sound) => sound.duration_ms(),
            Self::Finish(sound) => sound.duration_ms(),
        }
    }

    /// The length in seconds to one decimal: `1.0 s`.
    pub fn length_label(self) -> String {
        let tenths = self.duration_ms().div_ceil(100);
        format!("{}.{} s", tenths / 10, tenths % 10)
    }

    /// Parses any step or finish sound's id strictly: anything else is
    /// [`ModelError::UnknownSound`].
    pub fn try_parse(raw: &str) -> Result<Self, ModelError> {
        StepSound::try_parse(raw)
            .map(Self::Step)
            .or_else(|_| FinishSound::try_parse(raw).map(Self::Finish))
            .map_err(|_| ModelError::UnknownSound(raw.to_owned()))
    }
}

impl From<StepSound> for Sound {
    fn from(sound: StepSound) -> Self {
        Self::Step(sound)
    }
}

impl From<FinishSound> for Sound {
    fn from(sound: FinishSound) -> Self {
        Self::Finish(sound)
    }
}
