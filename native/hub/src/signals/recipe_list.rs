use rinf::{DartSignal, RustSignal, SignalPiece};
use serde::{Deserialize, Serialize};

/// The recipe list holder is subscribed: answer with a `Snapshot`.
#[derive(Deserialize, DartSignal)]
pub struct RecipeListReady;

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
}

/// One row of the recipe list. Labels are values only (`20 g`, `100 g`,
/// `1:35`); the captions (`Coffee: `, …) are UI text.
#[derive(Serialize, SignalPiece)]
pub struct RecipeSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub coffee_label: String,
    pub water_label: String,
    pub time_label: String,
}
