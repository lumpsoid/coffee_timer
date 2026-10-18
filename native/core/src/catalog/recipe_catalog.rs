use crate::catalog::{CatalogChange, RemovedRecipe, RestoreError};
use crate::model::{Recipe, RecipeId};
use crate::ports::{RecipeStore, StoreError};

/// Owner of the recipe list. Shared: other controllers receive it as an
/// action argument. Every action writes through the store first; a failed
/// write leaves the state unchanged.
///
/// It also keeps the recipe it removed last, so that one removal can be
/// undone: the next removal replaces it, a restore consumes it, and it is
/// never stored, so a restart drops it.
pub struct RecipeCatalog {
    state: Vec<Recipe>,
    removed: Option<RemovedRecipe>,
    store: Box<dyn RecipeStore>,
}

impl RecipeCatalog {
    /// Starts empty; call [`RecipeCatalog::load`] to read the store.
    pub fn new(store: Box<dyn RecipeStore>) -> Self {
        Self {
            state: Vec::new(),
            removed: None,
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

    /// Deletes the recipe with `id` and keeps it, with its place, as the
    /// one removal that can be undone. An unknown id is a no-op, as in the
    /// legacy app, and still reports `Removed`; it keeps the earlier undo.
    pub fn remove(&mut self, id: RecipeId) -> Result<CatalogChange, StoreError> {
        self.store.remove(&id)?;
        if let Some(index) = self.state.iter().position(|r| r.id() == &id) {
            self.removed = Some(RemovedRecipe::new(self.state.remove(index), index));
        }
        Ok(CatalogChange::Removed(id))
    }

    /// The removal of `id` that [`RecipeCatalog::restore`] would undo: the
    /// recipe removed last, while the catalog does not hold it again.
    pub fn restorable(&self, id: &RecipeId) -> Option<&RemovedRecipe> {
        self.removed
            .as_ref()
            .filter(|removed| removed.recipe().id() == id && self.get(id).is_none())
    }

    /// Undoes the removal of `id`: puts the whole recipe back where it was
    /// (at the end if the list has shrunk since), same id and steps, and
    /// reports it `Upserted`. The undo is used up. Any other id is
    /// [`RestoreError::NothingToUndo`]; a failed write leaves the undo on
    /// offer.
    pub fn restore(&mut self, id: &RecipeId) -> Result<CatalogChange, RestoreError> {
        if self.restorable(id).is_none() {
            return Err(RestoreError::NothingToUndo(id.clone()));
        }
        let Some(removed) = self.removed.take() else {
            return Err(RestoreError::NothingToUndo(id.clone()));
        };
        let index = removed.index().min(self.state.len());
        if let Err(e) = self.store.insert_at(removed.recipe(), index) {
            self.removed = Some(removed);
            return Err(e.into());
        }
        self.state.insert(index, removed.into_recipe());
        Ok(CatalogChange::Upserted(id.clone()))
    }

    /// The recipe with `id`, if the catalog holds it.
    pub fn get(&self, id: &RecipeId) -> Option<&Recipe> {
        self.state.iter().find(|r| r.id() == id)
    }

    /// The 0-based place of the recipe with `id` in catalog order.
    pub fn index_of(&self, id: &RecipeId) -> Option<usize> {
        self.state.iter().position(|r| r.id() == id)
    }

    pub fn state(&self) -> &[Recipe] {
        &self.state
    }
}
