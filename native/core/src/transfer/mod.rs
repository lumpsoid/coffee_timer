//! Recipes as a portable JSON document: export from the catalog, import
//! as new recipes. Pure functions over strings; reading and writing the
//! text (a file, the clipboard) is the shells' work.

mod document_error;
mod export_error;
mod import_error;
mod imported_recipe;
mod imported_step;
mod recipe_document;
mod recipe_export;
mod recipe_import;
mod wire;

pub use document_error::DocumentError;
pub use export_error::ExportError;
pub use import_error::ImportError;
pub use imported_recipe::ImportedRecipe;
pub use imported_step::ImportedStep;
pub use recipe_document::RecipeDocument;
pub use recipe_export::RecipeExport;
pub use recipe_import::RecipeImport;

/// `1 recipe`, `2 recipes`.
fn recipes_label(count: usize) -> String {
    if count == 1 {
        "1 recipe".to_owned()
    } else {
        format!("{count} recipes")
    }
}
