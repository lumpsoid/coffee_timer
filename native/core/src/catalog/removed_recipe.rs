use crate::model::Recipe;

/// The recipe the catalog removed last, whole, with the place it held in
/// catalog order: what an undo puts back. Held in memory only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovedRecipe {
    recipe: Recipe,
    index: usize,
}

impl RemovedRecipe {
    pub(crate) fn new(recipe: Recipe, index: usize) -> Self {
        Self { recipe, index }
    }

    pub fn recipe(&self) -> &Recipe {
        &self.recipe
    }

    /// 0-based position the recipe held in catalog order when removed.
    pub fn index(&self) -> usize {
        self.index
    }

    /// The undo offer's text, `Deleted V60`.
    pub fn notice(&self) -> String {
        format!("Deleted {}", self.recipe.name().as_str())
    }

    pub(crate) fn into_recipe(self) -> Recipe {
        self.recipe
    }
}
