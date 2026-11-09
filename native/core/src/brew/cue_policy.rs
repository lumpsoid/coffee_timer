use crate::brew::{Effect, RunningStep};
use crate::model::{FinishSound, PreAlertLead, Settings, Sound, StepSound, Toggle};

/// The cues a brew gives, as named config: the buzz and the chosen sound
/// at a step boundary and at the finish, and the pre-alert's two quick taps a few
/// seconds before a step ends. Built from the user's [`Settings`] on every
/// tick ([`crate::brew::BrewController::tick`]), so muting is decided
/// here: with sound off no cue plays the sound, with vibration off none
/// buzzes, and the shell carries out whatever is left.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CuePolicy {
    pre_alert: PreAlertLead,
    sound: Toggle,
    vibration: Toggle,
    step_sound: StepSound,
    finish_sound: FinishSound,
}

impl CuePolicy {
    /// Buzz length at each step boundary (legacy `vibrate_leaf.dart`:
    /// `Vibration.vibrate(duration: 100)`).
    pub const STEP_BUZZ_MS: u32 = 100;

    /// The pre-alert: two quick taps, so it feels unlike a step boundary's
    /// single buzz and the finish's two long ones. Pause and buzz lengths
    /// in ms, starting with a pause.
    pub const PRE_ALERT_PATTERN_MS: [u32; 4] = [0, 50, 100, 50];

    /// The end of a brew: two long buzzes, so it does not feel like one more
    /// step boundary. Pause and buzz lengths in ms, starting with a pause.
    pub const FINISH_PATTERN_MS: [u32; 4] = [0, 400, 200, 400];

    /// The cues `settings` allow.
    pub fn of(settings: &Settings) -> Self {
        Self {
            pre_alert: settings.pre_alert(),
            sound: settings.sound(),
            vibration: settings.vibration(),
            step_sound: settings.step_sound(),
            finish_sound: settings.finish_sound(),
        }
    }

    /// The pre-alert for `step`, just ticked, if this is its moment: the
    /// tick that leaves exactly the lead time. That moment comes once per
    /// step (the countdown only falls, and pausing keeps it), and never at
    /// a step boundary (the lead is at least 1 s). A step no longer than
    /// the lead gets none: it would buzz at its start, or never. It is a
    /// buzz, so vibration off silences it.
    pub(crate) fn pre_alert(self, step: &RunningStep) -> Option<Effect> {
        let lead = self.pre_alert.seconds();
        let due =
            lead > 0 && step.duration().seconds() > lead && step.remaining().seconds() == lead;
        (due && self.vibration.is_on()).then(|| Effect::VibratePattern {
            pattern_ms: Self::PRE_ALERT_PATTERN_MS.to_vec(),
        })
    }

    /// A step boundary's cues: a buzz, then the step sound.
    pub(crate) fn step_boundary(self) -> Vec<Effect> {
        self.buzz_then_sound(
            Effect::Vibrate {
                ms: Self::STEP_BUZZ_MS,
            },
            self.step_sound.into(),
        )
    }

    /// The finish's cues: the two long buzzes, then the finish sound.
    pub(crate) fn finish(self) -> Vec<Effect> {
        self.buzz_then_sound(
            Effect::VibratePattern {
                pattern_ms: Self::FINISH_PATTERN_MS.to_vec(),
            },
            self.finish_sound.into(),
        )
    }

    fn buzz_then_sound(self, buzz: Effect, sound: Sound) -> Vec<Effect> {
        let buzz = self.vibration.is_on().then_some(buzz);
        let sound = self.sound.is_on().then_some(Effect::PlaySound { sound });
        buzz.into_iter().chain(sound).collect()
    }
}
