use std::collections::HashSet;

use crate::model::{Pin, Recipe, RecipeId};
use crate::ports::{RecipeStore, StoreError};

/// `Vec`-backed [`RecipeStore`] that keeps the trait's order contract. Can be
/// told to fail loads or writes, to test that a controller's state survives a
/// storage failure.
#[derive(Debug, Clone, Default)]
pub struct FakeRecipeStore {
    recipes: Vec<Recipe>,
    pinned: HashSet<RecipeId>,
    fail_loads: bool,
    fail_writes: bool,
    fail_inserts: bool,
}

impl FakeRecipeStore {
    /// A store already holding `recipes`, in that order, none pinned.
    pub fn with_recipes(recipes: Vec<Recipe>) -> Self {
        Self {
            recipes,
            ..Self::default()
        }
    }

    /// The same store with the recipe `id` stored pinned.
    pub fn pinning(mut self, id: &RecipeId) -> Self {
        self.pinned.insert(id.clone());
        self
    }

    /// Every `load_all` and `load_pinned` returns `Err`.
    pub fn failing_loads(self) -> Self {
        Self {
            fail_loads: true,
            ..self
        }
    }

    /// Every `upsert`, `insert_at`, `set_pin` and `remove` returns `Err` and
    /// stores nothing.
    pub fn failing_writes(self) -> Self {
        Self {
            fail_writes: true,
            ..self
        }
    }

    /// Every `insert_at` returns `Err` and stores nothing; other writes work.
    pub fn failing_inserts(self) -> Self {
        Self {
            fail_inserts: true,
            ..self
        }
    }

    fn check_load(&self) -> Result<(), StoreError> {
        if self.fail_loads {
            return Err(StoreError::new("fake load failure"));
        }
        Ok(())
    }

    fn check_write(&self) -> Result<(), StoreError> {
        if self.fail_writes {
            return Err(StoreError::new("fake write failure"));
        }
        Ok(())
    }
}

impl RecipeStore for FakeRecipeStore {
    fn load_all(&self) -> Result<Vec<Recipe>, StoreError> {
        self.check_load()?;
        Ok(self.recipes.clone())
    }

    fn load_pinned(&self) -> Result<Vec<RecipeId>, StoreError> {
        self.check_load()?;
        Ok(self
            .recipes
            .iter()
            .map(Recipe::id)
            .filter(|id| self.pinned.contains(*id))
            .cloned()
            .collect())
    }

    fn upsert(&mut self, recipe: &Recipe) -> Result<(), StoreError> {
        self.check_write()?;
        match self.recipes.iter_mut().find(|r| r.id() == recipe.id()) {
            Some(slot) => *slot = recipe.clone(),
            None => self.recipes.push(recipe.clone()),
        }
        Ok(())
    }

    fn insert_at(&mut self, recipe: &Recipe, index: usize, pin: Pin) -> Result<(), StoreError> {
        self.check_write()?;
        if self.fail_inserts {
            return Err(StoreError::new("fake insert failure"));
        }
        let index = index.min(self.recipes.len());
        self.recipes.insert(index, recipe.clone());
        if pin.is_pinned() {
            self.pinned.insert(recipe.id().clone());
        }
        Ok(())
    }

    fn set_pin(&mut self, id: &RecipeId, pin: Pin) -> Result<(), StoreError> {
        self.check_write()?;
        if pin.is_pinned() && self.recipes.iter().any(|r| r.id() == id) {
            self.pinned.insert(id.clone());
        } else {
            self.pinned.remove(id);
        }
        Ok(())
    }

    fn remove(&mut self, id: &RecipeId) -> Result<(), StoreError> {
        self.check_write()?;
        self.recipes.retain(|r| r.id() != id);
        self.pinned.remove(id);
        Ok(())
    }
}
