use crate::model::{Recipe, RecipeId};
use crate::ports::StoreError;

/// Persistent recipes. Owned by `RecipeCatalog`, which writes through it
/// before changing its own state.
///
/// Order is part of the contract: `load_all` returns recipes in stored order,
/// `upsert` of a known id keeps its position, and `upsert` of a new id
/// appends, and `insert_at` puts a new id at the place it names.
pub trait RecipeStore: Send {
    fn load_all(&self) -> Result<Vec<Recipe>, StoreError>;

    /// Inserts `recipe`, or replaces the stored recipe with the same id
    /// (name, description, and all steps in order).
    fn upsert(&mut self, recipe: &Recipe) -> Result<(), StoreError>;

    /// Inserts `recipe`, whose id is not stored, so that `load_all` returns
    /// it at `index`, before the recipe there now; an index at or past the
    /// end appends. Puts a removed recipe back in its place.
    fn insert_at(&mut self, recipe: &Recipe, index: usize) -> Result<(), StoreError>;

    /// Deletes the recipe and its steps. An unknown id is not an error.
    fn remove(&mut self, id: &RecipeId) -> Result<(), StoreError>;
}
