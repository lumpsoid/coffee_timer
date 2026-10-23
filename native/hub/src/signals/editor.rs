use rinf::{DartSignal, RustSignal, SignalPiece};
use serde::{Deserialize, Serialize};

/// The editor holder is subscribed: answer with the current `EditorUpdate`.
#[derive(Deserialize, DartSignal)]
pub struct EditorReady;

/// Open a draft for a new recipe.
#[derive(Deserialize, DartSignal)]
pub struct NewDraft;

/// Open a draft of the stored recipe with this id.
#[derive(Deserialize, DartSignal)]
pub struct EditRecipe {
    pub recipe_id: String,
}

/// Open a draft of a new recipe copied from the stored recipe with this
/// id: `<name> copy`, fresh ids, nothing saved until `SaveRecipe`.
#[derive(Deserialize, DartSignal)]
pub struct DuplicateRecipe {
    pub recipe_id: String,
}

#[derive(Deserialize, DartSignal)]
pub struct SetRecipeName {
    pub raw: String,
}

#[derive(Deserialize, DartSignal)]
pub struct SetRecipeDescription {
    pub raw: String,
}

/// The coffee grams as typed.
#[derive(Deserialize, DartSignal)]
pub struct SetRecipeCoffee {
    pub raw: String,
}

/// The water grams as typed.
#[derive(Deserialize, DartSignal)]
pub struct SetRecipeWater {
    pub raw: String,
}

/// The pour speed in g/s as typed (`5`, `4.5`).
#[derive(Deserialize, DartSignal)]
pub struct SetRecipePourSpeed {
    pub raw: String,
}

/// `row` is an `EditorStep::id`.
#[derive(Deserialize, DartSignal)]
pub struct SetStepName {
    pub row: String,
    pub raw: String,
}

/// `row` is an `EditorStep::id`.
#[derive(Deserialize, DartSignal)]
pub struct SetStepDuration {
    pub row: String,
    pub raw: String,
}

/// A step card's quick duration button was pressed. `row` is an
/// `EditorStep::id`.
#[derive(Deserialize, DartSignal)]
pub struct NudgeStepDuration {
    pub row: String,
    pub nudge: DurationNudgeChoice,
}

/// The grams the step pours, as typed; blank is no pour. Valid grams also
/// set the step's duration to the time the pour takes at the recipe's pour
/// speed. `row` is an `EditorStep::id`.
#[derive(Deserialize, DartSignal)]
pub struct SetStepWater {
    pub row: String,
    pub raw: String,
}

#[derive(Deserialize, DartSignal)]
pub struct AddStep {
    pub preset: StepPresetChoice,
}

/// `row` is an `EditorStep::id`.
#[derive(Deserialize, DartSignal)]
pub struct RemoveStep {
    pub row: String,
}

/// Move the row to position `to` (0-based, counted after the move: the
/// index it will have). `row` is an `EditorStep::id`.
#[derive(Deserialize, DartSignal)]
pub struct MoveStep {
    pub row: String,
    pub to: u32,
}

#[derive(Deserialize, DartSignal)]
pub struct SaveRecipe;

#[derive(Deserialize, DartSignal)]
pub struct DeleteRecipe;

/// Leave the editor without saving.
#[derive(Deserialize, DartSignal)]
pub struct CloseEditor;

/// The whole editor page. Dart replaces its state with each one; `Closed`
/// after a save or delete means the page is done.
#[derive(Serialize, RustSignal)]
pub enum EditorUpdate {
    Closed,
    Open { draft: Box<EditorDraft> },
}

/// The recipe being edited, as typed.
#[derive(Serialize, SignalPiece)]
pub struct EditorDraft {
    pub id: String,
    /// `New Recipe` or `Edit Recipe`.
    pub title: String,
    /// Offer delete only when true.
    pub can_delete: bool,
    pub name: String,
    /// `Enter a recipe name`, once a save was attempted.
    pub name_error: Option<String>,
    pub description: String,
    /// The coffee grams as typed.
    pub coffee: String,
    /// `Enter grams` or `1–200 g`, once a save was attempted.
    pub coffee_error: Option<String>,
    /// The water grams as typed.
    pub water: String,
    /// `Enter grams` or `1–2000 g`, once a save was attempted.
    pub water_error: Option<String>,
    /// The pour speed in g/s as typed.
    pub pour_speed: String,
    /// `Enter g/s` or `0.5–30 g/s`, once a save was attempted.
    pub pour_speed_error: Option<String>,
    /// Live total, `M:SS`.
    pub total_label: String,
    /// `Step pours add up to 200 g of 250 g` when the typed step pours miss
    /// the recipe's water; a hint, not an error.
    pub pour_hint: Option<String>,
    pub steps: Vec<EditorStep>,
    /// The add-step buttons, in order.
    pub add_step_buttons: Vec<AddStepButton>,
    /// The quick duration buttons every step card shows, in order.
    pub duration_nudge_buttons: Vec<DurationNudgeButton>,
    /// A store failure on save or delete (`Failed to save recipe: …`), for a
    /// snackbar. Set only by the action that failed; every update after the
    /// next action has none.
    pub notice: Option<String>,
}

/// One step card.
#[derive(Serialize, SignalPiece)]
pub struct EditorStep {
    /// Row key: send it back in `SetStepName`, `SetStepDuration`,
    /// `NudgeStepDuration`, `SetStepWater`, `RemoveStep`, `MoveStep`.
    pub id: String,
    /// `Step N name`.
    pub name_label: String,
    pub name: String,
    pub duration_text: String,
    /// The typed duration as `MM:SS`.
    pub duration_label: String,
    /// `Enter seconds` or `0–5999 seconds`, once a save was attempted.
    pub duration_error: Option<String>,
    /// Moves on when Rust replaced `duration_text` (a nudge, or a pour
    /// timed at the pour speed), never while
    /// it is typed: key the duration field by it so it reloads the text.
    pub duration_revision: u32,
    /// The grams poured as typed; blank is no pour.
    pub water_text: String,
    /// `Enter grams` or `1–2000 g`, once a save was attempted.
    pub water_error: Option<String>,
}

#[derive(Serialize, SignalPiece)]
pub struct AddStepButton {
    pub preset: StepPresetChoice,
    pub label: String,
}

/// Which add-step button was pressed.
#[derive(Serialize, Deserialize, SignalPiece, Clone, Copy)]
pub enum StepPresetChoice {
    AddCoffee,
    AddWater,
    Wait,
    Blank,
}

/// One quick duration button: `−15`, `−5`, `+5`, `+15`.
#[derive(Serialize, SignalPiece)]
pub struct DurationNudgeButton {
    pub nudge: DurationNudgeChoice,
    pub label: String,
    /// For screen readers and the tooltip, `Add 5 seconds`.
    pub description: String,
}

/// Which quick duration button was pressed.
#[derive(Serialize, Deserialize, SignalPiece, Clone, Copy)]
pub enum DurationNudgeChoice {
    MinusLarge,
    MinusSmall,
    PlusSmall,
    PlusLarge,
}
