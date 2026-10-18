use rinf::{DartSignal, RustSignal, SignalPiece};
use serde::{Deserialize, Serialize};

/// The recipe list holder is subscribed: answer with a `Snapshot`.
#[derive(Deserialize, DartSignal)]
pub struct RecipeListReady;

/// Delete the listed recipe with this id (the list's long-press delete).
#[derive(Deserialize, DartSignal)]
pub struct RemoveRecipe {
    pub recipe_id: String,
}

/// Undo the removal of the recipe with this id (the Undo on the `Deleted …`
/// snackbar). Only the recipe removed last can come back.
#[derive(Deserialize, DartSignal)]
pub struct UndoRemoveRecipe {
    pub recipe_id: String,
}

/// The recipe list: a snapshot, then one diff per persisted catalog change.
/// One type, so the snapshot and the diffs after it arrive in order.
#[derive(Serialize, RustSignal)]
pub enum RecipeListUpdate {
    /// Every recipe, in catalog order. Replaces the list.
    Snapshot { recipes: Vec<RecipeSummary> },
    /// Replace the row with this id, or insert it at `index` (its 0-based
    /// place in catalog order: the end for a new recipe, its old place for
    /// a restored one).
    Upserted { recipe: RecipeSummary, index: u32 },
    /// Drop the row with this id. `undo_notice`, when present, offers to
    /// undo the removal: the snackbar text (`Deleted V60`), shown once with
    /// an Undo that sends `UndoRemoveRecipe`.
    Removed {
        id: String,
        undo_notice: Option<String>,
    },
    /// A `RemoveRecipe`, `UndoRemoveRecipe` or `SaveBrewRatio` failed; the
    /// list is unchanged. A one-shot notice for a snackbar (`Failed to
    /// delete recipe: …`, `Nothing to undo`, `Can't save this ratio: …`),
    /// not list state.
    Failed { message: String },
}

/// One row of the recipe list. Labels are values only (`20 g`, `100 g`,
/// `1:35`); the captions (`Coffee`, …) are UI text.
#[derive(Serialize, SignalPiece)]
pub struct RecipeSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub coffee_label: String,
    pub water_label: String,
    pub time_label: String,
    pub label: RecipeLabel,
}

/// How a row's card looks, as a printed coffee-bag label. Picked in Rust;
/// Dart maps each choice to colours and paint.
#[derive(Serialize, SignalPiece)]
pub struct RecipeLabel {
    pub tone: LabelTone,
    pub texture: LabelTexture,
    pub shade: LabelShade,
    pub stain: StainCorner,
}

/// Colour family, from total brew time.
#[derive(Serialize, SignalPiece)]
pub enum LabelTone {
    Roast,
    Caramel,
    Crema,
}

#[derive(Serialize, SignalPiece)]
pub enum LabelTexture {
    Kraft,
    Linen,
    Halftone,
    Pinstripe,
    Contour,
    Burlap,
}

/// A variation within the tone.
#[derive(Serialize, SignalPiece)]
pub enum LabelShade {
    Base,
    Darker,
    Lighter,
}

/// Where the coffee-ring stain sits.
#[derive(Serialize, SignalPiece)]
pub enum StainCorner {
    BottomRight,
    TopLeft,
}
