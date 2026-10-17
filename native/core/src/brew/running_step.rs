use crate::model::{RecipeStep, StepDuration};

/// The step a running brew is on, and how long it has left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningStep {
    index: usize,
    step: RecipeStep,
    remaining: StepDuration,
}

impl RunningStep {
    /// Starts `step` (at `index` in its recipe) with its full duration left.
    pub(crate) fn begin(index: usize, step: RecipeStep) -> Self {
        let remaining = step.duration();
        Self {
            index,
            step,
            remaining,
        }
    }

    /// One second less. Never goes below zero.
    pub(crate) fn ticked(self) -> Self {
        Self {
            remaining: self.remaining.saturating_sub(1),
            ..self
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn name(&self) -> &str {
        self.step.name()
    }

    pub fn remaining(&self) -> StepDuration {
        self.remaining
    }

    /// `MM:SS` countdown in the timer's centre.
    pub fn remaining_label(&self) -> String {
        self.remaining.mmss()
    }

    /// Period of the timer ring's repeating sweep: the step's full duration,
    /// or none for a 0-second step (the ring stays still).
    pub fn sweep(&self) -> Option<StepDuration> {
        let duration = self.step.duration();
        (!duration.is_zero()).then_some(duration)
    }
}
