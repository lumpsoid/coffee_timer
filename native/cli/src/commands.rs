//! One function per command: arguments in, controller actions, state out.
//! Each takes the controllers it uses, so tests can run it against an
//! in-memory store and read what it printed.

use std::io::Write;
use std::path::Path;

use coffee_timer_core::brew::{BrewController, RatioChange};
use coffee_timer_core::catalog::{CatalogChange, RecipeCatalog, RecipeSummary};
use coffee_timer_core::editor::{EditError, EditorController, StepPreset};
use coffee_timer_core::model::{
    AccentColor, AccentOption, BrewMetrics, CoffeeDose, FinishSound, Pin, PreAlertLead, RecipeId,
    Settings, StepId, StepSound, ThemeSelection, Toggle,
};
use coffee_timer_core::settings::SettingsController;

use crate::args::{NewRecipe, SettingsAction, StepArg};
use crate::cli_error::CliError;
use crate::render;

/// Every recipe in list order, pinned ones first.
pub fn list(catalog: &RecipeCatalog, out: &mut impl Write) -> Result<(), CliError> {
    let metrics = BrewMetrics::default();
    for recipe in catalog.listed() {
        let pin = catalog.pin_of(recipe.id());
        render::summary(out, &RecipeSummary::of(recipe, pin, &metrics))?;
    }
    Ok(())
}

/// Pins or unpins the recipe, as the list's long-press Pin / Unpin does,
/// and passes the change on.
pub fn set_pin(
    id: &str,
    pin: Pin,
    catalog: &mut RecipeCatalog,
    brew: &mut BrewController,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let change = catalog.set_pin(&RecipeId::try_parse(id)?, pin)?;
    pass_on(&change, brew, catalog, out)
}

/// Moves the recipe to place `to` in the list, as the list's reorder mode
/// does, and passes the change on.
pub fn move_to(
    id: &str,
    to: usize,
    catalog: &mut RecipeCatalog,
    brew: &mut BrewController,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let change = catalog.move_to(&RecipeId::try_parse(id)?, to)?;
    pass_on(&change, brew, catalog, out)
}

/// Opens the recipe, sets its `coffee` grams as the brew page's coffee tape
/// does, then presses its more-water (positive) or less-water (negative)
/// button `ratio_steps` times. With `save`, then saves that
/// dose to the recipe ("Save to recipe") and passes the change on before
/// showing the page.
pub fn show(
    id: &str,
    coffee: Option<i64>,
    ratio_steps: i32,
    save: bool,
    brew: &mut BrewController,
    catalog: &mut RecipeCatalog,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let effects = brew.open(&RecipeId::try_parse(id)?, catalog);
    render::effects(out, &effects)?;
    if let Some(coffee) = coffee {
        brew.set_coffee(CoffeeDose::try_parse(coffee)?)?;
    }
    let change = if ratio_steps < 0 {
        RatioChange::LessWater
    } else {
        RatioChange::MoreWater
    };
    for _ in 0..ratio_steps.unsigned_abs() {
        brew.adjust_ratio(change)?;
    }
    if save {
        let change = brew.save_ratio(catalog)?;
        pass_on(&change, brew, catalog, out)?;
    }
    render::brew_page(out, brew.state())?;
    Ok(())
}

/// Fills a new draft the way the editor page would: the coffee and water
/// fields take `--coffee` and `--water` when given, the draft's first row
/// takes the first step, and each further step is an added row. A step's
/// `@GRAMS` goes in its grams field.
pub fn new_recipe(
    recipe: &NewRecipe,
    editor: &mut EditorController,
    catalog: &mut RecipeCatalog,
    brew: &mut BrewController,
    out: &mut impl Write,
) -> Result<(), CliError> {
    editor.new_draft();
    editor.set_name(&recipe.name)?;
    editor.set_description(&recipe.description)?;
    if let Some(coffee) = &recipe.coffee {
        editor.set_coffee(coffee)?;
    }
    if let Some(water) = &recipe.water {
        editor.set_water(water)?;
    }
    if let Some(pour_speed) = &recipe.pour_speed {
        editor.set_pour_speed(pour_speed)?;
    }
    if let Some(grind) = &recipe.grind {
        editor.set_grind(grind)?;
    }
    let mut steps = recipe.steps.iter();
    if let Some(step) = steps.next() {
        let first_row = editor
            .state()
            .draft()
            .and_then(|draft| draft.step_rows().into_iter().next())
            .map(|row| row.id)
            .ok_or(EditError::NoDraft)?;
        fill_step(editor, &first_row, step)?;
    }
    for step in steps {
        let row = editor.add_step(StepPreset::Blank)?;
        fill_step(editor, &row, step)?;
    }
    let change = editor.save(catalog).map_err(|error| CliError::Save {
        error,
        fields: render::field_errors(editor.state()),
    })?;
    pass_on(&change, brew, catalog, out)
}

/// Types `step` into the draft's `row`, as the editor's step card would.
/// The grams go first, so typed seconds win over the time the pour takes;
/// a pouring step without seconds keeps that time.
fn fill_step(editor: &mut EditorController, row: &StepId, step: &StepArg) -> Result<(), CliError> {
    editor.set_step_name(row, &step.name)?;
    if let Some(water) = &step.water {
        editor.set_step_water(row, water)?;
    }
    if step.water.is_none() || !step.seconds.is_empty() {
        editor.set_step_duration(row, &step.seconds)?;
    }
    Ok(())
}

pub fn delete(
    id: &str,
    editor: &mut EditorController,
    catalog: &mut RecipeCatalog,
    brew: &mut BrewController,
    out: &mut impl Write,
) -> Result<(), CliError> {
    editor.edit(&RecipeId::try_parse(id)?, catalog)?;
    let change = editor.delete(catalog)?;
    pass_on(&change, brew, catalog, out)
}

/// Opens a copy of the recipe in the editor, as the list's Duplicate
/// does, types `name` over the copy's name if given, and saves it. The
/// change line names the copy's new id.
pub fn duplicate(
    id: &str,
    name: Option<&str>,
    editor: &mut EditorController,
    catalog: &mut RecipeCatalog,
    brew: &mut BrewController,
    out: &mut impl Write,
) -> Result<(), CliError> {
    editor.duplicate(&RecipeId::try_parse(id)?, catalog)?;
    if let Some(name) = name {
        editor.set_name(name)?;
    }
    let change = editor.save(catalog).map_err(|error| CliError::Save {
        error,
        fields: render::field_errors(editor.state()),
    })?;
    pass_on(&change, brew, catalog, out)
}

/// Writes the recipes `ids` names (every recipe, in list order, for none)
/// as a recipe document: to `file` if given, saying so, else to `out`.
pub fn export(
    ids: &[String],
    file: Option<&Path>,
    catalog: &RecipeCatalog,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let ids = ids
        .iter()
        .map(|id| RecipeId::try_parse(id))
        .collect::<Result<Vec<_>, _>>()?;
    let export = catalog.export(&ids)?;
    match file {
        Some(file) => {
            std::fs::write(file, format!("{}\n", export.json()))?;
            writeln!(
                out,
                "Exported {} to {}",
                export.count_label(),
                file.display()
            )?;
        }
        None => writeln!(out, "{}", export.json())?,
    }
    Ok(())
}

/// Adds every recipe in the document `text` as a new recipe, passes each
/// change on (its line names the new id), then prints the import's notice.
pub fn import(
    text: &str,
    editor: &mut EditorController,
    catalog: &mut RecipeCatalog,
    brew: &mut BrewController,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let import = editor.import(text, catalog)?;
    for change in import.changes() {
        pass_on(change, brew, catalog, out)?;
    }
    writeln!(out, "{}", import.notice())?;
    Ok(())
}

/// When `brew` pauses: after tick `at`, for `seconds` waits, during which
/// it does not tick (as the app's shell does not).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrewPause {
    pub at: u32,
    pub seconds: u32,
}

/// How `brew` runs: the step it starts from, and when it pauses, if ever.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrewRun {
    pub from: usize,
    pub pause: Option<BrewPause>,
}

/// Opens the recipe, starts it at `run.from`, then ticks while it runs,
/// calling `wait` before each tick (a one-second sleep, or nothing with
/// `--fast`). With `run.pause`, pauses and later resumes the brew as the
/// dial does. Each tick gives the cues `settings` allow, so a muted sound
/// or vibration prints no such effect.
pub fn brew(
    id: &str,
    run: BrewRun,
    wait: &mut dyn FnMut(),
    brew: &mut BrewController,
    catalog: &RecipeCatalog,
    settings: &Settings,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let opened = brew.open(&RecipeId::try_parse(id)?, catalog);
    render::effects(out, &opened)?;
    let started = brew.start(run.from)?;
    writeln!(out, "start    {}", render::timer_line(brew.state()))?;
    render::effects(out, &started)?;
    let mut tick: u32 = 0;
    while brew.state().is_running() {
        if let Some(pause) = run.pause.filter(|pause| pause.at == tick) {
            let paused = brew.pause()?;
            writeln!(out, "pause    {}", render::timer_line(brew.state()))?;
            render::effects(out, &paused)?;
            for _ in 0..pause.seconds {
                wait();
            }
            let resumed = brew.resume()?;
            writeln!(out, "resume   {}", render::timer_line(brew.state()))?;
            render::effects(out, &resumed)?;
        }
        wait();
        let effects = brew.tick(settings);
        tick = tick.saturating_add(1);
        writeln!(out, "tick {tick:<3} {}", render::timer_line(brew.state()))?;
        render::effects(out, &effects)?;
    }
    Ok(())
}

/// Shows the settings, after applying `action` if one is given.
pub fn settings(
    action: Option<&SettingsAction>,
    settings: &mut SettingsController,
    out: &mut impl Write,
) -> Result<(), CliError> {
    match action {
        None => {}
        Some(SettingsAction::Theme { value }) => {
            settings.set_theme(ThemeSelection::try_parse(value)?)?;
        }
        Some(SettingsAction::Accent { value }) => {
            let accent = match AccentOption::named(value) {
                Some(option) => option.color(),
                None => AccentColor::try_from_hex(value)?,
            };
            settings.set_accent(accent)?;
        }
        Some(SettingsAction::Sound { value }) => {
            settings.set_sound(Toggle::try_parse(value)?)?;
        }
        Some(SettingsAction::Vibration { value }) => {
            settings.set_vibration(Toggle::try_parse(value)?)?;
        }
        Some(SettingsAction::PreAlert { value }) => {
            settings.set_pre_alert(PreAlertLead::try_parse(value)?)?;
        }
        Some(SettingsAction::StepSound { value }) => {
            settings.set_step_sound(StepSound::try_parse(value)?)?;
        }
        Some(SettingsAction::FinishSound { value }) => {
            settings.set_finish_sound(FinishSound::try_parse(value)?)?;
        }
    }
    render::settings(out, settings.state())?;
    Ok(())
}

/// What every head does with a persisted change: show it, and hand it to
/// the brew view.
fn pass_on(
    change: &CatalogChange,
    brew: &mut BrewController,
    catalog: &RecipeCatalog,
    out: &mut impl Write,
) -> Result<(), CliError> {
    render::change(out, change)?;
    let effects = brew.on_catalog_change(change, catalog);
    render::effects(out, &effects)?;
    Ok(())
}
