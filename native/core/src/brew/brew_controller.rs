use crate::brew::{
    BrewError, BrewNotice, BrewState, BrewView, CuePolicy, Effect, RatioChange, RunningStep,
    SaveRatioError,
};
use crate::catalog::{CatalogChange, RecipeCatalog};
use crate::model::{BrewMetrics, BrewRatio, CoffeeDose, RecipeId, Settings};

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

    /// Shows the recipe with `id`, idle, at its own dose. An id the catalog
    /// does not hold shows "Empty recipe". A running brew stops silently.
    pub fn open(&mut self, id: &RecipeId, catalog: &RecipeCatalog) -> Vec<Effect> {
        let effects = self.stop_silently();
        self.state = match catalog.get(id) {
            Some(recipe) => {
                BrewState::Idle(BrewView::of(recipe.clone(), &self.metrics, None, None))
            }
            None => BrewState::Empty,
        };
        effects
    }

    /// Changes the open recipe's ratio for this brew only: the coffee shown
    /// stays and the water follows. Opening a recipe or leaving the page
    /// goes back to the recipe's own dose. A running brew keeps running. On
    /// `Err` nothing changes.
    pub fn adjust_ratio(&mut self, change: RatioChange) -> Result<(), BrewError> {
        let view = self.state.view().ok_or(BrewError::NoRecipe)?;
        let ratio = view.ratio().ratio();
        let adjusted = match change {
            RatioChange::MoreWater => ratio.more_water(),
            RatioChange::LessWater => ratio.less_water(),
        };
        self.set_dose(view.coffee().adjusted(), Some(adjusted));
        Ok(())
    }

    /// Brews the open recipe with `coffee` for this brew only: the ratio
    /// shown stays and the water follows (at the recipe's own ratio, as the
    /// recipe scales; see [`crate::model::Recipe::water_for`]). Opening a
    /// recipe or leaving the page goes back to the recipe's own dose. A
    /// running brew keeps running. On `Err` nothing changes.
    pub fn set_coffee(&mut self, coffee: CoffeeDose) -> Result<(), BrewError> {
        let view = self.state.view().ok_or(BrewError::NoRecipe)?;
        self.set_dose(Some(coffee), view.ratio().adjusted());
        Ok(())
    }

    /// Back to the open recipe's own coffee and ratio. A running brew keeps
    /// running. On `Err` nothing changes.
    pub fn back_to_recipe_dose(&mut self) -> Result<(), BrewError> {
        self.state.view().ok_or(BrewError::NoRecipe)?;
        self.set_dose(None, None);
        Ok(())
    }

    /// Makes the brew's changed dose the open recipe's own: saves the
    /// recipe with the coffee and water shown and every step's pour as
    /// shown (scaled by [`crate::model::PourScale`]), through `catalog`.
    /// The page then shows the saved recipe at its own dose, and a running,
    /// paused or finished brew keeps its place: the steps and their times
    /// are the same, only the coffee, water and pours change. The returned
    /// change is for the shell to pass on;
    /// [`BrewController::on_catalog_change`] finds the page already showing
    /// it. On `Err` nothing changes.
    pub fn save_ratio(
        &mut self,
        catalog: &mut RecipeCatalog,
    ) -> Result<CatalogChange, SaveRatioError> {
        let view = self.state.view().ok_or(SaveRatioError::NoRecipe)?;
        if !view.is_adjusted() {
            return Err(SaveRatioError::NotAdjusted);
        }
        let saved = view
            .recipe()
            .clone()
            .with_dose_scaled_to(view.coffee().dose(), view.water_grams())?;
        let change = catalog.upsert(saved.clone())?;
        let metrics = self.metrics;
        self.state =
            std::mem::take(&mut self.state).map_view(|_| BrewView::of(saved, &metrics, None, None));
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
    /// before a step ends it buzzes briefly, once. At zero the brew moves
    /// to the next step (vibrate, sound) or, after the last step, finishes
    /// (screen-on off, the finish buzz pattern, sound, notification gone).
    /// The buzzes and the sound are the cues `settings` allow
    /// ([`CuePolicy`]), read on every tick, so a change mid-brew applies
    /// from the next one. A no-op unless running (paused and finished
    /// included).
    pub fn tick(&mut self, settings: &Settings) -> Vec<Effect> {
        let (view, step) = match std::mem::take(&mut self.state) {
            BrewState::Running(view, step) => (view, step),
            other => {
                self.state = other;
                return Vec::new();
            }
        };
        let cues = CuePolicy::of(settings);
        let step = step.ticked();
        if !step.remaining().is_zero() {
            let pre_alert = cues.pre_alert(&step);
            return pre_alert
                .into_iter()
                .chain([self.keep_running(view, step)])
                .collect();
        }
        match view.next_step(&step) {
            Some(next) => {
                let mut effects = cues.step_boundary();
                effects.push(self.keep_running(view, next));
                effects
            }
            None => {
                let finished = view.finished(step.first(), &self.metrics);
                self.state = BrewState::Finished(view, finished);
                // The notification goes last: hiding it lets the platform
                // stop keeping the app awake, after the buzz and sound.
                [Effect::KeepScreenOn(false)]
                    .into_iter()
                    .chain(cues.finish())
                    .chain([Effect::HideBrewNotification])
                    .collect()
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
    /// running brew stops, a finished one clears), keeping a changed coffee
    /// and ratio; a delete shows "Empty
    /// recipe". Other recipes' changes do nothing, and so does one that
    /// leaves the open recipe as the page already shows it (a saved dose,
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
        let (coffee, ratio) = self.state.view().map_or((None, None), |view| {
            (view.coffee().adjusted(), view.ratio().adjusted())
        });
        let effects = self.open(id, catalog);
        self.set_dose(coffee, ratio);
        effects
    }

    pub fn state(&self) -> &BrewState {
        &self.state
    }

    /// Shows the open recipe at `coffee` and `ratio`, each at the recipe's
    /// own when none.
    fn set_dose(&mut self, coffee: Option<CoffeeDose>, ratio: Option<BrewRatio>) {
        let metrics = self.metrics;
        self.state = std::mem::take(&mut self.state)
            .map_view(|view| view.with_dose(coffee, ratio, &metrics));
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
