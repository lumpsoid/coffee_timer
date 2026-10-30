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
    AdjustBrewRatio, BrewFinished, BrewRatioChange, BrewRatioSetting, BrewReady, BrewRecipe,
    BrewStep, BrewStepStatus, BrewTimer, BrewUpdate, LeaveBrew, OpenBrew, PauseBrew,
    PlatformEffect, ResetBrew, ResumeBrew, SaveBrewRatio, StartBrew,
};
pub use editor::{
    AddStep, AddStepButton, CloseEditor, DeleteRecipe, DuplicateRecipe, DurationNudgeButton,
    DurationNudgeChoice, EditRecipe, EditorDraft, EditorReady, EditorStep, EditorUpdate, MoveStep,
    NewDraft, NudgeStepDuration, RemoveStep, SaveRecipe, SetRecipeCoffee, SetRecipeDescription,
    SetRecipeGrind, SetRecipeName, SetRecipePourSpeed, SetRecipeWater, SetStepDuration,
    SetStepName, SetStepWater, StepPresetChoice,
};
pub use recipe_list::{
    ExportRecipes, ImportRecipes, LabelShade, LabelTexture, LabelTone, MoveRecipe, PinRecipe,
    RecipeLabel, RecipeListReady, RecipeListUpdate, RecipeSummary, RemoveRecipe, StainCorner,
    UndoRemoveRecipe, UnpinRecipe,
};
pub use settings::{
    AccentChoice, PreAlertChoice, SetAccent, SetPreAlert, SetSound, SetTheme, SetVibration,
    SettingsReady, SettingsUpdate, ThemeChoice,
};
