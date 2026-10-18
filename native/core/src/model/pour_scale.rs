use crate::model::{Pour, RecipeStep, WaterAmount};

/// The one rounding rule for a recipe's step pours in a brew whose water
/// differs from the recipe's (an adjusted ratio): every pour scales by the
/// brew's water over the recipe's.
///
/// The running totals are scaled, each rounded half up to a gram, and a
/// step's pour is the difference between its total and the one before.
/// So the pours shown always add up to the "pour to" targets shown, and
/// rounding never drifts over the steps. At the recipe's own water the
/// scale is exact: every pour stays as stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PourScale {
    recipe_water: u32,
    brew_water: u32,
}

impl PourScale {
    /// From the recipe's stored water to `brew_water_grams`, the water of
    /// this brew. An adjusted brew may use more water than a recipe can
    /// store, so the brew side is plain grams.
    pub fn new(recipe_water: WaterAmount, brew_water_grams: u32) -> Self {
        Self {
            recipe_water: recipe_water.grams().max(1),
            brew_water: brew_water_grams,
        }
    }

    /// `grams` of the recipe's water at this brew's water, rounded half up.
    pub fn scaled(self, grams: u64) -> u32 {
        let recipe = u64::from(self.recipe_water);
        let product = grams.saturating_mul(u64::from(self.brew_water));
        let rounded = product.saturating_mul(2).saturating_add(recipe) / recipe.saturating_mul(2);
        u32::try_from(rounded).unwrap_or(u32::MAX)
    }

    /// Each step's pour in this brew, in step order; none for a step that
    /// pours nothing.
    pub fn pours(self, steps: &[RecipeStep]) -> Vec<Option<Pour>> {
        let mut stored_total: u64 = 0;
        let mut poured: u32 = 0;
        steps
            .iter()
            .map(|step| {
                let water = step.water()?;
                stored_total = stored_total.saturating_add(u64::from(water.grams()));
                let to_grams = self.scaled(stored_total);
                let pour = Pour::new(to_grams.saturating_sub(poured), to_grams);
                poured = to_grams;
                Some(pour)
            })
            .collect()
    }
}
