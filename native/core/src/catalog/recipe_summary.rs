use crate::model::{BrewMetrics, LabelStyle, Recipe, RecipeId};

/// One row of the recipe list, ready to show. Labels are the values the
/// legacy `RecipeSummaryTile` shows (`20 g`, `100 g`, `1:35`); its captions
/// (`Coffee`, `Water`, `Time`) are static text and stay in the UI. `style`
/// is how the row's card looks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeSummary {
    pub id: RecipeId,
    pub name: String,
    pub description: String,
    pub coffee_label: String,
    pub water_label: String,
    pub time_label: String,
    pub style: LabelStyle,
}

impl RecipeSummary {
    pub fn of(recipe: &Recipe, metrics: &BrewMetrics) -> Self {
        Self {
            id: recipe.id().clone(),
            name: recipe.name().as_str().to_owned(),
            description: recipe.description().to_owned(),
            coffee_label: format!("{} g", metrics.coffee_grams(recipe)),
            water_label: format!("{} g", metrics.water_grams(recipe)),
            time_label: metrics.duration_label(recipe),
            style: LabelStyle::of(recipe, metrics),
        }
    }
}
