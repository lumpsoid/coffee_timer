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

/// Stop the running or paused brew; the recipe stays open.
#[derive(Deserialize, DartSignal)]
pub struct ResetBrew;

/// Hold the running brew on its step, with the time it has left.
#[derive(Deserialize, DartSignal)]
pub struct PauseBrew;

/// Count the paused brew down again from where it was held.
#[derive(Deserialize, DartSignal)]
pub struct ResumeBrew;

/// Change the open recipe's ratio for this brew only; the coffee stays and
/// the water follows.
#[derive(Deserialize, DartSignal)]
pub struct AdjustBrewRatio {
    pub change: BrewRatioChange,
}

#[derive(Deserialize, SignalPiece)]
pub enum BrewRatioChange {
    /// One step more water per gram of coffee.
    MoreWater,
    /// One step less water per gram of coffee.
    LessWater,
    /// Back to the recipe's own ratio.
    Recipe,
}

/// Make the brew's adjusted ratio the open recipe's own: save the water
/// and step pours shown. A refusal (too much water, a pour out of range) or
/// a store failure comes back as `RecipeListUpdate::Failed`.
#[derive(Deserialize, DartSignal)]
pub struct SaveBrewRatio;

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
    /// The timer is held on `timer.step_index` with `timer.remaining_label`
    /// left; that step's status is `Paused`.
    Paused {
        recipe: BrewRecipe,
        timer: BrewTimer,
    },
    /// The last step ran out; every step's status is `Done`. Stopped like
    /// `Idle`: a `StartBrew` brews again, a `ResetBrew` goes back to idle.
    Finished {
        recipe: BrewRecipe,
        finished: BrewFinished,
    },
}

/// How the last run ended.
#[derive(Serialize, SignalPiece)]
pub struct BrewFinished {
    /// How long the run brewed, `M:SS`.
    pub total_label: String,
    /// What the scale should read now, `Poured to 300g`; none when no step
    /// pours.
    pub poured_label: Option<String>,
}

/// The open recipe with each step's status.
#[derive(Serialize, SignalPiece)]
pub struct BrewRecipe {
    pub id: String,
    pub name: String,
    pub description: String,
    /// `20g`.
    pub coffee_label: String,
    /// `100g`, at the ratio shown.
    pub water_label: String,
    /// Water per gram of coffee, `1:5`.
    pub ratio_label: String,
    pub ratio: BrewRatioSetting,
    /// Where to set the grinder, in clicks, `22`; none when the recipe does
    /// not say.
    pub grind_label: Option<String>,
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
    /// The water this step pours at the water shown, `50g`; none for a step
    /// that pours nothing.
    pub pour_label: Option<String>,
    pub status: BrewStepStatus,
    /// The step's arc on the dial's step track, as fractions of the whole
    /// brew: steps lie end to end from 0.0 to 1.0.
    pub track_start: f64,
    pub track_end: f64,
}

/// Whose ratio the page shows.
#[derive(Serialize, SignalPiece)]
pub enum BrewRatioSetting {
    /// The recipe's own ratio.
    Recipe,
    /// Changed for this brew; water and ratio differ from the recipe's.
    Adjusted,
}

#[derive(Serialize, SignalPiece)]
pub enum BrewStepStatus {
    Done,
    /// Being counted down.
    Current,
    /// The step a paused brew is held on.
    Paused,
    Pending,
}

/// The step being counted down, or held while paused.
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
    /// What the scale should read once this step has poured, `Pour to
    /// 150g`; none for a step that pours nothing.
    pub pour_to_label: Option<String>,
    /// What comes next, shown under the dial: `Next: Pour 2 · 0:45 ·
    /// 100g` (the pour only if that step pours), or `Last step`.
    pub next_label: String,
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
    /// Alternating pause and buzz lengths in ms, starting with a pause.
    VibratePattern {
        pattern_ms: Vec<u32>,
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
    /// Put `text` on the clipboard: an exported recipe document.
    CopyToClipboard {
        text: String,
    },
}
