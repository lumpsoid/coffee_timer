use crate::editor::field::Field;
use crate::model::{RecipeStep, StepDuration, StepId};

/// One step row of the draft, as typed. Its id is the row key and becomes
/// the saved step's id: an existing step keeps its id, a new row gets a
/// fresh one when it is added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DraftStep {
    id: StepId,
    name: String,
    duration: Field<StepDuration>,
}

impl DraftStep {
    /// Text a new row's duration field starts with.
    const NEW_DURATION_TEXT: &'static str = "0";

    /// A new row named `name`, `0` seconds long.
    pub(crate) fn new(id: StepId, name: &str) -> Self {
        Self {
            id,
            name: name.to_owned(),
            duration: Field::parse(Self::NEW_DURATION_TEXT, StepDuration::try_parse_text),
        }
    }

    /// A row showing a saved step.
    pub(crate) fn of(step: &RecipeStep) -> Self {
        Self {
            id: step.id().clone(),
            name: step.name().to_owned(),
            duration: Field::parse(
                &step.duration().seconds().to_string(),
                StepDuration::try_parse_text,
            ),
        }
    }

    pub(crate) fn set_name(&mut self, raw: &str) {
        raw.clone_into(&mut self.name);
    }

    pub(crate) fn set_duration(&mut self, raw: &str) {
        self.duration = Field::parse(raw, StepDuration::try_parse_text);
    }

    /// The step to save at `position` (1-based); none while the duration is
    /// invalid. A blank name saves as `Step N`.
    pub(crate) fn to_step(&self, position: usize) -> Option<RecipeStep> {
        let duration = *self.duration.value()?;
        Some(RecipeStep::new(
            self.id.clone(),
            RecipeStep::label_or_default(&self.name, position),
            duration,
        ))
    }

    /// The duration the typed text shows while typing: lenient, so the
    /// labels follow the text even while it is invalid.
    pub(crate) fn live_duration(&self) -> StepDuration {
        StepDuration::clamped_text(self.duration.raw())
    }

    pub(crate) fn id(&self) -> &StepId {
        &self.id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn duration(&self) -> &Field<StepDuration> {
        &self.duration
    }
}
