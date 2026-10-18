//! One function per command: arguments in, controller actions, state out.
//! Each takes the controllers it uses, so tests can run it against an
//! in-memory store and read what it printed.

use std::io::Write;

use coffee_timer_core::brew::{BrewController, RatioChange};
use coffee_timer_core::catalog::{CatalogChange, RecipeCatalog, RecipeSummary};
use coffee_timer_core::editor::{EditError, EditorController, StepPreset};
use coffee_timer_core::model::{AccentColor, AccentOption, BrewMetrics, RecipeId, ThemeSelection};
use coffee_timer_core::settings::SettingsController;

use crate::args::{NewRecipe, SettingsAction};
use crate::cli_error::CliError;
use crate::render;

pub fn list(catalog: &RecipeCatalog, out: &mut impl Write) -> Result<(), CliError> {
    let metrics = BrewMetrics::default();
    for recipe in catalog.state() {
        render::summary(out, &RecipeSummary::of(recipe, &metrics))?;
    }
    Ok(())
}

/// Opens the recipe, then presses the brew page's more-water (positive) or
/// less-water (negative) button `ratio_steps` times.
pub fn show(
    id: &str,
    ratio_steps: i32,
    brew: &mut BrewController,
    catalog: &RecipeCatalog,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let effects = brew.open(&RecipeId::try_parse(id)?, catalog);
    render::effects(out, &effects)?;
    let change = if ratio_steps < 0 {
        RatioChange::LessWater
    } else {
        RatioChange::MoreWater
    };
    for _ in 0..ratio_steps.unsigned_abs() {
        brew.adjust_ratio(change)?;
    }
    render::brew_page(out, brew.state())?;
    Ok(())
}

/// Fills a new draft the way the editor page would: the draft's first row
/// takes the first step, and each further step is an added row.
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
    let mut steps = recipe.steps.iter();
    if let Some(step) = steps.next() {
        let first_row = editor
            .state()
            .draft()
            .and_then(|draft| draft.step_rows().into_iter().next())
            .map(|row| row.id)
            .ok_or(EditError::NoDraft)?;
        editor.set_step_name(&first_row, &step.name)?;
        editor.set_step_duration(&first_row, &step.seconds)?;
    }
    for step in steps {
        let row = editor.add_step(StepPreset::Blank)?;
        editor.set_step_name(&row, &step.name)?;
        editor.set_step_duration(&row, &step.seconds)?;
    }
    let change = editor.save(catalog).map_err(|error| CliError::Save {
        error,
        fields: render::field_errors(editor.state()),
    })?;
    pass_on(&change, brew, catalog, out)
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

/// Opens the recipe, starts it at `from`, then ticks while it runs, calling
/// `wait` before each tick (a one-second sleep, or nothing with `--fast`).
pub fn brew(
    id: &str,
    from: usize,
    wait: &mut dyn FnMut(),
    brew: &mut BrewController,
    catalog: &RecipeCatalog,
    out: &mut impl Write,
) -> Result<(), CliError> {
    let opened = brew.open(&RecipeId::try_parse(id)?, catalog);
    render::effects(out, &opened)?;
    let started = brew.start(from)?;
    writeln!(out, "start    {}", render::timer_line(brew.state()))?;
    render::effects(out, &started)?;
    let mut tick: u32 = 0;
    while brew.state().is_running() {
        wait();
        let effects = brew.tick();
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
