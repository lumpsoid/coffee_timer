use crate::catalog::CatalogChange;
use crate::model::{Recipe, RecipeId};
use crate::ports::{RecipeStore, StoreError};

/// Owner of the recipe list. Shared: other controllers receive it as an
/// action argument. Every action writes through the store first; a failed
/// write leaves the state unchanged.
pub struct RecipeCatalog {
    state: Vec<Recipe>,
    store: Box<dyn RecipeStore>,
}

impl RecipeCatalog {
    /// Starts empty; call [`RecipeCatalog::load`] to read the store.
    pub fn new(store: Box<dyn RecipeStore>) -> Self {
        Self {
            state: Vec::new(),
            store,
        }
    }

    /// Replaces the state with everything stored, in stored order.
    ///
    /// Boot-only: it returns no [`CatalogChange`], so a later reload would
    /// leave the brew's copy of the open recipe stale.
    pub fn load(&mut self) -> Result<(), StoreError> {
        self.state = self.store.load_all()?;
        Ok(())
    }

    /// Saves `recipe`. A known id is replaced in place; a new id is appended.
    pub fn upsert(&mut self, recipe: Recipe) -> Result<CatalogChange, StoreError> {
        self.store.upsert(&recipe)?;
        let id = recipe.id().clone();
        match self.state.iter_mut().find(|r| r.id() == &id) {
            Some(slot) => *slot = recipe,
            None => self.state.push(recipe),
        }
        Ok(CatalogChange::Upserted(id))
    }

    /// Deletes the recipe with `id`. An unknown id is a no-op, as in the
    /// legacy app, and still reports `Removed`.
    pub fn remove(&mut self, id: RecipeId) -> Result<CatalogChange, StoreError> {
        self.store.remove(&id)?;
        self.state.retain(|r| r.id() != &id);
        Ok(CatalogChange::Removed(id))
    }

    pub fn state(&self) -> &[Recipe] {
        &self.state
    }
}
