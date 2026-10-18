//! The hub task: owns every controller, turns each Dart signal into one
//! controller action, and sends the feature's state after it. One task, so
//! actions run one at a time. It decides nothing: errors are logged and the
//! state re-sent, except a failed list delete, undo, pin, ratio save,
//! export or import, which goes to Dart as the one-shot
//! `RecipeListUpdate::Failed`; every `CatalogChange` is passed on
//! unconditionally.

use std::error::Error;
use std::fmt::Display;
use std::time::Duration;

use coffee_timer_core::brew::{BrewController, Effect};
use coffee_timer_core::catalog::{CatalogChange, RecipeCatalog};
use coffee_timer_core::editor::{DeleteError, EditorController, SaveError};
use coffee_timer_core::model::{self, AccentColor, ModelError, PreAlertLead, RecipeId, StepId};
use coffee_timer_core::ports::StoreError;
use coffee_timer_core::settings::SettingsController;
use coffee_timer_store::{SqliteStore, UuidIds};
use rinf::{DartSignal, RustSignal, debug_print};
use tokio::time::{Instant, Interval, MissedTickBehavior, interval_at};

use crate::signals::{
    AddStep, AdjustBrewRatio, BrewReady, BrewUpdate, CloseEditor, DeleteRecipe, DuplicateRecipe,
    EditRecipe, EditorReady, EditorUpdate, ExportRecipes, ImportRecipes, LeaveBrew, MoveStep,
    NewDraft, OpenBrew, PauseBrew, PinRecipe, PlatformEffect, RecipeListReady, RecipeListUpdate,
    RemoveRecipe, RemoveStep, ResetBrew, ResumeBrew, SaveBrewRatio, SaveRecipe, SetAccent,
    SetPreAlert, SetRecipeCoffee, SetRecipeDescription, SetRecipeName, SetRecipeWater, SetSound,
    SetStepDuration, SetStepName, SetStepWater, SetTheme, SetVibration, SettingsReady,
    SettingsUpdate, StartBrew, UndoRemoveRecipe, UnpinRecipe,
};

const ONE_SECOND: Duration = Duration::from_secs(1);

/// Every controller, built and loaded from one store.
pub struct Controllers {
    catalog: RecipeCatalog,
    settings: SettingsController,
    brew: BrewController,
    editor: EditorController,
}

impl Controllers {
    /// Builds each controller on `store` (UUID v7 ids for the editor), then
    /// loads the catalog and settings.
    pub fn boot(store: &SqliteStore) -> Result<Self, StoreError> {
        let mut catalog = RecipeCatalog::new(Box::new(store.clone()));
        catalog.load()?;
        let mut settings = SettingsController::new(Box::new(store.clone()));
        settings.load()?;
        Ok(Self {
            catalog,
            settings,
            brew: BrewController::new(),
            editor: EditorController::new(Box::new(UuidIds)),
        })
    }
}

/// Runs until the runtime shuts down. Holds the only receiver of every Dart
/// signal but `Boot`; signals sent before it starts are queued, not lost.
pub async fn run(mut c: Controllers) {
    let recipe_list_ready = RecipeListReady::get_dart_signal_receiver();
    let remove_recipe = RemoveRecipe::get_dart_signal_receiver();
    let undo_remove_recipe = UndoRemoveRecipe::get_dart_signal_receiver();
    let pin_recipe = PinRecipe::get_dart_signal_receiver();
    let unpin_recipe = UnpinRecipe::get_dart_signal_receiver();
    let export_recipes = ExportRecipes::get_dart_signal_receiver();
    let import_recipes = ImportRecipes::get_dart_signal_receiver();
    let brew_ready = BrewReady::get_dart_signal_receiver();
    let open_brew = OpenBrew::get_dart_signal_receiver();
    let start_brew = StartBrew::get_dart_signal_receiver();
    let reset_brew = ResetBrew::get_dart_signal_receiver();
    let pause_brew = PauseBrew::get_dart_signal_receiver();
    let resume_brew = ResumeBrew::get_dart_signal_receiver();
    let leave_brew = LeaveBrew::get_dart_signal_receiver();
    let adjust_brew_ratio = AdjustBrewRatio::get_dart_signal_receiver();
    let save_brew_ratio = SaveBrewRatio::get_dart_signal_receiver();
    let editor_ready = EditorReady::get_dart_signal_receiver();
    let new_draft = NewDraft::get_dart_signal_receiver();
    let edit_recipe = EditRecipe::get_dart_signal_receiver();
    let duplicate_recipe = DuplicateRecipe::get_dart_signal_receiver();
    let set_recipe_name = SetRecipeName::get_dart_signal_receiver();
    let set_recipe_description = SetRecipeDescription::get_dart_signal_receiver();
    let set_recipe_coffee = SetRecipeCoffee::get_dart_signal_receiver();
    let set_recipe_water = SetRecipeWater::get_dart_signal_receiver();
    let set_step_name = SetStepName::get_dart_signal_receiver();
    let set_step_duration = SetStepDuration::get_dart_signal_receiver();
    let set_step_water = SetStepWater::get_dart_signal_receiver();
    let add_step = AddStep::get_dart_signal_receiver();
    let remove_step = RemoveStep::get_dart_signal_receiver();
    let move_step = MoveStep::get_dart_signal_receiver();
    let save_recipe = SaveRecipe::get_dart_signal_receiver();
    let delete_recipe = DeleteRecipe::get_dart_signal_receiver();
    let close_editor = CloseEditor::get_dart_signal_receiver();
    let settings_ready = SettingsReady::get_dart_signal_receiver();
    let set_theme = SetTheme::get_dart_signal_receiver();
    let set_accent = SetAccent::get_dart_signal_receiver();
    let set_sound = SetSound::get_dart_signal_receiver();
    let set_vibration = SetVibration::get_dart_signal_receiver();
    let set_pre_alert = SetPreAlert::get_dart_signal_receiver();

    // The brew's one-second beat: exists exactly while a brew runs.
    let mut ticker: Option<Interval> = None;

    loop {
        tokio::select! {
            // Recipe list
            Some(_) = recipe_list_ready.recv() => {
                RecipeListUpdate::snapshot(&c.catalog).send_signal_to_dart();
            }
            Some(pack) = remove_recipe.recv() => {
                report(remove(&mut c, &pack.message.recipe_id));
            }
            Some(pack) = undo_remove_recipe.recv() => {
                report(restore(&mut c, &pack.message.recipe_id));
            }
            // `model::Pin`: `select!` brings `std::pin::Pin` into scope.
            Some(pack) = pin_recipe.recv() => {
                report(set_pin(&mut c, &pack.message.recipe_id, model::Pin::Pinned));
            }
            Some(pack) = unpin_recipe.recv() => {
                report(set_pin(&mut c, &pack.message.recipe_id, model::Pin::Unpinned));
            }
            Some(pack) = export_recipes.recv() => {
                report(export(&c.catalog, &pack.message.recipe_ids));
            }
            Some(pack) = import_recipes.recv() => import(&mut c, &pack.message.text),

            // Brew
            Some(_) = brew_ready.recv() => show_brew(&c.brew, &[]),
            Some(pack) = open_brew.recv() => {
                let effects = report(open(&mut c.brew, &c.catalog, &pack.message.recipe_id));
                show_brew(&c.brew, &effects.unwrap_or_default());
            }
            Some(pack) = start_brew.recv() => {
                let from = usize::try_from(pack.message.from).unwrap_or(usize::MAX);
                let effects = report(c.brew.start(from));
                if effects.is_some() {
                    ticker = Some(every_second());
                }
                show_brew(&c.brew, &effects.unwrap_or_default());
            }
            () = next_tick(&mut ticker) => {
                // The settings as they are now: a change mid-brew applies
                // from this tick.
                let effects = c.brew.tick(c.settings.state());
                show_brew(&c.brew, &effects);
            }
            Some(_) = reset_brew.recv() => {
                let effects = c.brew.reset();
                show_brew(&c.brew, &effects);
            }
            // Pausing needs no ticker work: it is dropped below once the
            // brew no longer runs.
            Some(_) = pause_brew.recv() => {
                let effects = report(c.brew.pause());
                show_brew(&c.brew, &effects.unwrap_or_default());
            }
            Some(_) = resume_brew.recv() => {
                let effects = report(c.brew.resume());
                if effects.is_some() {
                    ticker = Some(every_second());
                }
                show_brew(&c.brew, &effects.unwrap_or_default());
            }
            Some(pack) = adjust_brew_ratio.recv() => {
                report(c.brew.adjust_ratio(pack.message.change.into()));
                show_brew(&c.brew, &[]);
            }
            // `pass_on` sends the brew view, or it is unchanged.
            Some(_) = save_brew_ratio.recv() => save_ratio(&mut c),
            Some(_) = leave_brew.recv() => {
                let effects = c.brew.leave();
                show_brew(&c.brew, &effects);
            }

            // Editor
            Some(_) = editor_ready.recv() => show_editor(&c.editor),
            Some(_) = new_draft.recv() => {
                c.editor.new_draft();
                show_editor(&c.editor);
            }
            Some(pack) = edit_recipe.recv() => {
                report(edit(&mut c.editor, &c.catalog, &pack.message.recipe_id));
                show_editor(&c.editor);
            }
            Some(pack) = duplicate_recipe.recv() => {
                report(duplicate(&mut c.editor, &c.catalog, &pack.message.recipe_id));
                show_editor(&c.editor);
            }
            Some(pack) = set_recipe_name.recv() => {
                report(c.editor.set_name(&pack.message.raw));
                show_editor(&c.editor);
            }
            Some(pack) = set_recipe_description.recv() => {
                report(c.editor.set_description(&pack.message.raw));
                show_editor(&c.editor);
            }
            Some(pack) = set_recipe_coffee.recv() => {
                report(c.editor.set_coffee(&pack.message.raw));
                show_editor(&c.editor);
            }
            Some(pack) = set_recipe_water.recv() => {
                report(c.editor.set_water(&pack.message.raw));
                show_editor(&c.editor);
            }
            Some(pack) = set_step_name.recv() => {
                let SetStepName { row, raw } = pack.message;
                report(on_row(&row, |row| c.editor.set_step_name(row, &raw)));
                show_editor(&c.editor);
            }
            Some(pack) = set_step_duration.recv() => {
                let SetStepDuration { row, raw } = pack.message;
                report(on_row(&row, |row| c.editor.set_step_duration(row, &raw)));
                show_editor(&c.editor);
            }
            Some(pack) = set_step_water.recv() => {
                let SetStepWater { row, raw } = pack.message;
                report(on_row(&row, |row| c.editor.set_step_water(row, &raw)));
                show_editor(&c.editor);
            }
            Some(pack) = add_step.recv() => {
                report(c.editor.add_step(pack.message.preset.into()));
                show_editor(&c.editor);
            }
            Some(pack) = remove_step.recv() => {
                report(on_row(&pack.message.row, |row| c.editor.remove_step(row)));
                show_editor(&c.editor);
            }
            Some(pack) = move_step.recv() => {
                let to = usize::try_from(pack.message.to).unwrap_or(usize::MAX);
                report(on_row(&pack.message.row, |row| c.editor.move_step(row, to)));
                show_editor(&c.editor);
            }
            Some(_) = save_recipe.recv() => {
                report(save(&mut c));
                show_editor(&c.editor);
            }
            Some(_) = delete_recipe.recv() => {
                report(delete(&mut c));
                show_editor(&c.editor);
            }
            Some(_) = close_editor.recv() => {
                c.editor.close();
                show_editor(&c.editor);
            }

            // Settings
            Some(_) = settings_ready.recv() => show_settings(&c.settings),
            Some(pack) = set_theme.recv() => {
                report(c.settings.set_theme(pack.message.theme.into()));
                show_settings(&c.settings);
            }
            Some(pack) = set_accent.recv() => {
                report(c.settings.set_accent(AccentColor::from_argb32(pack.message.argb)));
                show_settings(&c.settings);
            }
            Some(pack) = set_sound.recv() => {
                report(c.settings.set_sound(pack.message.on.into()));
                show_settings(&c.settings);
            }
            Some(pack) = set_vibration.recv() => {
                report(c.settings.set_vibration(pack.message.on.into()));
                show_settings(&c.settings);
            }
            Some(pack) = set_pre_alert.recv() => {
                report(set_pre_alert_seconds(&mut c.settings, pack.message.seconds));
                show_settings(&c.settings);
            }

            else => break,
        }
        if !c.brew.state().is_running() {
            ticker = None;
        }
    }
}

/// What every head does with a persisted change: send it as a list diff,
/// hand it to the brew view, send the brew view.
fn pass_on(change: &CatalogChange, brew: &mut BrewController, catalog: &RecipeCatalog) {
    RecipeListUpdate::of(change, catalog).send_signal_to_dart();
    let effects = brew.on_catalog_change(change, catalog);
    show_brew(brew, &effects);
}

fn save(c: &mut Controllers) -> Result<(), SaveError> {
    let change = c.editor.save(&mut c.catalog)?;
    pass_on(&change, &mut c.brew, &c.catalog);
    Ok(())
}

fn delete(c: &mut Controllers) -> Result<(), DeleteError> {
    let change = c.editor.delete(&mut c.catalog)?;
    pass_on(&change, &mut c.brew, &c.catalog);
    Ok(())
}

/// Saves the brew's adjusted ratio to its recipe. A refusal or a store
/// failure goes to the list as core's error, for a snackbar.
fn save_ratio(c: &mut Controllers) {
    match c.brew.save_ratio(&mut c.catalog) {
        Ok(change) => pass_on(&change, &mut c.brew, &c.catalog),
        Err(error) => RecipeListUpdate::from(error).send_signal_to_dart(),
    }
}

/// Deletes a listed recipe. A store failure goes to the list as core's
/// delete error, for a snackbar.
fn remove(c: &mut Controllers, recipe_id: &str) -> Result<(), ModelError> {
    match c.catalog.remove(RecipeId::try_parse(recipe_id)?) {
        Ok(change) => pass_on(&change, &mut c.brew, &c.catalog),
        Err(error) => RecipeListUpdate::from(DeleteError::Store(error)).send_signal_to_dart(),
    }
    Ok(())
}

/// Undoes the last removal. A refusal or a store failure goes to the list
/// as core's restore error, for a snackbar.
fn restore(c: &mut Controllers, recipe_id: &str) -> Result<(), ModelError> {
    match c.catalog.restore(&RecipeId::try_parse(recipe_id)?) {
        Ok(change) => pass_on(&change, &mut c.brew, &c.catalog),
        Err(error) => RecipeListUpdate::from(error).send_signal_to_dart(),
    }
    Ok(())
}

/// Pins or unpins a listed recipe. A refusal or a store failure goes to
/// the list as core's pin error, for a snackbar.
fn set_pin(c: &mut Controllers, recipe_id: &str, pin: model::Pin) -> Result<(), ModelError> {
    match c.catalog.set_pin(&RecipeId::try_parse(recipe_id)?, pin) {
        Ok(change) => pass_on(&change, &mut c.brew, &c.catalog),
        Err(error) => RecipeListUpdate::from(error).send_signal_to_dart(),
    }
    Ok(())
}

/// Copies the recipes `recipe_ids` names (every recipe, for none) to the
/// clipboard as a recipe document, with core's notice for a snackbar. An
/// unknown id goes to the list as core's export error, for a snackbar.
fn export(catalog: &RecipeCatalog, recipe_ids: &[String]) -> Result<(), ModelError> {
    let ids = recipe_ids
        .iter()
        .map(|id| RecipeId::try_parse(id))
        .collect::<Result<Vec<_>, _>>()?;
    match catalog.export(&ids) {
        Ok(export) => {
            PlatformEffect::CopyToClipboard {
                text: export.json().to_owned(),
            }
            .send_signal_to_dart();
            RecipeListUpdate::Notice {
                message: export.copied_notice(),
            }
            .send_signal_to_dart();
        }
        Err(error) => RecipeListUpdate::from(error).send_signal_to_dart(),
    }
    Ok(())
}

/// Imports the recipe document `text`: each new recipe is passed on like
/// any change, then core's notice goes to the list. A refused document or
/// a store failure goes to the list as core's import error.
fn import(c: &mut Controllers, text: &str) {
    match c.editor.import(text, &mut c.catalog) {
        Ok(import) => {
            for change in import.changes() {
                pass_on(change, &mut c.brew, &c.catalog);
            }
            RecipeListUpdate::Notice {
                message: import.notice(),
            }
            .send_signal_to_dart();
        }
        Err(error) => RecipeListUpdate::from(error).send_signal_to_dart(),
    }
}

fn open(
    brew: &mut BrewController,
    catalog: &RecipeCatalog,
    recipe_id: &str,
) -> Result<Vec<Effect>, ModelError> {
    Ok(brew.open(&RecipeId::try_parse(recipe_id)?, catalog))
}

fn edit(
    editor: &mut EditorController,
    catalog: &RecipeCatalog,
    recipe_id: &str,
) -> Result<(), Box<dyn Error>> {
    editor.edit(&RecipeId::try_parse(recipe_id)?, catalog)?;
    Ok(())
}

fn duplicate(
    editor: &mut EditorController,
    catalog: &RecipeCatalog,
    recipe_id: &str,
) -> Result<(), Box<dyn Error>> {
    editor.duplicate(&RecipeId::try_parse(recipe_id)?, catalog)?;
    Ok(())
}

fn set_pre_alert_seconds(
    settings: &mut SettingsController,
    seconds: u32,
) -> Result<(), Box<dyn Error>> {
    settings.set_pre_alert(PreAlertLead::try_from_seconds(seconds)?)?;
    Ok(())
}

/// Runs `action` on the step row `row` names.
fn on_row<E: Error + 'static>(
    row: &str,
    action: impl FnOnce(&StepId) -> Result<(), E>,
) -> Result<(), Box<dyn Error>> {
    action(&StepId::try_parse(row)?)?;
    Ok(())
}

/// Sends the brew view, then carries out `effects`.
fn show_brew(brew: &BrewController, effects: &[Effect]) {
    BrewUpdate::from(brew.state()).send_signal_to_dart();
    for effect in effects {
        PlatformEffect::from(effect.clone()).send_signal_to_dart();
    }
}

fn show_editor(editor: &EditorController) {
    EditorUpdate::from(editor.state()).send_signal_to_dart();
}

fn show_settings(settings: &SettingsController) {
    SettingsUpdate::from(settings.state()).send_signal_to_dart();
}

/// The value, or none after logging the error. Anything the GUI must show
/// is already in the feature's state (the editor's notice), or, for a
/// failed list delete, undo, pin, ratio save, export or import, sent as
/// the one-shot `RecipeListUpdate::Failed`.
fn report<T>(result: Result<T, impl Display>) -> Option<T> {
    result.map_err(|error| debug_print!("{error}")).ok()
}

/// A beat whose first tick lands one second from now (a plain `interval`
/// ticks at once), then every second; a late beat delays the rest rather
/// than bursting to catch up.
fn every_second() -> Interval {
    let now = Instant::now();
    let first = now.checked_add(ONE_SECOND).unwrap_or(now);
    let mut ticker = interval_at(first, ONE_SECOND);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    ticker
}

/// The next beat; pending forever while no brew runs.
async fn next_tick(ticker: &mut Option<Interval>) {
    match ticker {
        Some(ticker) => {
            ticker.tick().await;
        }
        None => std::future::pending().await,
    }
}
