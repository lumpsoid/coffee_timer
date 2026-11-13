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

/// Pin the listed recipe with this id to the top of the list (the list's
/// long-press Pin).
#[derive(Deserialize, DartSignal)]
pub struct PinRecipe {
    pub recipe_id: String,
}

/// Unpin the listed recipe with this id (the list's long-press Unpin).
#[derive(Deserialize, DartSignal)]
pub struct UnpinRecipe {
    pub recipe_id: String,
}

/// Move the listed recipe with this id to `to`, its 0-based place in the
/// list counted after the move (the list's reorder mode). A recipe moves
/// only among those that share its pin; Rust answers with an `Upserted` at
/// the place it took.
#[derive(Deserialize, DartSignal)]
pub struct MoveRecipe {
    pub recipe_id: String,
    pub to: u32,
}

/// Export recipes as a JSON recipe document, to the clipboard: the ones
/// with these ids, in this order (the long-press sheet's Export), or every
/// recipe, in list order, when empty (the app bar's Export all). Rust
/// answers with `PlatformEffect::CopyToClipboard` and a
/// `RecipeListUpdate::Notice` (`Copied 2 recipes`), or a `Failed`.
#[derive(Deserialize, DartSignal)]
pub struct ExportRecipes {
    pub recipe_ids: Vec<String>,
}

/// Import the recipe document `text`, the clipboard's text as read (empty
/// when it holds none). Each new recipe arrives as an `Upserted` at the end
/// of the list, then a `Notice` (`Imported 2 recipes`); an invalid document
/// adds none and is a `Failed`.
#[derive(Deserialize, DartSignal)]
pub struct ImportRecipes {
    pub text: String,
}

/// The recipe list: a snapshot, then one diff per persisted catalog change.
/// One type, so the snapshot and the diffs after it arrive in order.
#[derive(Serialize, RustSignal)]
pub enum RecipeListUpdate {
    /// Every recipe, in list order (pinned first). Replaces the list.
    Snapshot { recipes: Vec<RecipeSummary> },
    /// The row with this id now reads `recipe` and stands at `index`, its
    /// 0-based place in list order: drop the row with this id if there is
    /// one, then insert `recipe` at `index`. That one rule covers an edit
    /// (same place), a new recipe (the end), a restored one (its old place),
    /// a pin or unpin and a `MoveRecipe` (a move).
    Upserted { recipe: RecipeSummary, index: u32 },
    /// Drop the row with this id. `undo_notice`, when present, offers to
    /// undo the removal: the snackbar text (`Deleted V60`), shown once with
    /// an Undo that sends `UndoRemoveRecipe`.
    Removed {
        id: String,
        undo_notice: Option<String>,
    },
    /// A `RemoveRecipe`, `UndoRemoveRecipe`, `PinRecipe`, `UnpinRecipe`,
    /// `MoveRecipe`, `SaveBrewRatio`, `ExportRecipes` or `ImportRecipes`
    /// failed; the list is unchanged. A one-shot notice for a snackbar
    /// (`Failed to delete recipe: …`, `Nothing to undo`, `Failed to save
    /// the pin: …`, `Failed to save the order: …`, `Can't save this ratio:
    /// …`, `Can't import: …`), not list state.
    Failed { message: String },
    /// An `ExportRecipes` or `ImportRecipes` succeeded: a one-shot notice
    /// for a snackbar (`Copied 2 recipes`, `Imported 1 recipe`), not list
    /// state.
    Notice { message: String },
}

/// One row of the recipe list. Labels are values only (`20 g`, `100 g`,
/// `1:35`, `22`); the captions (`Coffee`, …, `Clicks`) are UI text.
/// `grind_label` is none for a recipe without a grind. `pinned` shows the pin
/// marker and picks the long-press sheet's Pin or Unpin.
#[derive(Serialize, SignalPiece)]
pub struct RecipeSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub coffee_label: String,
    pub water_label: String,
    pub time_label: String,
    pub grind_label: Option<String>,
    pub label: RecipeLabel,
    pub pinned: bool,
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

#[derive(Serialize, Deserialize, SignalPiece, Clone, Copy)]
pub enum LabelTexture {
    Kraft,
    Linen,
    Halftone,
    Pinstripe,
    Contour,
    Burlap,
    Plain,
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
