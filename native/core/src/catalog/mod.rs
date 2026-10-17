//! The recipe catalog: the shared list of recipes and its persistence.

mod catalog_change;
mod recipe_catalog;
mod recipe_summary;

pub use catalog_change::CatalogChange;
pub use recipe_catalog::RecipeCatalog;
pub use recipe_summary::RecipeSummary;
