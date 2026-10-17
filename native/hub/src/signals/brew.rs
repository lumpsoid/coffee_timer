use rinf::{DartSignal, RustSignal, SignalPiece};
use serde::{Deserialize, Serialize};

/// The brew holder is subscribed: answer with the current `BrewUpdate`.
#[derive(Deserialize, DartSignal)]
pub struct BrewReady;

/// Show the recipe with this id, idle. An unknown id shows "Empty recipe".
#[derive(Deserialize, DartSignal)]
pub struct OpenBrew {
    pub recipe_id: String,
}

/// Run the open recipe from step `from` (0 for the whole recipe).
#[derive(Deserialize, DartSignal)]
pub struct StartBrew {
    pub from: u32,
}

/// Stop the running brew; the recipe stays open.
#[derive(Deserialize, DartSignal)]
pub struct ResetBrew;

/// The brew page closed.
#[derive(Deserialize, DartSignal)]
pub struct LeaveBrew;

/// The whole brew page. Dart replaces its state with each one.
#[derive(Serialize, RustSignal)]
pub enum BrewUpdate {
    /// No recipe to show; the page reads `label` (`Empty recipe`).
    Empty { label: String },
    /// A recipe is open and its timer is stopped.
    Idle { recipe: BrewRecipe },
    /// The timer is counting down `timer.step_index`.
    Running { recipe: BrewRecipe, timer: BrewTimer },
}

/// The open recipe with each step's status.
#[derive(Serialize, SignalPiece)]
pub struct BrewRecipe {
    pub id: String,
    pub name: String,
    pub description: String,
    /// `20g`.
    pub coffee_label: String,
    /// `100g`.
    pub water_label: String,
    /// Total, `M:SS`.
    pub duration_label: String,
    pub steps: Vec<BrewStep>,
}

/// One line of the step list.
#[derive(Serialize, SignalPiece)]
pub struct BrewStep {
    /// 1-based position, shown while pending.
    pub number_label: String,
    pub name: String,
    /// `Ns`.
    pub duration_label: String,
    pub status: BrewStepStatus,
}

#[derive(Serialize, SignalPiece)]
pub enum BrewStepStatus {
    Done,
    Current,
    Pending,
}

/// The step being counted down.
#[derive(Serialize, SignalPiece)]
pub struct BrewTimer {
    /// 0-based; changes exactly when the step changes, so the ring
    /// animation is keyed on it.
    pub step_index: u32,
    pub step_name: String,
    /// `MM:SS` left.
    pub remaining_label: String,
    /// Period of the ring's repeating sweep, in seconds: the step's full
    /// duration. None for a 0-second step (the ring stays still).
    pub sweep_seconds: Option<u32>,
}

/// A one-shot platform command, already decided in Rust. Dart performs it
/// and keeps nothing.
#[derive(Serialize, RustSignal)]
pub enum PlatformEffect {
    PlayStepSound,
    Vibrate { ms: u32 },
    KeepScreenOn { on: bool },
}
