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

/// The recipe list: a snapshot, then one diff per persisted catalog change.
/// One type, so the snapshot and the diffs after it arrive in order.
#[derive(Serialize, RustSignal)]
pub enum RecipeListUpdate {
    /// Every recipe, in catalog order. Replaces the list.
    Snapshot { recipes: Vec<RecipeSummary> },
    /// Replace the row with this id, or append it.
    Upserted { recipe: RecipeSummary },
    /// Drop the row with this id.
    Removed { id: String },
    /// A `RemoveRecipe` failed; the list is unchanged. A one-shot notice for
    /// a snackbar (`Failed to delete recipe: …`), not list state.
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
