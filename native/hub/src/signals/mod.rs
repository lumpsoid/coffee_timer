//! Signal types shared with Dart, one file per feature.
//! Run `rinf gen` after changing anything here.
//!
//! `rinf gen` reads every signal type in the crate into one flat namespace,
//! so names are unique across files and avoid Flutter's own (`Theme`).

mod boot;
mod brew;
mod editor;
mod recipe_list;
mod settings;

pub use boot::{Boot, BootFailed};
pub use brew::{
    BrewReady, BrewRecipe, BrewStep, BrewStepStatus, BrewTimer, BrewUpdate, LeaveBrew, OpenBrew,
    PlatformEffect, ResetBrew, StartBrew,
};
pub use editor::{
    AddStep, AddStepButton, CloseEditor, DeleteRecipe, EditRecipe, EditorDraft, EditorReady,
    EditorStep, EditorUpdate, NewDraft, RemoveStep, SaveRecipe, SetRecipeDescription,
    SetRecipeName, SetStepDuration, SetStepName, StepPresetChoice,
};
pub use recipe_list::{RecipeListReady, RecipeListUpdate, RecipeSummary, RemoveRecipe};
pub use settings::{AccentChoice, SetAccent, SetTheme, SettingsReady, SettingsUpdate, ThemeChoice};
