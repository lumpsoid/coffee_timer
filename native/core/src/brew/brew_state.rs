use crate::brew::{BrewView, RunningStep, StepRow};
use crate::model::RecipeId;

/// The whole brew page. A finished brew is `Idle` again: the legacy page
/// showed nothing that told an ended brew from one never started.
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
}

impl BrewState {
    pub const EMPTY_LABEL: &'static str = "Empty recipe";

    pub fn view(&self) -> Option<&BrewView> {
        match self {
            Self::Empty => None,
            Self::Idle(view) | Self::Running(view, _) => Some(view),
        }
    }

    pub fn running(&self) -> Option<&RunningStep> {
        match self {
            Self::Running(_, step) => Some(step),
            Self::Empty | Self::Idle(_) => None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.running().is_some()
    }

    /// The step list with each step's status; empty when no recipe is open.
    pub fn step_rows(&self) -> Vec<StepRow> {
        match self.view() {
            Some(view) => view.step_rows(self.running().map(RunningStep::index)),
            None => Vec::new(),
        }
    }

    /// Whether the page shows the recipe with `id`.
    pub(crate) fn shows(&self, id: &RecipeId) -> bool {
        self.view().is_some_and(|view| view.recipe_id() == id)
    }
}
