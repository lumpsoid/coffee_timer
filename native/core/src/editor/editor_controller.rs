use crate::catalog::{CatalogChange, RecipeCatalog};
use crate::editor::{
    DeleteError, Draft, DurationNudge, EditError, EditorState, SaveError, StepPreset,
};
use crate::model::{BrewMetrics, RecipeId, StepId};
use crate::ports::IdGen;
use crate::transfer::{ImportError, RecipeDocument, RecipeImport};

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

    /// Opens a draft for a new recipe: no name, 15 g coffee to 250 g water,
    /// one blank `0` second step.
    /// Replaces any open draft.
    pub fn new_draft(&mut self) {
        self.state = EditorState::Open(Box::new(Draft::new(self.ids.as_mut(), &self.metrics)));
    }

    /// Opens a draft of the recipe with `id`, as stored. Replaces any open
    /// draft. An id the catalog does not hold is an error and changes nothing.
    pub fn edit(&mut self, id: &RecipeId, catalog: &RecipeCatalog) -> Result<(), EditError> {
        let recipe = catalog
            .get(id)
            .ok_or_else(|| EditError::UnknownRecipe(id.clone()))?;
        self.state = EditorState::Open(Box::new(Draft::of(
            recipe,
            self.ids.as_mut(),
            &self.metrics,
        )));
        Ok(())
    }

    /// Opens a draft of a new recipe copied from the recipe with `id`
    /// (see [`Draft`]'s copy: `<name> copy`, fresh ids). Replaces any open
    /// draft. Nothing is saved until [`EditorController::save`], which
    /// appends it; closing leaves the catalog as it was. An id the catalog
    /// does not hold is an error and changes nothing.
    pub fn duplicate(&mut self, id: &RecipeId, catalog: &RecipeCatalog) -> Result<(), EditError> {
        let recipe = catalog
            .get(id)
            .ok_or_else(|| EditError::UnknownRecipe(id.clone()))?;
        self.state = EditorState::Open(Box::new(Draft::copy_of(
            recipe,
            self.ids.as_mut(),
            &self.metrics,
        )));
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

    /// The coffee grams as typed.
    pub fn set_coffee(&mut self, raw: &str) -> Result<(), EditError> {
        self.draft_mut()?.set_coffee(raw);
        Ok(())
    }

    /// The water grams as typed.
    pub fn set_water(&mut self, raw: &str) -> Result<(), EditError> {
        self.draft_mut()?.set_water(raw);
        Ok(())
    }

    pub fn set_step_name(&mut self, row: &StepId, raw: &str) -> Result<(), EditError> {
        self.draft_mut()?.set_step_name(row, raw)
    }

    pub fn set_step_duration(&mut self, row: &StepId, raw: &str) -> Result<(), EditError> {
        self.draft_mut()?.set_step_duration(row, raw)
    }

    /// A step card's quick button: moves the row's duration by the nudge's
    /// seconds, from what its `MM:SS` label shows (text that is not a number
    /// counts as `0`), clamped to `0..=5999`. The duration text becomes the
    /// new seconds, so the row's duration error clears and the total
    /// follows; the row's `duration_revision` moves on.
    pub fn nudge_step_duration(
        &mut self,
        row: &StepId,
        nudge: DurationNudge,
    ) -> Result<(), EditError> {
        self.draft_mut()?.nudge_step_duration(row, nudge)
    }

    /// The grams the step pours, as typed; blank is no pour.
    pub fn set_step_water(&mut self, row: &StepId, raw: &str) -> Result<(), EditError> {
        self.draft_mut()?.set_step_water(row, raw)
    }

    /// Appends a row named after `preset`, `0` seconds long, and returns
    /// its id: the key for editing the new row.
    pub fn add_step(&mut self, preset: StepPreset) -> Result<StepId, EditError> {
        let draft = self.state.draft_mut().ok_or(EditError::NoDraft)?;
        Ok(draft.add_step(preset, self.ids.as_mut()))
    }

    /// Drops the row; removing the last row leaves one blank row instead.
    pub fn remove_step(&mut self, row: &StepId) -> Result<(), EditError> {
        let draft = self.state.draft_mut().ok_or(EditError::NoDraft)?;
        draft.remove_step(row, self.ids.as_mut())
    }

    /// Moves the row to position `to` (0-based, counted after the move),
    /// with its typed text, errors and pour; step labels follow the new
    /// positions. An unknown row or a position past the last row is an error
    /// and changes nothing.
    pub fn move_step(&mut self, row: &StepId, to: usize) -> Result<(), EditError> {
        self.draft_mut()?.move_step(row, to)
    }

    /// Saves the draft to `catalog` and closes it. Field errors show from
    /// the first attempt on. On `Err` the catalog is unchanged and the draft
    /// stays open: [`SaveError::Invalid`] while a field is invalid,
    /// [`SaveError::Store`] when the write failed, which the draft also
    /// shows as its notice.
    pub fn save(&mut self, catalog: &mut RecipeCatalog) -> Result<CatalogChange, SaveError> {
        let draft = self.state.draft_mut().ok_or(SaveError::NoDraft)?;
        let recipe = draft.submit().ok_or(SaveError::Invalid)?;
        match catalog.upsert(recipe) {
            Ok(change) => {
                self.state = EditorState::Closed;
                Ok(change)
            }
            Err(e) => {
                let error = SaveError::Store(e);
                draft.notify(&error);
                Err(error)
            }
        }
    }

    /// Deletes the edited recipe from `catalog` and closes the draft. A new
    /// recipe has nothing to delete. On `Err` the catalog is unchanged and
    /// the draft stays open; a failed write is also the draft's notice.
    pub fn delete(&mut self, catalog: &mut RecipeCatalog) -> Result<CatalogChange, DeleteError> {
        let draft = self.state.draft_mut().ok_or(DeleteError::NoDraft)?;
        if !draft.can_delete() {
            return Err(DeleteError::NotSaved);
        }
        match catalog.remove(draft.id().clone()) {
            Ok(change) => {
                self.state = EditorState::Closed;
                Ok(change)
            }
            Err(e) => {
                let error = DeleteError::Store(e);
                draft.notify(&error);
                Err(error)
            }
        }
    }

    /// Adds every recipe in the document `text` to `catalog` as a new
    /// recipe, after the ones there: fresh recipe and step ids from this
    /// controller's [`IdGen`], unpinned. All or nothing: a document with
    /// any invalid value, or a failed write, adds none and mints no id.
    /// The draft, open or not, is left alone.
    ///
    /// It lives here because the editor is the one minter of new recipes'
    /// ids; a second `IdGen` could hand out an id the editor also does.
    pub fn import(
        &mut self,
        text: &str,
        catalog: &mut RecipeCatalog,
    ) -> Result<RecipeImport, ImportError> {
        let recipes = RecipeDocument::decode(text)?
            .into_iter()
            .map(|recipe| recipe.into_recipe(self.ids.as_mut()))
            .collect();
        Ok(RecipeImport::new(catalog.append_all(recipes)?))
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
