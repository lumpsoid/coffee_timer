use crate::catalog::{CatalogChange, RecipeCatalog};
use crate::editor::{DeleteError, Draft, EditError, EditorState, SaveError, StepPreset};
use crate::model::{BrewMetrics, RecipeId, StepId};
use crate::ports::IdGen;

/// Owner of the recipe editor page: one draft, new or of an existing recipe.
///
/// Mints ids through its [`IdGen`]: a new recipe's id when the draft opens
/// (so a retried save keeps it), and a step id for every new row (the row
/// key, kept as the saved step's id). Writes go through the catalog, passed
/// in; a successful save or delete closes the draft.
pub struct EditorController {
    state: EditorState,
    ids: Box<dyn IdGen>,
    metrics: BrewMetrics,
}

impl EditorController {
    /// Starts closed.
    pub fn new(ids: Box<dyn IdGen>) -> Self {
        Self {
            state: EditorState::Closed,
            ids,
            metrics: BrewMetrics::default(),
        }
    }

    /// Opens a draft for a new recipe: no name, one blank `0` second step.
    /// Replaces any open draft.
    pub fn new_draft(&mut self) {
        self.state = EditorState::Open(Draft::new(self.ids.as_mut(), &self.metrics));
    }

    /// Opens a draft of the recipe with `id`, as stored. Replaces any open
    /// draft. An id the catalog does not hold is an error and changes nothing.
    pub fn edit(&mut self, id: &RecipeId, catalog: &RecipeCatalog) -> Result<(), EditError> {
        let recipe = catalog
            .get(id)
            .ok_or_else(|| EditError::UnknownRecipe(id.clone()))?;
        self.state = EditorState::Open(Draft::of(recipe, self.ids.as_mut(), &self.metrics));
        Ok(())
    }

    pub fn set_name(&mut self, raw: &str) -> Result<(), EditError> {
        self.draft_mut()?.set_name(raw);
        Ok(())
    }

    pub fn set_description(&mut self, raw: &str) -> Result<(), EditError> {
        self.draft_mut()?.set_description(raw);
        Ok(())
    }

    pub fn set_step_name(&mut self, row: &StepId, raw: &str) -> Result<(), EditError> {
        self.draft_mut()?.set_step_name(row, raw)
    }

    pub fn set_step_duration(&mut self, row: &StepId, raw: &str) -> Result<(), EditError> {
        self.draft_mut()?.set_step_duration(row, raw)
    }

    /// Appends a row named after `preset`, `0` seconds long.
    pub fn add_step(&mut self, preset: StepPreset) -> Result<(), EditError> {
        let draft = self.state.draft_mut().ok_or(EditError::NoDraft)?;
        draft.add_step(preset, self.ids.as_mut());
        Ok(())
    }

    /// Drops the row; removing the last row leaves one blank row instead.
    pub fn remove_step(&mut self, row: &StepId) -> Result<(), EditError> {
        let draft = self.state.draft_mut().ok_or(EditError::NoDraft)?;
        draft.remove_step(row, self.ids.as_mut())
    }

    /// Saves the draft to `catalog` and closes it. Field errors show from
    /// the first attempt on. On `Err` the catalog is unchanged and the draft
    /// stays open: [`SaveError::Invalid`] while a field is invalid,
    /// [`SaveError::Store`] when the write failed.
    pub fn save(&mut self, catalog: &mut RecipeCatalog) -> Result<CatalogChange, SaveError> {
        let draft = self.state.draft_mut().ok_or(SaveError::NoDraft)?;
        let recipe = draft.submit().ok_or(SaveError::Invalid)?;
        let change = catalog.upsert(recipe).map_err(SaveError::Store)?;
        self.state = EditorState::Closed;
        Ok(change)
    }

    /// Deletes the edited recipe from `catalog` and closes the draft. A new
    /// recipe has nothing to delete. On `Err` the catalog is unchanged and
    /// the draft stays open.
    pub fn delete(&mut self, catalog: &mut RecipeCatalog) -> Result<CatalogChange, DeleteError> {
        let draft = self.state.draft().ok_or(DeleteError::NoDraft)?;
        if !draft.can_delete() {
            return Err(DeleteError::NotSaved);
        }
        let change = catalog
            .remove(draft.id().clone())
            .map_err(DeleteError::Store)?;
        self.state = EditorState::Closed;
        Ok(change)
    }

    /// Leaves the page without saving.
    pub fn close(&mut self) {
        self.state = EditorState::Closed;
    }

    pub fn state(&self) -> &EditorState {
        &self.state
    }

    fn draft_mut(&mut self) -> Result<&mut Draft, EditError> {
        self.state.draft_mut().ok_or(EditError::NoDraft)
    }
}
