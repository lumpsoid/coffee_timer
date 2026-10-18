use std::fmt;

use crate::editor::draft_step::DraftStep;
use crate::editor::field::Field;
use crate::editor::{DraftMode, EditError, EditorStepRow, StepPreset};
use crate::model::{
    BrewMetrics, CoffeeDose, ModelError, Recipe, RecipeId, RecipeName, StepId, WaterAmount,
};
use crate::ports::IdGen;

/// The recipe being edited, as typed: the whole editor view state.
///
/// Holds raw text so the form can show in-progress input a model value
/// would reject. Always has at least one step row. Field errors are hidden
/// until the first save attempt, then follow the text. A save or delete the
/// store rejected leaves a notice, cleared by the next action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    id: RecipeId,
    mode: DraftMode,
    name: Field<RecipeName>,
    description: String,
    coffee: Field<CoffeeDose>,
    water: Field<WaterAmount>,
    steps: Vec<DraftStep>,
    errors_shown: bool,
    notice: Option<String>,
    metrics: BrewMetrics,
}

impl Draft {
    /// Text a new recipe's coffee field starts with: a single pour-over cup.
    const NEW_COFFEE_TEXT: &'static str = "15";
    /// Text a new recipe's water field starts with, `1:16.7` to the coffee.
    const NEW_WATER_TEXT: &'static str = "250";

    /// A new recipe: fresh id, no name, a one-cup dose, one blank step.
    pub(crate) fn new(ids: &mut dyn IdGen, metrics: &BrewMetrics) -> Self {
        let id = ids.recipe_id();
        Self {
            id,
            mode: DraftMode::New,
            name: Field::parse("", RecipeName::try_parse),
            description: String::new(),
            coffee: Field::parse(Self::NEW_COFFEE_TEXT, CoffeeDose::try_parse_text),
            water: Field::parse(Self::NEW_WATER_TEXT, WaterAmount::try_parse_text),
            steps: vec![DraftStep::new(ids.step_id(), StepPreset::Blank.step_name())],
            errors_shown: false,
            notice: None,
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
            coffee: Field::parse(
                &recipe.coffee().grams().to_string(),
                CoffeeDose::try_parse_text,
            ),
            water: Field::parse(
                &recipe.water().grams().to_string(),
                WaterAmount::try_parse_text,
            ),
            steps: recipe.steps().iter().map(DraftStep::of).collect(),
            errors_shown: false,
            notice: None,
            metrics: *metrics,
        };
        draft.keep_a_step(ids);
        draft
    }

    /// Shows `failure` until the next action: a save or delete the store
    /// rejected (legacy snackbar `Failed to save recipe: …`).
    pub(crate) fn notify(&mut self, failure: &impl fmt::Display) {
        self.notice = Some(failure.to_string());
    }

    pub(crate) fn clear_notice(&mut self) {
        self.notice = None;
    }

    pub(crate) fn set_name(&mut self, raw: &str) {
        self.name = Field::parse(raw, RecipeName::try_parse);
    }

    pub(crate) fn set_description(&mut self, raw: &str) {
        raw.clone_into(&mut self.description);
    }

    pub(crate) fn set_coffee(&mut self, raw: &str) {
        self.coffee = Field::parse(raw, CoffeeDose::try_parse_text);
    }

    pub(crate) fn set_water(&mut self, raw: &str) {
        self.water = Field::parse(raw, WaterAmount::try_parse_text);
    }

    pub(crate) fn set_step_name(&mut self, row: &StepId, raw: &str) -> Result<(), EditError> {
        self.step_mut(row)?.set_name(raw);
        Ok(())
    }

    pub(crate) fn set_step_duration(&mut self, row: &StepId, raw: &str) -> Result<(), EditError> {
        self.step_mut(row)?.set_duration(raw);
        Ok(())
    }

    /// Appends a row named after `preset`, `0` seconds long; returns its id.
    pub(crate) fn add_step(&mut self, preset: StepPreset, ids: &mut dyn IdGen) -> StepId {
        let id = ids.step_id();
        self.steps
            .push(DraftStep::new(id.clone(), preset.step_name()));
        id
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
        let coffee = *self.coffee.value()?;
        let water = *self.water.value()?;
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
            coffee,
            water,
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

    /// The coffee grams as typed.
    pub fn coffee(&self) -> &str {
        self.coffee.raw()
    }

    /// `Enter grams` or `1–200 g`, once errors are shown.
    pub fn coffee_error(&self) -> Option<String> {
        self.shown(self.coffee.error())
    }

    /// The water grams as typed.
    pub fn water(&self) -> &str {
        self.water.raw()
    }

    /// `Enter grams` or `1–2000 g`, once errors are shown.
    pub fn water_error(&self) -> Option<String> {
        self.shown(self.water.error())
    }

    /// Live `M:SS` total of the typed durations, read leniently and capped
    /// like a saved recipe's total.
    pub fn total_label(&self) -> String {
        self.metrics
            .duration_label_of(self.steps.iter().map(DraftStep::live_duration))
    }

    /// Why the last save or delete failed in the store, for a snackbar; none
    /// after any other outcome. Invalid fields leave no notice: their errors
    /// are already on the fields.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
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
            let _blank_row = self.add_step(StepPreset::Blank, ids);
        }
    }
}
