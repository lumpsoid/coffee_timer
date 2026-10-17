use crate::editor::draft_step::DraftStep;
use crate::editor::field::Field;
use crate::editor::{DraftMode, EditError, EditorStepRow, StepPreset};
use crate::model::{BrewMetrics, ModelError, Recipe, RecipeId, RecipeName, StepId};
use crate::ports::IdGen;

/// The recipe being edited, as typed: the whole editor view state.
///
/// Holds raw text so the form can show in-progress input a model value
/// would reject. Always has at least one step row. Field errors are hidden
/// until the first save attempt, then follow the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    id: RecipeId,
    mode: DraftMode,
    name: Field<RecipeName>,
    description: String,
    steps: Vec<DraftStep>,
    errors_shown: bool,
    metrics: BrewMetrics,
}

impl Draft {
    /// A new recipe: fresh id, no name, one blank step.
    pub(crate) fn new(ids: &mut dyn IdGen, metrics: &BrewMetrics) -> Self {
        let id = ids.recipe_id();
        Self {
            id,
            mode: DraftMode::New,
            name: Field::parse("", RecipeName::try_parse),
            description: String::new(),
            steps: vec![DraftStep::new(ids.step_id(), StepPreset::Blank.step_name())],
            errors_shown: false,
            metrics: *metrics,
        }
    }

    /// `recipe` as stored. A recipe without steps gets one blank row.
    pub(crate) fn of(recipe: &Recipe, ids: &mut dyn IdGen, metrics: &BrewMetrics) -> Self {
        let mut draft = Self {
            id: recipe.id().clone(),
            mode: DraftMode::Existing,
            name: Field::parse(recipe.name().as_str(), RecipeName::try_parse),
            description: recipe.description().to_owned(),
            steps: recipe.steps().iter().map(DraftStep::of).collect(),
            errors_shown: false,
            metrics: *metrics,
        };
        draft.keep_a_step(ids);
        draft
    }

    pub(crate) fn set_name(&mut self, raw: &str) {
        self.name = Field::parse(raw, RecipeName::try_parse);
    }

    pub(crate) fn set_description(&mut self, raw: &str) {
        raw.clone_into(&mut self.description);
    }

    pub(crate) fn set_step_name(&mut self, row: &StepId, raw: &str) -> Result<(), EditError> {
        self.step_mut(row)?.set_name(raw);
        Ok(())
    }

    pub(crate) fn set_step_duration(&mut self, row: &StepId, raw: &str) -> Result<(), EditError> {
        self.step_mut(row)?.set_duration(raw);
        Ok(())
    }

    /// Appends a row named after `preset`, `0` seconds long.
    pub(crate) fn add_step(&mut self, preset: StepPreset, ids: &mut dyn IdGen) {
        self.steps
            .push(DraftStep::new(ids.step_id(), preset.step_name()));
    }

    /// Drops the row; removing the last row leaves one blank row instead.
    pub(crate) fn remove_step(
        &mut self,
        row: &StepId,
        ids: &mut dyn IdGen,
    ) -> Result<(), EditError> {
        let index = self
            .steps
            .iter()
            .position(|step| step.id() == row)
            .ok_or_else(|| EditError::UnknownStep(row.clone()))?;
        self.steps.remove(index);
        self.keep_a_step(ids);
        Ok(())
    }

    /// A save attempt: from now on field errors are shown. The recipe to
    /// save, or none while a field is invalid. The name and description are
    /// trimmed; a blank step name saves as `Step N`.
    pub(crate) fn submit(&mut self) -> Option<Recipe> {
        self.errors_shown = true;
        let name = self.name.value()?.clone();
        let steps = self
            .steps
            .iter()
            .enumerate()
            .map(|(index, step)| step.to_step(index + 1))
            .collect::<Option<Vec<_>>>()?;
        Some(Recipe::new(
            self.id.clone(),
            name,
            self.description.trim().to_owned(),
            steps,
        ))
    }

    pub fn id(&self) -> &RecipeId {
        &self.id
    }

    pub fn mode(&self) -> DraftMode {
        self.mode
    }

    /// Page title: `New Recipe` or `Edit Recipe`.
    pub fn title(&self) -> &'static str {
        match self.mode {
            DraftMode::New => "New Recipe",
            DraftMode::Existing => "Edit Recipe",
        }
    }

    /// Delete is offered only for a recipe already in the catalog.
    pub fn can_delete(&self) -> bool {
        self.mode == DraftMode::Existing
    }

    /// The name as typed.
    pub fn name(&self) -> &str {
        self.name.raw()
    }

    /// `Enter a recipe name`, once errors are shown.
    pub fn name_error(&self) -> Option<String> {
        self.shown(self.name.error())
    }

    /// The description as typed.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Live `M:SS` total of the typed durations, read leniently and capped
    /// like a saved recipe's total.
    pub fn total_label(&self) -> String {
        self.metrics
            .duration_label_of(self.steps.iter().map(DraftStep::live_duration))
    }

    /// Every step row in order, with its live label and error.
    pub fn step_rows(&self) -> Vec<EditorStepRow> {
        self.steps
            .iter()
            .enumerate()
            .map(|(index, step)| EditorStepRow {
                id: step.id().clone(),
                name_label: format!("Step {} name", index + 1),
                name: step.name().to_owned(),
                duration_text: step.duration().raw().to_owned(),
                duration_label: step.live_duration().mmss(),
                duration_error: self.shown(step.duration().error()),
            })
            .collect()
    }

    fn shown(&self, error: Option<&ModelError>) -> Option<String> {
        error.filter(|_| self.errors_shown).map(ToString::to_string)
    }

    fn step_mut(&mut self, row: &StepId) -> Result<&mut DraftStep, EditError> {
        self.steps
            .iter_mut()
            .find(|step| step.id() == row)
            .ok_or_else(|| EditError::UnknownStep(row.clone()))
    }

    /// The at-least-one-row rule.
    fn keep_a_step(&mut self, ids: &mut dyn IdGen) {
        if self.steps.is_empty() {
            self.add_step(StepPreset::Blank, ids);
        }
    }
}
