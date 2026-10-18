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

/// The grams the step pours, as typed; blank is no pour. `row` is an
/// `EditorStep::id`.
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
    /// Live total, `M:SS`.
    pub total_label: String,
    /// `Step pours add up to 200 g of 250 g` when the typed step pours miss
    /// the recipe's water; a hint, not an error.
    pub pour_hint: Option<String>,
    pub steps: Vec<EditorStep>,
    /// The add-step buttons, in order.
    pub add_step_buttons: Vec<AddStepButton>,
    /// A store failure on save or delete (`Failed to save recipe: …`), for a
    /// snackbar. Set only by the action that failed; every update after the
    /// next action has none.
    pub notice: Option<String>,
}

/// One step card.
#[derive(Serialize, SignalPiece)]
pub struct EditorStep {
    /// Row key: send it back in `SetStepName`, `SetStepDuration`,
    /// `SetStepWater`, `RemoveStep`.
    pub id: String,
    /// `Step N name`.
    pub name_label: String,
    pub name: String,
    pub duration_text: String,
    /// The typed duration as `MM:SS`.
    pub duration_label: String,
    /// `Enter seconds` or `0–5999 seconds`, once a save was attempted.
    pub duration_error: Option<String>,
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
