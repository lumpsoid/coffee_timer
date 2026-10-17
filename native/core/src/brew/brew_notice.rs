use crate::brew::{BrewView, RunningStep};

/// What the ongoing brew notification shows: the running step with its
/// countdown, `Bloom · 00:35`, and the step after it, `Next: Pour`, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrewNotice {
    pub title: String,
    pub next: Option<String>,
}

impl BrewNotice {
    pub(crate) fn of(view: &BrewView, step: &RunningStep) -> Self {
        Self {
            title: format!("{} · {}", step.name(), step.remaining_label()),
            next: view
                .step_name(step.index() + 1)
                .map(|name| format!("Next: {name}")),
        }
    }
}
