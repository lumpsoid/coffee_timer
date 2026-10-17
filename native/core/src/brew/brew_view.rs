use std::cmp::Ordering;

use crate::brew::{RunningStep, StepRow, StepStatus};
use crate::model::{BrewMetrics, LabelStyle, Recipe, RecipeId};

/// The open recipe as the brew page shows it. Built once per recipe version;
/// the timer's position is kept beside it in [`crate::brew::BrewState`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrewView {
    recipe: Recipe,
    coffee_label: String,
    water_label: String,
    ratio_label: String,
    duration_label: String,
    style: LabelStyle,
}

impl BrewView {
    pub(crate) fn of(recipe: Recipe, metrics: &BrewMetrics) -> Self {
        Self {
            coffee_label: format!("{}g", metrics.coffee_grams(&recipe)),
            water_label: format!("{}g", metrics.water_grams(&recipe)),
            ratio_label: metrics.ratio_label(&recipe),
            duration_label: metrics.duration_label(&recipe),
            style: LabelStyle::of(&recipe, metrics),
            recipe,
        }
    }

    /// The step at `index`, about to run with its full duration in start
    /// number `run`; none past the end.
    pub(crate) fn step(&self, run: u64, index: usize) -> Option<RunningStep> {
        let step = self.recipe.steps().get(index)?;
        Some(RunningStep::begin(run, index, step.clone()))
    }

    pub(crate) fn step_count(&self) -> usize {
        self.recipe.steps().len()
    }

    /// Every step with its status: before `current` done, at it current,
    /// after it pending. With no current step, all pending.
    ///
    /// Each step also gets its share of the whole brew, laid end to end from
    /// 0 to 1, for the dial's step track. A brew of 0-second steps shares it
    /// equally.
    pub(crate) fn step_rows(&self, current: Option<usize>) -> Vec<StepRow> {
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
            .enumerate()
            .map(|(index, (step, &weight))| {
                let start = elapsed;
                elapsed = elapsed.saturating_add(weight);
                StepRow {
                    number_label: (index + 1).to_string(),
                    name: step.name().to_owned(),
                    duration_label: step.duration().seconds_label(),
                    status: match current.map(|c| index.cmp(&c)) {
                        Some(Ordering::Less) => StepStatus::Done,
                        Some(Ordering::Equal) => StepStatus::Current,
                        Some(Ordering::Greater) | None => StepStatus::Pending,
                    },
                    track_start: f64::from(start) / whole,
                    track_end: f64::from(elapsed) / whole,
                }
            })
            .collect()
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

    /// Water grams, `100g`.
    pub fn water_label(&self) -> &str {
        &self.water_label
    }

    /// Water per gram of coffee, `1:5`.
    pub fn ratio_label(&self) -> &str {
        &self.ratio_label
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
