use crate::catalog::CatalogChange;

/// The recipes an import added: one persisted `Upserted` per recipe, in
/// document order. The shell passes each on like any change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeImport {
    changes: Vec<CatalogChange>,
}

impl RecipeImport {
    pub(crate) fn new(changes: Vec<CatalogChange>) -> Self {
        Self { changes }
    }

    pub fn changes(&self) -> &[CatalogChange] {
        &self.changes
    }

    /// `Imported 2 recipes`.
    pub fn notice(&self) -> String {
        format!("Imported {}", super::recipes_label(self.changes.len()))
    }
}
