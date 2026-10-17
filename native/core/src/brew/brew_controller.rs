use crate::brew::{BrewError, BrewState, BrewView, Effect};
use crate::catalog::{CatalogChange, RecipeCatalog};
use crate::model::{BrewMetrics, RecipeId};

/// Buzz length at each step boundary and at the end (legacy
/// `vibrate_leaf.dart`: `Vibration.vibrate(duration: 100)`).
const VIBRATE_MS: u32 = 100;

/// Owner of the brew page: which recipe is open and where its timer is.
///
/// Holds the timer as a state machine and never reads the clock: time
/// arrives as [`BrewController::tick`], one call per second while
/// [`BrewState::is_running`]. Keeps a copy of the open recipe, refreshed by
/// [`BrewController::on_catalog_change`], so its state is the whole view.
#[derive(Debug, Default)]
pub struct BrewController {
    state: BrewState,
    metrics: BrewMetrics,
}

impl BrewController {
    /// Starts with no recipe open.
    pub fn new() -> Self {
        Self::default()
    }

    /// Shows the recipe with `id`, idle. An id the catalog does not hold
    /// shows "Empty recipe". A running brew stops silently.
    pub fn open(&mut self, id: &RecipeId, catalog: &RecipeCatalog) -> Vec<Effect> {
        let effects = self.stop_silently();
        self.state = match catalog.get(id) {
            Some(recipe) => BrewState::Idle(BrewView::of(recipe.clone(), &self.metrics)),
            None => BrewState::Empty,
        };
        effects
    }

    /// Runs the open recipe from step `from` (0 for the whole recipe),
    /// restarting a brew already running. On `Err` nothing changes.
    pub fn start(&mut self, from: usize) -> Result<Vec<Effect>, BrewError> {
        let view = self.state.view().ok_or(BrewError::NoRecipe)?;
        let step = view.step(from).ok_or(BrewError::StepOutOfRange {
            index: from,
            len: view.step_count(),
        })?;
        self.state = BrewState::Running(view.clone(), step);
        Ok(vec![Effect::KeepScreenOn(true)])
    }

    /// One second passes. At zero the brew moves to the next step (vibrate,
    /// sound) or, after the last step, ends (screen-on off, vibrate, no
    /// sound). A no-op unless running.
    pub fn tick(&mut self) -> Vec<Effect> {
        let (view, step) = match std::mem::take(&mut self.state) {
            BrewState::Running(view, step) => (view, step),
            other => {
                self.state = other;
                return Vec::new();
            }
        };
        let step = step.ticked();
        if !step.remaining().is_zero() {
            self.state = BrewState::Running(view, step);
            return Vec::new();
        }
        match view.step(step.index() + 1) {
            Some(next) => {
                self.state = BrewState::Running(view, next);
                vec![Effect::Vibrate { ms: VIBRATE_MS }, Effect::PlayStepSound]
            }
            None => {
                self.state = BrewState::Idle(view);
                vec![
                    Effect::KeepScreenOn(false),
                    Effect::Vibrate { ms: VIBRATE_MS },
                ]
            }
        }
    }

    /// Stops a running brew; the recipe stays open, idle.
    pub fn reset(&mut self) -> Vec<Effect> {
        let effects = self.stop_silently();
        self.state = match std::mem::take(&mut self.state) {
            BrewState::Running(view, _) => BrewState::Idle(view),
            other => other,
        };
        effects
    }

    /// Closes the page. A running brew stops with no vibration or sound.
    pub fn leave(&mut self) -> Vec<Effect> {
        let effects = self.stop_silently();
        self.state = BrewState::Empty;
        effects
    }

    /// Follows a persisted catalog change. A change to the open recipe
    /// reopens it from `catalog`: an edit shows the new version, idle (a
    /// running brew stops); a delete shows "Empty recipe". Other recipes'
    /// changes do nothing.
    pub fn on_catalog_change(
        &mut self,
        change: &CatalogChange,
        catalog: &RecipeCatalog,
    ) -> Vec<Effect> {
        let (CatalogChange::Upserted(id) | CatalogChange::Removed(id)) = change;
        if !self.state.shows(id) {
            return Vec::new();
        }
        self.open(id, catalog)
    }

    pub fn state(&self) -> &BrewState {
        &self.state
    }

    /// What stopping the current brew takes, without vibration or sound.
    fn stop_silently(&self) -> Vec<Effect> {
        if self.state.is_running() {
            vec![Effect::KeepScreenOn(false)]
        } else {
            Vec::new()
        }
    }
}
