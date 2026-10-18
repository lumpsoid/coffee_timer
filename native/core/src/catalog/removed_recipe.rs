use crate::model::{Pin, Recipe};

/// The recipe the catalog removed last, whole, with the place it held in
/// catalog order and its pin: what an undo puts back. Held in memory only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovedRecipe {
    recipe: Recipe,
    index: usize,
    pin: Pin,
}

impl RemovedRecipe {
    pub(crate) fn new(recipe: Recipe, index: usize, pin: Pin) -> Self {
        Self { recipe, index, pin }
    }

    pub fn recipe(&self) -> &Recipe {
        &self.recipe
    }

    /// 0-based position the recipe held in catalog (stored) order when
    /// removed.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Whether the recipe was pinned when removed; an undo keeps it so.
    pub fn pin(&self) -> Pin {
        self.pin
    }

    /// The undo offer's text, `Deleted V60`.
    pub fn notice(&self) -> String {
        format!("Deleted {}", self.recipe.name().as_str())
    }

    pub(crate) fn into_recipe(self) -> Recipe {
        self.recipe
    }
}
