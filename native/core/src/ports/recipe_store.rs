use crate::model::{Pin, Recipe, RecipeId};
use crate::ports::StoreError;

/// Persistent recipes. Owned by `RecipeCatalog`, which writes through it
/// before changing its own state.
///
/// Order is part of the contract: `load_all` returns recipes in stored order,
/// `upsert` of a known id keeps its position, and `upsert` of a new id
/// appends, and `insert_at` puts a new id at the place it names. A
/// recipe's [`Pin`] is stored beside it and never moves it: the list's
/// pinned-first order is the catalog's, not the store's.
pub trait RecipeStore: Send {
    fn load_all(&self) -> Result<Vec<Recipe>, StoreError>;

    /// The ids of the stored recipes that are pinned, in stored order.
    fn load_pinned(&self) -> Result<Vec<RecipeId>, StoreError>;

    /// Inserts `recipe`, or replaces the stored recipe with the same id
    /// (name, description, and all steps in order). A new id is unpinned;
    /// a known id keeps its pin.
    fn upsert(&mut self, recipe: &Recipe) -> Result<(), StoreError>;

    /// Appends `recipes`, whose ids are not stored, in order and unpinned:
    /// all of them or, on `Err`, none. Imports a document.
    fn append_all(&mut self, recipes: &[Recipe]) -> Result<(), StoreError>;

    /// Inserts `recipe`, whose id is not stored, with `pin`, so that
    /// `load_all` returns it at `index`, before the recipe there now; an
    /// index at or past the end appends. Puts a removed recipe back in its
    /// place.
    fn insert_at(&mut self, recipe: &Recipe, index: usize, pin: Pin) -> Result<(), StoreError>;

    /// Stores `pin` for the stored recipe `id`; its position stays. An
    /// unknown id is not an error.
    fn set_pin(&mut self, id: &RecipeId, pin: Pin) -> Result<(), StoreError>;

    /// Deletes the recipe and its steps. An unknown id is not an error.
    fn remove(&mut self, id: &RecipeId) -> Result<(), StoreError>;
}
