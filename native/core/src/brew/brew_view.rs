use std::cmp::Ordering;

use crate::brew::{FinishedBrew, RatioSetting, RunningStep, StepRow, StepStatus};
use crate::model::{
    BrewMetrics, BrewRatio, GrindClicks, LabelStyle, Pour, PourScale, Recipe, RecipeId,
};

/// The open recipe as the brew page shows it. Built once per recipe version
/// and ratio; the timer's position is kept beside it in
/// [`crate::brew::BrewState`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrewView {
    recipe: Recipe,
    coffee_label: String,
    water_label: String,
    ratio_label: String,
    ratio: RatioSetting,
    duration_label: String,
    style: LabelStyle,
    /// Each step's pour at the water shown, in step order.
    pours: Vec<Option<Pour>>,
    /// What follows each step, in step order (see [`BrewView::next_label`]).
    next_labels: Vec<String>,
}

impl BrewView {
    /// [`BrewView::next_label`] on the last step: nothing follows.
    pub const LAST_STEP_LABEL: &'static str = "Last step";

    /// `recipe` at the ratio `adjusted` on the brew page, or at its own
    /// ratio. An adjusted ratio equal to the recipe's is no adjustment.
    /// The step pours follow the water ([`PourScale`]).
    pub(crate) fn of(recipe: Recipe, metrics: &BrewMetrics, adjusted: Option<BrewRatio>) -> Self {
        let coffee = recipe.coffee();
        let own = metrics.ratio(&recipe);
        let ratio = match adjusted {
            Some(ratio) if ratio != own => RatioSetting::Adjusted(ratio),
            Some(_) | None => RatioSetting::Recipe(own),
        };
        let water = match ratio {
            RatioSetting::Adjusted(ratio) => ratio.water_grams(coffee),
            RatioSetting::Recipe(_) => recipe.water().grams(),
        };
        let pours = PourScale::new(recipe.water(), water).pours(recipe.steps());
        let next_labels = next_labels(&recipe, &pours, metrics);
        Self {
            coffee_label: format!("{}g", coffee.grams()),
            water_label: format!("{water}g"),
            ratio_label: ratio.ratio().label(),
            ratio,
            duration_label: metrics.duration_label(&recipe),
            style: LabelStyle::of(&recipe, metrics),
            pours,
            next_labels,
            recipe,
        }
    }

    /// The recipe shown, as the catalog held it when the page opened it.
    pub(crate) fn recipe(&self) -> &Recipe {
        &self.recipe
    }

    /// The same recipe at the ratio `adjusted`, or at its own.
    pub(crate) fn with_ratio(self, adjusted: Option<BrewRatio>, metrics: &BrewMetrics) -> Self {
        Self::of(self.recipe, metrics, adjusted)
    }

    /// The step at `index`, about to run with its full duration in start
    /// number `run`; none past the end.
    pub(crate) fn step(&self, run: u64, index: usize) -> Option<RunningStep> {
        let step = self.recipe.steps().get(index)?;
        Some(RunningStep::begin(run, index, index, step.clone()))
    }

    /// The step after `step`, in the same run; none after the last.
    pub(crate) fn next_step(&self, step: &RunningStep) -> Option<RunningStep> {
        let index = step.index() + 1;
        let next = self.recipe.steps().get(index)?;
        Some(RunningStep::begin(
            step.run(),
            step.first(),
            index,
            next.clone(),
        ))
    }

    /// How a run that started at step `first` ended: the steps it brewed,
    /// timed by `metrics`.
    pub(crate) fn finished(&self, first: usize, metrics: &BrewMetrics) -> FinishedBrew {
        let brewed = self.recipe.steps().iter().skip(first);
        FinishedBrew::new(metrics.duration_label_of(brewed.map(|step| step.duration())))
    }

    /// The name of the step at `index`; none past the end.
    pub(crate) fn step_name(&self, index: usize) -> Option<&str> {
        self.recipe.steps().get(index).map(|step| step.name())
    }

    pub(crate) fn step_count(&self) -> usize {
        self.recipe.steps().len()
    }

    /// Every step with its status: before the `current` index done, at it
    /// the given status (`Current`, or `Paused`), after it pending. With no
    /// current step, all pending; with one past the last step, all done.
    ///
    /// Each step also gets its share of the whole brew, laid end to end from
    /// 0 to 1, for the dial's step track. A brew of 0-second steps shares it
    /// equally.
    pub(crate) fn step_rows(&self, current: Option<(usize, StepStatus)>) -> Vec<StepRow> {
        let steps = self.recipe.steps();
        let seconds: Vec<u32> = steps.iter().map(|s| s.duration().seconds()).collect();
        let weights = if seconds.iter().all(|&s| s == 0) {
            vec![1; seconds.len()]
        } else {
            seconds
        };
        let whole = f64::from(weights.iter().fold(0_u32, |sum, &w| sum.saturating_add(w)));
        let mut elapsed: u32 = 0;
        steps
            .iter()
            .zip(weights.iter())
            .zip(self.pours.iter())
            .enumerate()
            .map(|(index, ((step, &weight), pour))| {
                let start = elapsed;
                elapsed = elapsed.saturating_add(weight);
                StepRow {
                    number_label: (index + 1).to_string(),
                    name: step.name().to_owned(),
                    duration_label: step.duration().seconds_label(),
                    pour_label: pour.map(Pour::label),
                    status: match current.map(|(c, status)| (index.cmp(&c), status)) {
                        Some((Ordering::Less, _)) => StepStatus::Done,
                        Some((Ordering::Equal, status)) => status,
                        Some((Ordering::Greater, _)) | None => StepStatus::Pending,
                    },
                    track_start: f64::from(start) / whole,
                    track_end: f64::from(elapsed) / whole,
                }
            })
            .collect()
    }

    /// What the scale should read once step `index` has poured, `Pour to
    /// 150g`: its pour and every earlier one, at the water shown. None for
    /// a step that pours nothing, or past the end.
    pub fn pour_to_label(&self, index: usize) -> Option<String> {
        self.pours.get(index).copied().flatten().map(Pour::to_label)
    }

    /// What the scale reads once every step has poured, `Poured to 300g`,
    /// at the water shown; none for a recipe whose steps pour nothing.
    pub fn poured_label(&self) -> Option<String> {
        self.pours
            .iter()
            .rev()
            .find_map(|pour| *pour)
            .map(Pour::poured_to_label)
    }

    /// What comes after step `index`, shown under the dial while it runs
    /// or is paused: `Next: Pour 2 · 0:45 · 100g` (the pour only if that
    /// step pours, at the water shown), or [`BrewView::LAST_STEP_LABEL`] on
    /// the last step (and past it).
    pub fn next_label(&self, index: usize) -> String {
        self.next_labels
            .get(index)
            .cloned()
            .unwrap_or_else(|| Self::LAST_STEP_LABEL.to_owned())
    }

    /// Where step `index` sits in the brew, `Step 2 of 3`.
    pub fn position_label(&self, index: usize) -> String {
        format!("Step {} of {}", index + 1, self.step_count())
    }

    pub fn recipe_id(&self) -> &RecipeId {
        self.recipe.id()
    }

    pub fn name(&self) -> &str {
        self.recipe.name().as_str()
    }

    pub fn description(&self) -> &str {
        self.recipe.description()
    }

    /// Coffee grams, `20g`.
    pub fn coffee_label(&self) -> &str {
        &self.coffee_label
    }

    /// Water grams at the ratio shown, `100g`.
    pub fn water_label(&self) -> &str {
        &self.water_label
    }

    /// Where to set the grinder, in clicks, `22`; none when the recipe does
    /// not say.
    pub fn grind_label(&self) -> Option<String> {
        self.recipe.grind().map(GrindClicks::label)
    }

    /// Water per gram of coffee, `1:5`.
    pub fn ratio_label(&self) -> &str {
        &self.ratio_label
    }

    /// The ratio shown, and whether it is the recipe's own.
    pub fn ratio(&self) -> RatioSetting {
        self.ratio
    }

    /// The recipe's printed-label look, the same one the list shows.
    pub fn style(&self) -> LabelStyle {
        self.style
    }

    /// Total duration, `M:SS`.
    pub fn duration_label(&self) -> &str {
        &self.duration_label
    }
}

/// For each step, the label of the one after it; the last reads
/// [`BrewView::LAST_STEP_LABEL`].
fn next_labels(recipe: &Recipe, pours: &[Option<Pour>], metrics: &BrewMetrics) -> Vec<String> {
    let steps = recipe.steps();
    (1..=steps.len())
        .map(
            |next| match (steps.get(next), pours.get(next).copied().flatten()) {
                (Some(step), pour) => {
                    let duration = metrics.duration_label_of([step.duration()]);
                    match pour {
                        Some(pour) => {
                            format!("Next: {} · {duration} · {}", step.name(), pour.label())
                        }
                        None => format!("Next: {} · {duration}", step.name()),
                    }
                }
                (None, _) => BrewView::LAST_STEP_LABEL.to_owned(),
            },
        )
        .collect()
}
