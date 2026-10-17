use rinf::{DartSignal, RustSignal, SignalPiece};
use serde::{Deserialize, Serialize};

use crate::signals::RecipeLabel;

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
    Running {
        recipe: BrewRecipe,
        timer: BrewTimer,
    },
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
    /// Water per gram of coffee, `1:5`.
    pub ratio_label: String,
    /// Total, `M:SS`.
    pub duration_label: String,
    /// The page wears the recipe's label, the same one the list shows.
    pub label: RecipeLabel,
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
    /// The step's arc on the dial's step track, as fractions of the whole
    /// brew: steps lie end to end from 0.0 to 1.0.
    pub track_start: f64,
    pub track_end: f64,
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
    /// Start number of this brew: grows with every accepted `StartBrew`,
    /// kept by ticks and step changes. Saturates at `u32::MAX`.
    pub run: u32,
    /// 0-based; changes exactly when the step changes. The ring animation
    /// restarts whenever `(run, step_index)` changes.
    pub step_index: u32,
    pub step_name: String,
    /// `Step 1 of 3`.
    pub position_label: String,
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
    Vibrate {
        ms: u32,
    },
    KeepScreenOn {
        on: bool,
    },
    /// Show or refresh the ongoing notification of a running brew; on
    /// Android it keeps the brew ticking in the background.
    ShowBrewNotification {
        /// `Bloom · 00:35`.
        title: String,
        /// `Next: Pour`; none on the last step.
        next: Option<String>,
    },
    HideBrewNotification,
}
