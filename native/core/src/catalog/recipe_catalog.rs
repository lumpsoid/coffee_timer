use std::collections::HashSet;

use crate::catalog::{
    CatalogChange, MoveError, PinError, PinnedFirst, RemovedRecipe, RestoreError,
};
use crate::model::{Pin, Recipe, RecipeId};
use crate::ports::{RecipeStore, StoreError};
use crate::transfer::{ExportError, RecipeDocument, RecipeExport};

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

    /// Appends `recipes`, new ids all, in order and unpinned, so they list
    /// after every recipe already there: all of them or, on `Err`, none.
    /// One `Upserted` per recipe, in order.
    pub fn append_all(&mut self, recipes: Vec<Recipe>) -> Result<Vec<CatalogChange>, StoreError> {
        self.store.append_all(&recipes)?;
        let changes = recipes
            .iter()
            .map(|recipe| CatalogChange::Upserted(recipe.id().clone()))
            .collect();
        self.state.extend(recipes);
        Ok(changes)
    }

    /// The recipes `ids` names, in that order, as a [`RecipeDocument`]; no
    /// ids is every recipe, in list order. An id the catalog does not hold
    /// is [`ExportError::UnknownRecipe`], and nothing is exported.
    pub fn export(&self, ids: &[RecipeId]) -> Result<RecipeExport, ExportError> {
        if ids.is_empty() {
            return RecipeDocument::encode(&self.listed());
        }
        let recipes = ids
            .iter()
            .map(|id| {
                self.get(id)
                    .ok_or_else(|| ExportError::UnknownRecipe(id.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        RecipeDocument::encode(&recipes)
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

    /// Moves the recipe `id` to `to`, its 0-based place in the list
    /// ([`RecipeCatalog::listed`] order) counted after the move, and
    /// reports it `Upserted`. A recipe moves only among those that share
    /// its pin, so a `to` outside that group puts it at the group's nearer
    /// end. Only the group's own stored places are reshuffled: the other
    /// group keeps its places, so an unpinned recipe still goes back among
    /// the unpinned where it stood. Moving a recipe to the place it has
    /// writes the order again. An id the catalog does not hold is
    /// [`MoveError::UnknownRecipe`].
    pub fn move_to(&mut self, id: &RecipeId, to: usize) -> Result<CatalogChange, MoveError> {
        let pin = self.pin_of(id);
        let mut group: Vec<RecipeId> = self
            .state
            .iter()
            .map(Recipe::id)
            .filter(|r| self.pin_of(r) == pin)
            .cloned()
            .collect();
        let Some(from) = group.iter().position(|r| r == id) else {
            return Err(MoveError::UnknownRecipe(id.clone()));
        };
        let first = if pin.is_pinned() {
            0
        } else {
            self.state.len() - group.len()
        };
        let moved = group.remove(from);
        group.insert(to.saturating_sub(first).min(group.len()), moved);
        // The group, in its new order, fills the stored places it had.
        let mut regrouped = group.into_iter();
        let order: Vec<RecipeId> = self
            .state
            .iter()
            .map(|r| {
                if self.pin_of(r.id()) == pin {
                    regrouped.next().unwrap_or_else(|| r.id().clone())
                } else {
                    r.id().clone()
                }
            })
            .collect();
        self.store.reorder(&order)?;
        self.state
            .sort_by_key(|r| order.iter().position(|o| o == r.id()));
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
