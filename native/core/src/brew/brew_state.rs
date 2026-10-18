use crate::brew::{BrewView, FinishedBrew, RunningStep, StepRow, StepStatus};
use crate::model::RecipeId;

/// The whole brew page. A brew that ran past its last step is `Finished`,
/// not `Idle`: the page tells an ended brew from one never started (the
/// legacy page did not; see docs/port-plan.md).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum BrewState {
    /// No recipe to show: none opened, left, unknown id, or deleted while
    /// open. The page reads [`BrewState::EMPTY_LABEL`].
    #[default]
    Empty,
    /// A recipe is open and its timer is stopped; the timer shows the
    /// recipe's metrics.
    Idle(BrewView),
    /// The timer is counting down a step. The shell ticks the controller once
    /// a second exactly while the state is `Running`.
    Running(BrewView, RunningStep),
    /// The timer is held on a step, keeping the time it had left; resuming
    /// counts down from there. The shell does not tick while paused.
    Paused(BrewView, RunningStep),
    /// The last step ran out: every step is done and the timer shows how
    /// long the run brewed. Stopped like `Idle`; a start, reset or reopen
    /// leaves it.
    Finished(BrewView, FinishedBrew),
}

impl BrewState {
    pub const EMPTY_LABEL: &'static str = "Empty recipe";

    pub fn view(&self) -> Option<&BrewView> {
        match self {
            Self::Empty => None,
            Self::Idle(view)
            | Self::Running(view, _)
            | Self::Paused(view, _)
            | Self::Finished(view, _) => Some(view),
        }
    }

    /// The step being counted down; none while paused.
    pub fn running(&self) -> Option<&RunningStep> {
        match self {
            Self::Running(_, step) => Some(step),
            Self::Empty | Self::Idle(_) | Self::Paused(..) | Self::Finished(..) => None,
        }
    }

    /// The step the timer is held on; none unless paused.
    pub fn paused(&self) -> Option<&RunningStep> {
        match self {
            Self::Paused(_, step) => Some(step),
            Self::Empty | Self::Idle(_) | Self::Running(..) | Self::Finished(..) => None,
        }
    }

    /// How the last run ended; none unless finished.
    pub fn finished(&self) -> Option<&FinishedBrew> {
        match self {
            Self::Finished(_, finished) => Some(finished),
            Self::Empty | Self::Idle(_) | Self::Running(..) | Self::Paused(..) => None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running().is_some()
    }

    /// The step list with each step's status; empty when no recipe is open.
    /// The step a paused brew is held on is `Paused`, not `Current`. Every
    /// step of a finished brew is `Done`.
    pub fn step_rows(&self) -> Vec<StepRow> {
        match self {
            Self::Empty => Vec::new(),
            Self::Idle(view) => view.step_rows(None),
            Self::Running(view, step) => view.step_rows(Some((step.index(), StepStatus::Current))),
            Self::Paused(view, step) => view.step_rows(Some((step.index(), StepStatus::Paused))),
            // Past the last step: every step is before it.
            Self::Finished(view, _) => {
                view.step_rows(Some((view.step_count(), StepStatus::Pending)))
            }
        }
    }

    /// The same state with `f` applied to its view; `Empty` stays empty.
    pub(crate) fn map_view(self, f: impl FnOnce(BrewView) -> BrewView) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Idle(view) => Self::Idle(f(view)),
            Self::Running(view, step) => Self::Running(f(view), step),
            Self::Paused(view, step) => Self::Paused(f(view), step),
            Self::Finished(view, finished) => Self::Finished(f(view), finished),
        }
    }

    /// Whether the page shows the recipe with `id`.
    pub(crate) fn shows(&self, id: &RecipeId) -> bool {
        self.view().is_some_and(|view| view.recipe_id() == id)
    }
}
