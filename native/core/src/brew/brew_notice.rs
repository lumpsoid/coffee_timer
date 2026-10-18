use crate::brew::{BrewView, RunningStep};

/// What the ongoing brew notification shows: the running step with its
/// countdown, `Bloom · 00:35` (`Bloom · 00:35 · Paused` while paused), and
/// the step after it, `Next: Pour`, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrewNotice {
    pub title: String,
    pub next: Option<String>,
}

impl BrewNotice {
    /// The notice of `step` counting down.
    pub(crate) fn of(view: &BrewView, step: &RunningStep) -> Self {
        Self::titled(
            view,
            step,
            format!("{} · {}", step.name(), step.remaining_label()),
        )
    }

    /// The notice of a brew paused on `step`.
    pub(crate) fn paused(view: &BrewView, step: &RunningStep) -> Self {
        Self::titled(
            view,
            step,
            format!("{} · {} · Paused", step.name(), step.remaining_label()),
        )
    }

    fn titled(view: &BrewView, step: &RunningStep, title: String) -> Self {
        Self {
            title,
            next: view
                .step_name(step.index() + 1)
                .map(|name| format!("Next: {name}")),
        }
    }
}
