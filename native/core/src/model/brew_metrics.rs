use crate::model::{BrewRatio, Recipe, RecipeStep, StepDuration};

/// The one home for the brew figures derived from a recipe: the ratio of
/// its stored coffee and water, and its total duration. Every threshold and
/// formula lives here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrewMetrics {
    /// Total brew duration is capped here.
    total_cap_seconds: u32,
}

impl Default for BrewMetrics {
    fn default() -> Self {
        Self {
            total_cap_seconds: 6000,
        }
    }
}

impl BrewMetrics {
    /// The recipe's water per gram of its coffee.
    pub fn ratio(&self, recipe: &Recipe) -> BrewRatio {
        BrewRatio::of(recipe.coffee(), recipe.water())
    }

    /// Coffee-to-water ratio, `1:5`, to one decimal when it is not whole
    /// (`1:2.5`).
    pub fn ratio_label(&self, recipe: &Recipe) -> String {
        self.ratio(recipe).label()
    }

    /// Sum of every step's duration, clamped to the cap.
    pub fn total_seconds(&self, recipe: &Recipe) -> u32 {
        self.total_seconds_of(durations(recipe))
    }

    /// `M:SS` total-duration label. Minutes are unpadded; at the cap it reads
    /// `99:99` (both preserved from the legacy display).
    pub fn duration_label(&self, recipe: &Recipe) -> String {
        self.duration_label_of(durations(recipe))
    }

    /// Sum of `steps`, clamped to the cap. Also used by the edit form, which
    /// needs a live total before it has assembled a `Recipe`.
    pub fn total_seconds_of(&self, steps: impl IntoIterator<Item = StepDuration>) -> u32 {
        let mut total: u32 = 0;
        for step in steps {
            total = total.saturating_add(step.seconds());
            if total >= self.total_cap_seconds {
                return self.total_cap_seconds;
            }
        }
        total
    }

    /// [`BrewMetrics::duration_label`] over raw durations.
    pub fn duration_label_of(&self, steps: impl IntoIterator<Item = StepDuration>) -> String {
        let total = self.total_seconds_of(steps);
        let minutes = total / 60;
        if minutes >= 100 {
            return "99:99".to_owned();
        }
        format!("{minutes}:{:02}", total % 60)
    }
}

fn durations(recipe: &Recipe) -> impl Iterator<Item = StepDuration> + '_ {
    recipe.steps().iter().map(RecipeStep::duration)
}
