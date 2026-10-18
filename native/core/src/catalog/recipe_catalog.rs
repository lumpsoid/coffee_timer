use std::collections::HashSet;

use crate::catalog::{CatalogChange, PinError, PinnedFirst, RemovedRecipe, RestoreError};
use crate::model::{Pin, Recipe, RecipeId};
use crate::ports::{RecipeStore, StoreError};

/// Owner of the recipe list. Shared: other controllers receive it as an
/// action argument. Every action writes through the store first; a failed
/// write leaves the state unchanged.
///
/// It also keeps the recipe it removed last, so that one removal can be
/// undone: the next removal replaces it, a restore consumes it, and it is
/// never stored, so a restart drops it.
///
/// Which recipes are pinned is catalog metadata beside the recipes, not a
/// field of [`Recipe`]: the editor, the brew page and a duplicate never
/// see it, and pinning the open recipe is no change to what it brews. The
/// list shows the recipes in [`PinnedFirst`] order ([`RecipeCatalog::listed`]).
pub struct RecipeCatalog {
    state: Vec<Recipe>,
    pinned: HashSet<RecipeId>,
    removed: Option<RemovedRecipe>,
    store: Box<dyn RecipeStore>,
}

impl RecipeCatalog {
    /// Starts empty; call [`RecipeCatalog::load`] to read the store.
    pub fn new(store: Box<dyn RecipeStore>) -> Self {
        Self {
            state: Vec::new(),
            pinned: HashSet::new(),
            removed: None,
            store,
        }
    }

    /// Replaces the state with everything stored, in stored order, and
    /// which of it is pinned.
    ///
    /// Boot-only: it returns no [`CatalogChange`], so a later reload would
    /// leave the brew's copy of the open recipe stale.
    pub fn load(&mut self) -> Result<(), StoreError> {
        let recipes = self.store.load_all()?;
        let pinned = self.store.load_pinned()?;
        self.state = recipes;
        self.pinned = pinned.into_iter().collect();
        Ok(())
    }

    /// Saves `recipe`. A known id is replaced in place and keeps its pin; a
    /// new id is appended, unpinned.
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
            let pin = self.pin_of(&id);
            self.pinned.remove(&id);
            self.removed = Some(RemovedRecipe::new(self.state.remove(index), index, pin));
        }
        Ok(CatalogChange::Removed(id))
    }

    /// Pins the recipe `id`, or unpins it, as `pin` says, and reports it
    /// `Upserted`: its row now shows the pin, at its new place in the
    /// list. The recipe itself, and its stored position, are unchanged.
    /// Setting the pin it already has writes it again. An id the catalog
    /// does not hold is [`PinError::UnknownRecipe`].
    pub fn set_pin(&mut self, id: &RecipeId, pin: Pin) -> Result<CatalogChange, PinError> {
        if self.get(id).is_none() {
            return Err(PinError::UnknownRecipe(id.clone()));
        }
        self.store.set_pin(id, pin)?;
        match pin {
            Pin::Pinned => self.pinned.insert(id.clone()),
            Pin::Unpinned => self.pinned.remove(id),
        };
        Ok(CatalogChange::Upserted(id.clone()))
    }

    /// The removal of `id` that [`RecipeCatalog::restore`] would undo: the
    /// recipe removed last, while the catalog does not hold it again.
    pub fn restorable(&self, id: &RecipeId) -> Option<&RemovedRecipe> {
        self.removed
            .as_ref()
            .filter(|removed| removed.recipe().id() == id && self.get(id).is_none())
    }

    /// Undoes the removal of `id`: puts the whole recipe back where it was
    /// (at the end if the list has shrunk since), same id, steps and pin, and
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
        let pin = removed.pin();
        if let Err(e) = self.store.insert_at(removed.recipe(), index, pin) {
            self.removed = Some(removed);
            return Err(e.into());
        }
        if pin.is_pinned() {
            self.pinned.insert(id.clone());
        }
        self.state.insert(index, removed.into_recipe());
        Ok(CatalogChange::Upserted(id.clone()))
    }

    /// The recipe with `id`, if the catalog holds it.
    pub fn get(&self, id: &RecipeId) -> Option<&Recipe> {
        self.state.iter().find(|r| r.id() == id)
    }

    /// Whether the recipe `id` is pinned; an id the catalog does not hold
    /// is not.
    pub fn pin_of(&self, id: &RecipeId) -> Pin {
        Pin::from(self.pinned.contains(id))
    }

    /// Every recipe in the order the list shows them: [`PinnedFirst`].
    pub fn listed(&self) -> Vec<&Recipe> {
        PinnedFirst::order(&self.state, |recipe| self.pin_of(recipe.id()))
    }

    /// The 0-based place of the recipe with `id` in the list
    /// ([`RecipeCatalog::listed`] order).
    pub fn index_of(&self, id: &RecipeId) -> Option<usize> {
        self.listed().iter().position(|r| r.id() == id)
    }

    /// Every recipe in catalog (stored) order, pinned or not. The list
    /// shows them in [`RecipeCatalog::listed`] order instead.
    pub fn state(&self) -> &[Recipe] {
        &self.state
    }
}
