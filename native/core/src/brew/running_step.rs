use crate::model::{RecipeStep, StepDuration};

/// The step a running or paused brew is on, and how long it has left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningStep {
    run: u64,
    /// The step this run started from.
    first: usize,
    index: usize,
    step: RecipeStep,
    remaining: StepDuration,
}

impl RunningStep {
    /// Starts `step` (at `index` in its recipe) with its full duration left,
    /// as part of start number `run`, which began at step `first`.
    pub(crate) fn begin(run: u64, first: usize, index: usize, step: RecipeStep) -> Self {
        let remaining = step.duration();
        Self {
            run,
            first,
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

    /// Which start of the brew this step belongs to: every accepted
    /// `BrewController::start` counts one more, ticks and step changes keep
    /// it. A restart of the same step therefore differs from a tick.
    pub fn run(&self) -> u64 {
        self.run
    }

    /// The step this run started from: 0 for the whole recipe, more after
    /// a long-press on a later step. Step changes keep it.
    pub fn first(&self) -> usize {
        self.first
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
