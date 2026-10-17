use crate::model::RecipeId;

/// One persisted change to the catalog. Returned only after the store has
/// written it; the shell passes it on as a list diff and to the brew view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogChange {
    Upserted(RecipeId),
    Removed(RecipeId),
}
