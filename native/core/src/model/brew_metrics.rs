use crate::model::{BrewRatio, Recipe, RecipeStep, StepDuration};

/// The one home for the brew figures shown on a recipe: coffee grams, water
/// grams, their ratio, and total duration. Every threshold and formula lives here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrewMetrics {
    coffee_per_step_g: u32,
    water_per_step_g: u32,
    /// Total brew duration is capped here.
    total_cap_seconds: u32,
}

impl Default for BrewMetrics {
    fn default() -> Self {
        Self {
            coffee_per_step_g: 10,
            water_per_step_g: 50,
            total_cap_seconds: 6000,
        }
    }
}

impl BrewMetrics {
    pub fn coffee_grams(&self, recipe: &Recipe) -> u32 {
        step_count(recipe).saturating_mul(self.coffee_per_step_g)
    }

    pub fn water_grams(&self, recipe: &Recipe) -> u32 {
        step_count(recipe).saturating_mul(self.water_per_step_g)
    }

    /// Water per gram of coffee; none for a recipe without coffee.
    pub fn ratio(&self, recipe: &Recipe) -> Option<BrewRatio> {
        BrewRatio::of(self.coffee_grams(recipe), self.water_grams(recipe))
    }

    /// Coffee-to-water ratio, `1:5`, to one decimal when it is not whole
    /// (`1:2.5`). A recipe without coffee reads `—`.
    pub fn ratio_label(&self, recipe: &Recipe) -> String {
        self.ratio(recipe)
            .map_or_else(|| BrewRatio::NONE_LABEL.to_owned(), BrewRatio::label)
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

fn step_count(recipe: &Recipe) -> u32 {
    u32::try_from(recipe.steps().len()).unwrap_or(u32::MAX)
}

fn durations(recipe: &Recipe) -> impl Iterator<Item = StepDuration> + '_ {
    recipe.steps().iter().map(RecipeStep::duration)
}
