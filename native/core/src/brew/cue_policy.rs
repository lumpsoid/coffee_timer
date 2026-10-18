use std::num::NonZeroU32;

use crate::brew::{Effect, RunningStep};

/// The lighter cues a brew gives ahead of a step boundary, as named
/// config: how many seconds before a step ends the pre-alert buzzes, and
/// how long it buzzes. One value, held by
/// [`crate::brew::BrewController`], so a setting can later replace the
/// default without touching the timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CuePolicy {
    /// Seconds left in a step when the pre-alert fires; none turns it off.
    pre_alert_lead: Option<NonZeroU32>,
}

impl Default for CuePolicy {
    fn default() -> Self {
        Self::new(Self::DEFAULT_PRE_ALERT_LEAD_SECONDS)
    }
}

impl CuePolicy {
    /// Seconds before a step ends that the pre-alert fires by default.
    pub const DEFAULT_PRE_ALERT_LEAD_SECONDS: u32 = 3;

    /// The pre-alert's buzz, shorter than a step boundary's 100 ms so the
    /// two feel different.
    pub const PRE_ALERT_MS: u32 = 40;

    /// A pre-alert `lead_seconds` before each step ends; 0 is no pre-alert.
    pub fn new(lead_seconds: u32) -> Self {
        Self {
            pre_alert_lead: NonZeroU32::new(lead_seconds),
        }
    }

    /// Seconds before a step ends that the pre-alert fires; 0 when off.
    pub fn pre_alert_lead_seconds(self) -> u32 {
        self.pre_alert_lead.map_or(0, NonZeroU32::get)
    }

    /// The pre-alert for `step`, just ticked, if this is its moment: the
    /// tick that leaves exactly the lead time. That moment comes once per
    /// step (the countdown only falls, and pausing keeps it), and never at
    /// a step boundary (the lead is at least 1 s). A step no longer than
    /// the lead gets none: it would buzz at its start, or never.
    pub(crate) fn pre_alert(self, step: &RunningStep) -> Option<Effect> {
        let lead = self.pre_alert_lead?.get();
        (step.duration().seconds() > lead && step.remaining().seconds() == lead).then_some(
            Effect::Vibrate {
                ms: Self::PRE_ALERT_MS,
            },
        )
    }
}
