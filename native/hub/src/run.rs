//! The hub task: owns every controller, turns each Dart signal into one
//! controller action, and sends the feature's state after it. One task, so
//! actions run one at a time. It decides nothing: errors are logged and the
//! state re-sent, and every `CatalogChange` is passed on unconditionally.

use std::error::Error;
use std::fmt::Display;
use std::time::Duration;

use coffee_timer_core::brew::{BrewController, Effect};
use coffee_timer_core::catalog::{CatalogChange, RecipeCatalog};
use coffee_timer_core::editor::{DeleteError, EditorController, SaveError};
use coffee_timer_core::model::{AccentColor, ModelError, RecipeId, StepId};
use coffee_timer_core::ports::StoreError;
use coffee_timer_core::settings::SettingsController;
use coffee_timer_store::{SqliteStore, UuidIds};
use rinf::{DartSignal, RustSignal, debug_print};
use tokio::time::{Instant, Interval, MissedTickBehavior, interval_at};

use crate::signals::{
    AddStep, BrewReady, BrewUpdate, CloseEditor, DeleteRecipe, EditRecipe, EditorReady,
    EditorUpdate, LeaveBrew, NewDraft, OpenBrew, PlatformEffect, RecipeListReady,
    RecipeListUpdate, RemoveStep, ResetBrew, SaveRecipe, SetAccent, SetRecipeDescription,
    SetRecipeName, SetStepDuration, SetStepName, SetTheme, SettingsReady, SettingsUpdate,
    StartBrew,
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
    let brew_ready = BrewReady::get_dart_signal_receiver();
    let open_brew = OpenBrew::get_dart_signal_receiver();
    let start_brew = StartBrew::get_dart_signal_receiver();
    let reset_brew = ResetBrew::get_dart_signal_receiver();
    let leave_brew = LeaveBrew::get_dart_signal_receiver();
    let editor_ready = EditorReady::get_dart_signal_receiver();
    let new_draft = NewDraft::get_dart_signal_receiver();
    let edit_recipe = EditRecipe::get_dart_signal_receiver();
    let set_recipe_name = SetRecipeName::get_dart_signal_receiver();
    let set_recipe_description = SetRecipeDescription::get_dart_signal_receiver();
    let set_step_name = SetStepName::get_dart_signal_receiver();
    let set_step_duration = SetStepDuration::get_dart_signal_receiver();
    let add_step = AddStep::get_dart_signal_receiver();
    let remove_step = RemoveStep::get_dart_signal_receiver();
    let save_recipe = SaveRecipe::get_dart_signal_receiver();
    let delete_recipe = DeleteRecipe::get_dart_signal_receiver();
    let close_editor = CloseEditor::get_dart_signal_receiver();
    let settings_ready = SettingsReady::get_dart_signal_receiver();
    let set_theme = SetTheme::get_dart_signal_receiver();
    let set_accent = SetAccent::get_dart_signal_receiver();

    // The brew's one-second beat: exists exactly while a brew runs.
    let mut ticker: Option<Interval> = None;

    loop {
        tokio::select! {
            // Recipe list
            Some(_) = recipe_list_ready.recv() => {
                RecipeListUpdate::snapshot(&c.catalog).send_signal_to_dart();
            }

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
                let effects = c.brew.tick();
                show_brew(&c.brew, &effects);
            }
            Some(_) = reset_brew.recv() => {
                let effects = c.brew.reset();
                show_brew(&c.brew, &effects);
            }
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
            Some(pack) = set_recipe_name.recv() => {
                report(c.editor.set_name(&pack.message.raw));
                show_editor(&c.editor);
            }
            Some(pack) = set_recipe_description.recv() => {
                report(c.editor.set_description(&pack.message.raw));
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
            Some(pack) = add_step.recv() => {
                report(c.editor.add_step(pack.message.preset.into()));
                show_editor(&c.editor);
            }
            Some(pack) = remove_step.recv() => {
                report(on_row(&pack.message.row, |row| c.editor.remove_step(row)));
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
    for &effect in effects {
        PlatformEffect::from(effect).send_signal_to_dart();
    }
}

fn show_editor(editor: &EditorController) {
    EditorUpdate::from(editor.state()).send_signal_to_dart();
}

fn show_settings(settings: &SettingsController) {
    SettingsUpdate::from(settings.state()).send_signal_to_dart();
}

/// The value, or none after logging the error. Anything the GUI must show
/// is already in the feature's state (the editor's notice).
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
