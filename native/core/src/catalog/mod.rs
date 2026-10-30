//! The recipe catalog: the shared list of recipes and its persistence.

mod catalog_change;
mod move_error;
mod pin_error;
mod pinned_first;
mod recipe_catalog;
mod recipe_summary;
mod removed_recipe;
mod restore_error;

pub use catalog_change::CatalogChange;
pub use move_error::MoveError;
pub use pin_error::PinError;
pub use pinned_first::PinnedFirst;
pub use recipe_catalog::RecipeCatalog;
pub use recipe_summary::RecipeSummary;
pub use removed_recipe::RemovedRecipe;
pub use restore_error::RestoreError;
