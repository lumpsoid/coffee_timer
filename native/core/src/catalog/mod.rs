//! The recipe catalog: the shared list of recipes and its persistence.

mod catalog_change;
mod recipe_catalog;
mod recipe_summary;
mod removed_recipe;
mod restore_error;

pub use catalog_change::CatalogChange;
pub use recipe_catalog::RecipeCatalog;
pub use recipe_summary::RecipeSummary;
pub use removed_recipe::RemovedRecipe;
pub use restore_error::RestoreError;
