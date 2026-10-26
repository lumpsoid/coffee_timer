use crate::model::{BrewMetrics, GrindClicks, LabelStyle, Pin, Recipe, RecipeId};

/// One row of the recipe list, ready to show. Labels are the values the
/// legacy `RecipeSummaryTile` shows (`20 g`, `100 g`, `1:35`); its captions
/// (`Coffee`, `Water`, `Time`) are static text and stay in the UI.
/// `grind_label` (`22`, under a `Clicks` caption) is none for a recipe
/// without a grind. `style`
/// is how the row's card looks; `pin` whether it shows the pin marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeSummary {
    pub id: RecipeId,
    pub name: String,
    pub description: String,
    pub coffee_label: String,
    pub water_label: String,
    pub time_label: String,
    pub grind_label: Option<String>,
    pub style: LabelStyle,
    pub pin: Pin,
}

impl RecipeSummary {
    pub fn of(recipe: &Recipe, pin: Pin, metrics: &BrewMetrics) -> Self {
        Self {
            id: recipe.id().clone(),
            name: recipe.name().as_str().to_owned(),
            description: recipe.description().to_owned(),
            coffee_label: format!("{} g", recipe.coffee().grams()),
            water_label: format!("{} g", recipe.water().grams()),
            time_label: metrics.duration_label(recipe),
            grind_label: recipe.grind().map(GrindClicks::label),
            style: LabelStyle::of(recipe, metrics),
            pin,
        }
    }
}
