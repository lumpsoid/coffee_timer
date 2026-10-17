use std::cmp::Ordering;

use crate::brew::{RunningStep, StepRow, StepStatus};
use crate::model::{BrewMetrics, Recipe, RecipeId};

/// The open recipe as the brew page shows it. Built once per recipe version;
/// the timer's position is kept beside it in [`crate::brew::BrewState`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrewView {
    recipe: Recipe,
    coffee_label: String,
    water_label: String,
    duration_label: String,
}

impl BrewView {
    pub(crate) fn of(recipe: Recipe, metrics: &BrewMetrics) -> Self {
        Self {
            coffee_label: format!("{}g", metrics.coffee_grams(&recipe)),
            water_label: format!("{}g", metrics.water_grams(&recipe)),
            duration_label: metrics.duration_label(&recipe),
            recipe,
        }
    }

    /// The step at `index`, about to run with its full duration; none past
    /// the end.
    pub(crate) fn step(&self, index: usize) -> Option<RunningStep> {
        let step = self.recipe.steps().get(index)?;
        Some(RunningStep::begin(index, step.clone()))
    }

    pub(crate) fn step_count(&self) -> usize {
        self.recipe.steps().len()
    }

    /// Every step with its status: before `current` done, at it current,
    /// after it pending. With no current step, all pending.
    pub(crate) fn step_rows(&self, current: Option<usize>) -> Vec<StepRow> {
        self.recipe
            .steps()
            .iter()
            .enumerate()
            .map(|(index, step)| StepRow {
                number_label: (index + 1).to_string(),
                name: step.name().to_owned(),
                duration_label: step.duration().seconds_label(),
                status: match current.map(|c| index.cmp(&c)) {
                    Some(Ordering::Less) => StepStatus::Done,
                    Some(Ordering::Equal) => StepStatus::Current,
                    Some(Ordering::Greater) | None => StepStatus::Pending,
                },
            })
            .collect()
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

    /// Total duration, `M:SS`.
    pub fn duration_label(&self) -> &str {
        &self.duration_label
    }
}
