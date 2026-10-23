use crate::editor::field::Field;
use crate::model::{PourSpeed, RecipeStep, StepDuration, StepId, WaterAmount};

/// One step row of the draft, as typed. Its id is the row key and becomes
/// the saved step's id: an existing step keeps its id, a new row gets a
/// fresh one when it is added.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DraftStep {
    id: StepId,
    name: String,
    duration: Field<StepDuration>,
    /// Counts the times the duration text was replaced rather than typed.
    duration_revision: u32,
    /// Grams poured in the step; blank is no pour.
    water: Field<Option<WaterAmount>>,
}

impl DraftStep {
    /// Text a new row's duration field starts with.
    const NEW_DURATION_TEXT: &'static str = "0";

    /// A new row named `name`, `0` seconds long, pouring nothing until
    /// grams are typed.
    pub(crate) fn new(id: StepId, name: &str) -> Self {
        Self {
            id,
            name: name.to_owned(),
            duration: Field::parse(Self::NEW_DURATION_TEXT, StepDuration::try_parse_text),
            duration_revision: 0,
            water: Field::parse("", WaterAmount::try_parse_optional_text),
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
            duration_revision: 0,
            water: Field::parse(
                &step
                    .water()
                    .map_or_else(String::new, |water| water.grams().to_string()),
                WaterAmount::try_parse_optional_text,
            ),
        }
    }

    /// Gives the row a new key: a copied step must not share its source's.
    pub(crate) fn renew_id(&mut self, id: StepId) {
        self.id = id;
    }

    pub(crate) fn set_name(&mut self, raw: &str) {
        raw.clone_into(&mut self.name);
    }

    pub(crate) fn set_duration(&mut self, raw: &str) {
        self.duration = Field::parse(raw, StepDuration::try_parse_text);
    }

    /// Moves the duration by `delta` seconds from what its label shows
    /// (the lenient reading: text that is not a number counts as `0`, an
    /// out-of-range number as its bound), clamped into range.
    pub(crate) fn nudge_duration(&mut self, delta: i64) {
        self.replace_duration(self.live_duration().nudged(delta));
    }

    /// The grams as typed. A valid pour at a known `speed` also replaces
    /// the duration with the time the pour takes; blank or invalid grams,
    /// or no speed, leave the duration as it is.
    pub(crate) fn set_water(&mut self, raw: &str, speed: Option<PourSpeed>) {
        self.water = Field::parse(raw, WaterAmount::try_parse_optional_text);
        if let (Some(Some(water)), Some(speed)) = (self.water.value().copied(), speed) {
            self.replace_duration(speed.duration_of(water));
        }
    }

    /// Core, not the user, sets the duration: the text becomes `duration`
    /// in seconds, so it is valid again, and the revision moves on so the
    /// field shows the new text.
    fn replace_duration(&mut self, duration: StepDuration) {
        self.set_duration(&duration.seconds().to_string());
        self.duration_revision = self.duration_revision.wrapping_add(1);
    }

    /// The step to save at `position` (1-based); none while the duration or
    /// the grams are invalid. A blank name saves as `Step N`.
    pub(crate) fn to_step(&self, position: usize) -> Option<RecipeStep> {
        let duration = *self.duration.value()?;
        let water = *self.water.value()?;
        Some(RecipeStep::new(
            self.id.clone(),
            RecipeStep::label_or_default(&self.name, position),
            duration,
            water,
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

    pub(crate) fn duration_revision(&self) -> u32 {
        self.duration_revision
    }

    pub(crate) fn water(&self) -> &Field<Option<WaterAmount>> {
        &self.water
    }
}
