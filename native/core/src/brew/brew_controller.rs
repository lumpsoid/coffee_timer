use crate::brew::{
    BrewError, BrewNotice, BrewState, BrewView, CuePolicy, Effect, RatioChange, RunningStep,
    SaveRatioError,
};
use crate::catalog::{CatalogChange, RecipeCatalog};
use crate::model::{BrewMetrics, BrewRatio, RecipeId};

/// Buzz length at each step boundary (legacy `vibrate_leaf.dart`:
/// `Vibration.vibrate(duration: 100)`).
const VIBRATE_MS: u32 = 100;

/// The end of a brew: two long buzzes, so it does not feel like one more
/// step boundary. Pause and buzz lengths in ms, starting with a pause.
const FINISHED_PATTERN_MS: [u32; 4] = [0, 400, 200, 400];

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
    cues: CuePolicy,
    /// Accepted starts so far; the running step carries the latest.
    runs: u64,
}

impl BrewController {
    /// Starts with no recipe open.
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts with no recipe open, cueing step boundaries by `cues`.
    pub fn with_cues(cues: CuePolicy) -> Self {
        Self {
            cues,
            ..Self::default()
        }
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
        let ratio = view.ratio().ratio();
        let adjusted = match change {
            RatioChange::MoreWater => Some(ratio.more_water()),
            RatioChange::LessWater => Some(ratio.less_water()),
            RatioChange::Recipe => None,
        };
        self.set_ratio(adjusted);
        Ok(())
    }

    /// Makes the adjusted ratio the open recipe's own: saves the recipe
    /// with the water shown and every step's pour as shown (scaled by
    /// [`crate::model::PourScale`]), coffee kept, through `catalog`. The
    /// page then shows the saved recipe at its own ratio, and a running,
    /// paused or finished brew keeps its place: the steps and their times
    /// are the same, only the water and pours change. The returned change is for the
    /// shell to pass on; [`BrewController::on_catalog_change`] finds the
    /// page already showing it. On `Err` nothing changes.
    pub fn save_ratio(
        &mut self,
        catalog: &mut RecipeCatalog,
    ) -> Result<CatalogChange, SaveRatioError> {
        let view = self.state.view().ok_or(SaveRatioError::NoRecipe)?;
        let ratio = view.ratio().adjusted().ok_or(SaveRatioError::NotAdjusted)?;
        let recipe = view.recipe().clone();
        let water = ratio.water_grams(recipe.coffee());
        let saved = recipe.with_water_scaled_to(water)?;
        let change = catalog.upsert(saved.clone())?;
        let metrics = self.metrics;
        self.state =
            std::mem::take(&mut self.state).map_view(|_| BrewView::of(saved, &metrics, None));
        Ok(change)
    }

    /// Runs the open recipe from step `from` (0 for the whole recipe),
    /// restarting a brew already running, paused or finished, as a new run
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

    /// Holds the running brew on its step with the time it has left. The
    /// screen may sleep again (nothing on it moves until resumed) and the
    /// notification stays, reading `Paused`, so the brew survives in the
    /// background and is one tap away. On `Err` nothing changes.
    pub fn pause(&mut self) -> Result<Vec<Effect>, BrewError> {
        let (view, step) = match std::mem::take(&mut self.state) {
            BrewState::Running(view, step) => (view, step),
            other => {
                self.state = other;
                return Err(BrewError::NotRunning);
            }
        };
        let notice = Effect::ShowBrewNotification(BrewNotice::paused(&view, &step));
        self.state = BrewState::Paused(view, step);
        Ok(vec![Effect::KeepScreenOn(false), notice])
    }

    /// Counts a paused brew down again from where it was held, in the same
    /// run. No vibration or sound. On `Err` nothing changes.
    pub fn resume(&mut self) -> Result<Vec<Effect>, BrewError> {
        let (view, step) = match std::mem::take(&mut self.state) {
            BrewState::Paused(view, step) => (view, step),
            other => {
                self.state = other;
                return Err(BrewError::NotPaused);
            }
        };
        Ok(vec![
            Effect::KeepScreenOn(true),
            self.keep_running(view, step),
        ])
    }

    /// One second passes and the brew notification follows. A few seconds
    /// before a step ends it buzzes briefly, once ([`CuePolicy`]). At zero
    /// the brew moves to the next step (vibrate, sound) or, after the last
    /// step, finishes (screen-on off, the finish buzz pattern, sound,
    /// notification gone). A no-op unless running (paused and finished
    /// included).
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
            let pre_alert = self.cues.pre_alert(&step);
            return pre_alert
                .into_iter()
                .chain([self.keep_running(view, step)])
                .collect();
        }
        match view.next_step(&step) {
            Some(next) => vec![
                Effect::Vibrate { ms: VIBRATE_MS },
                Effect::PlayStepSound,
                self.keep_running(view, next),
            ],
            None => {
                let finished = view.finished(step.first(), &self.metrics);
                self.state = BrewState::Finished(view, finished);
                // The notification goes last: hiding it lets the platform
                // stop keeping the app awake, after the buzz and sound.
                vec![
                    Effect::KeepScreenOn(false),
                    Effect::VibratePattern {
                        pattern_ms: FINISHED_PATTERN_MS.to_vec(),
                    },
                    Effect::PlayStepSound,
                    Effect::HideBrewNotification,
                ]
            }
        }
    }

    /// Stops a running or paused brew, or clears a finished one; the recipe
    /// stays open, idle.
    pub fn reset(&mut self) -> Vec<Effect> {
        let effects = self.stop_silently();
        self.state = match std::mem::take(&mut self.state) {
            BrewState::Running(view, _)
            | BrewState::Paused(view, _)
            | BrewState::Finished(view, _) => BrewState::Idle(view),
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
    /// running brew stops, a finished one clears), keeping an adjusted ratio; a delete shows "Empty
    /// recipe". Other recipes' changes do nothing, and so does one that
    /// leaves the open recipe as the page already shows it (a saved ratio,
    /// or a save that changed nothing).
    pub fn on_catalog_change(
        &mut self,
        change: &CatalogChange,
        catalog: &RecipeCatalog,
    ) -> Vec<Effect> {
        let (CatalogChange::Upserted(id) | CatalogChange::Removed(id)) = change;
        let unchanged = self
            .state
            .view()
            .is_some_and(|view| catalog.get(id) == Some(view.recipe()));
        if !self.state.shows(id) || unchanged {
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

    /// What stopping the current brew takes, without vibration or sound. A
    /// paused brew already let the screen sleep; a finished one let go of
    /// both.
    fn stop_silently(&self) -> Vec<Effect> {
        match self.state {
            BrewState::Running(..) => {
                vec![Effect::KeepScreenOn(false), Effect::HideBrewNotification]
            }
            BrewState::Paused(..) => vec![Effect::HideBrewNotification],
            BrewState::Empty | BrewState::Idle(_) | BrewState::Finished(..) => Vec::new(),
        }
    }
}
