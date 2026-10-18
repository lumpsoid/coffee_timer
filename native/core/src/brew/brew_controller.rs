use crate::brew::{BrewError, BrewNotice, BrewState, BrewView, Effect, RatioChange, RunningStep};
use crate::catalog::{CatalogChange, RecipeCatalog};
use crate::model::{BrewMetrics, BrewRatio, RecipeId};

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
    /// Accepted starts so far; the running step carries the latest.
    runs: u64,
}

impl BrewController {
    /// Starts with no recipe open.
    pub fn new() -> Self {
        Self::default()
    }

    /// Shows the recipe with `id`, idle, at its own ratio. An id the catalog
    /// does not hold shows "Empty recipe". A running brew stops silently.
    pub fn open(&mut self, id: &RecipeId, catalog: &RecipeCatalog) -> Vec<Effect> {
        let effects = self.stop_silently();
        self.state = match catalog.get(id) {
            Some(recipe) => BrewState::Idle(BrewView::of(recipe.clone(), &self.metrics, None)),
            None => BrewState::Empty,
        };
        effects
    }

    /// Changes the open recipe's ratio for this brew only: the coffee stays
    /// and the water follows. Opening a recipe or leaving the page goes back
    /// to the recipe's own ratio. A running brew keeps running. On `Err`
    /// nothing changes.
    pub fn adjust_ratio(&mut self, change: RatioChange) -> Result<(), BrewError> {
        let view = self.state.view().ok_or(BrewError::NoRecipe)?;
        let ratio = view.ratio().ratio().ok_or(BrewError::NoCoffee)?;
        let adjusted = match change {
            RatioChange::MoreWater => Some(ratio.more_water()),
            RatioChange::LessWater => Some(ratio.less_water()),
            RatioChange::Recipe => None,
        };
        self.set_ratio(adjusted);
        Ok(())
    }

    /// Runs the open recipe from step `from` (0 for the whole recipe),
    /// restarting a brew already running, as a new run
    /// ([`RunningStep::run`]). Shows the brew notification. On `Err` nothing
    /// changes.
    pub fn start(&mut self, from: usize) -> Result<Vec<Effect>, BrewError> {
        let view = self.state.view().ok_or(BrewError::NoRecipe)?;
        let run = self.runs.saturating_add(1);
        let step = view.step(run, from).ok_or(BrewError::StepOutOfRange {
            index: from,
            len: view.step_count(),
        })?;
        let notice = Effect::ShowBrewNotification(BrewNotice::of(view, &step));
        self.state = BrewState::Running(view.clone(), step);
        self.runs = run;
        Ok(vec![Effect::KeepScreenOn(true), notice])
    }

    /// One second passes and the brew notification follows. At zero the brew
    /// moves to the next step (vibrate, sound) or, after the last step, ends
    /// (screen-on off, vibrate, no sound, notification gone). A no-op unless
    /// running.
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
            return vec![self.keep_running(view, step)];
        }
        match view.step(step.run(), step.index() + 1) {
            Some(next) => vec![
                Effect::Vibrate { ms: VIBRATE_MS },
                Effect::PlayStepSound,
                self.keep_running(view, next),
            ],
            None => {
                self.state = BrewState::Idle(view);
                // The notification goes last: hiding it lets the platform
                // stop keeping the app awake.
                vec![
                    Effect::KeepScreenOn(false),
                    Effect::Vibrate { ms: VIBRATE_MS },
                    Effect::HideBrewNotification,
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
    /// running brew stops), keeping an adjusted ratio; a delete shows "Empty
    /// recipe". Other recipes' changes do nothing.
    pub fn on_catalog_change(
        &mut self,
        change: &CatalogChange,
        catalog: &RecipeCatalog,
    ) -> Vec<Effect> {
        let (CatalogChange::Upserted(id) | CatalogChange::Removed(id)) = change;
        if !self.state.shows(id) {
            return Vec::new();
        }
        let adjusted = self.state.view().and_then(|view| view.ratio().adjusted());
        let effects = self.open(id, catalog);
        self.set_ratio(adjusted);
        effects
    }

    pub fn state(&self) -> &BrewState {
        &self.state
    }

    /// Shows the open recipe at the ratio `adjusted`, or at its own.
    fn set_ratio(&mut self, adjusted: Option<BrewRatio>) {
        let metrics = self.metrics;
        self.state =
            std::mem::take(&mut self.state).map_view(|view| view.with_ratio(adjusted, &metrics));
    }

    /// Runs `step` of `view` and returns the notification showing it.
    fn keep_running(&mut self, view: BrewView, step: RunningStep) -> Effect {
        let notice = Effect::ShowBrewNotification(BrewNotice::of(&view, &step));
        self.state = BrewState::Running(view, step);
        notice
    }

    /// What stopping the current brew takes, without vibration or sound.
    fn stop_silently(&self) -> Vec<Effect> {
        if self.state.is_running() {
            vec![Effect::KeepScreenOn(false), Effect::HideBrewNotification]
        } else {
            Vec::new()
        }
    }
}
