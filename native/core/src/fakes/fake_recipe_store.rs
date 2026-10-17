use crate::model::{Recipe, RecipeId};
use crate::ports::{RecipeStore, StoreError};

/// `Vec`-backed [`RecipeStore`] that keeps the trait's order contract. Can be
/// told to fail loads or writes, to test that a controller's state survives a
/// storage failure.
#[derive(Debug, Clone, Default)]
pub struct FakeRecipeStore {
    recipes: Vec<Recipe>,
    fail_loads: bool,
    fail_writes: bool,
}

impl FakeRecipeStore {
    /// A store already holding `recipes`, in that order.
    pub fn with_recipes(recipes: Vec<Recipe>) -> Self {
        Self {
            recipes,
            ..Self::default()
        }
    }

    /// Every `load_all` returns `Err`.
    pub fn failing_loads(self) -> Self {
        Self {
            fail_loads: true,
            ..self
        }
    }

    /// Every `upsert` and `remove` returns `Err` and stores nothing.
    pub fn failing_writes(self) -> Self {
        Self {
            fail_writes: true,
            ..self
        }
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
        if self.fail_loads {
            return Err(StoreError::new("fake load failure"));
        }
        Ok(self.recipes.clone())
    }

    fn upsert(&mut self, recipe: &Recipe) -> Result<(), StoreError> {
        self.check_write()?;
        match self.recipes.iter_mut().find(|r| r.id() == recipe.id()) {
            Some(slot) => *slot = recipe.clone(),
            None => self.recipes.push(recipe.clone()),
        }
        Ok(())
    }

    fn remove(&mut self, id: &RecipeId) -> Result<(), StoreError> {
        self.check_write()?;
        self.recipes.retain(|r| r.id() != id);
        Ok(())
    }
}
