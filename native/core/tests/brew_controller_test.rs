//! `BrewController`: the brew page and its step timer, tick by tick. Legacy
//! reference: `lib/core/recipe/actors/recipe_actor.dart` (timer) and
//! `lib/recipe_details/pages/recipe_details_page.dart` (what is shown).

use coffee_timer_core::brew::{BrewController, BrewError, BrewState, Effect, StepStatus};
use coffee_timer_core::catalog::RecipeCatalog;
use coffee_timer_core::fakes::FakeRecipeStore;
use coffee_timer_core::model::{
    ModelError, Recipe, RecipeId, RecipeName, RecipeStep, StepDuration, StepId,
};

use StepStatus::{Current, Done, Pending};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const BUZZ: Effect = Effect::Vibrate { ms: 100 };
const SCREEN_ON: Effect = Effect::KeepScreenOn(true);
const SCREEN_OFF: Effect = Effect::KeepScreenOn(false);

/// A recipe `id` whose steps are named `Step 1`, `Step 2`, … and last
/// `step_seconds` each.
fn recipe(id: &str, step_seconds: &[i64]) -> Result<Recipe, ModelError> {
    let steps = step_seconds
        .iter()
        .enumerate()
        .map(|(i, &seconds)| {
            Ok(RecipeStep::new(
                StepId::try_parse(&format!("{id}-s{i}"))?,
                format!("Step {}", i + 1),
                StepDuration::try_parse(seconds)?,
            ))
        })
        .collect::<Result<_, ModelError>>()?;
    Ok(Recipe::new(
        RecipeId::try_parse(id)?,
        RecipeName::try_parse(&format!("Recipe {id}"))?,
        format!("About {id}"),
        steps,
    ))
}

fn catalog(recipes: Vec<Recipe>) -> Result<RecipeCatalog, Box<dyn std::error::Error>> {
    let mut catalog = RecipeCatalog::new(Box::new(FakeRecipeStore::with_recipes(recipes)));
    catalog.load()?;
    Ok(catalog)
}

fn id(raw: &str) -> Result<RecipeId, ModelError> {
    RecipeId::try_parse(raw)
}

/// A controller with recipe `a` (`step_seconds`) open, plus recipe `b`.
fn opened(
    step_seconds: &[i64],
) -> Result<(BrewController, RecipeCatalog), Box<dyn std::error::Error>> {
    let catalog = catalog(vec![recipe("a", step_seconds)?, recipe("b", &[5])?])?;
    let mut brew = BrewController::new();
    let effects = brew.open(&id("a")?, &catalog);
    assert_eq!(effects, []);
    Ok((brew, catalog))
}

/// `(current step index, remaining MM:SS)` while running; `None` otherwise.
fn timer(brew: &BrewController) -> Option<(usize, String)> {
    brew.state()
        .running()
        .map(|step| (step.index(), step.remaining_label()))
}

fn running(index: usize, remaining: &str) -> Option<(usize, String)> {
    Some((index, remaining.to_owned()))
}

fn statuses(brew: &BrewController) -> Vec<StepStatus> {
    brew.state()
        .step_rows()
        .iter()
        .map(|row| row.status)
        .collect()
}

fn is_idle(brew: &BrewController) -> bool {
    matches!(brew.state(), BrewState::Idle(_))
}

// open

#[test]
fn open_shows_the_recipe_idle_with_its_labels() -> TestResult {
    let (brew, _catalog) = opened(&[30, 60, 5])?;
    let view = brew.state().view().ok_or("no view")?;

    assert!(is_idle(&brew));
    assert_eq!(view.recipe_id(), &id("a")?);
    assert_eq!(view.name(), "Recipe a");
    assert_eq!(view.description(), "About a");
    assert_eq!(view.coffee_label(), "30g");
    assert_eq!(view.water_label(), "150g");
    assert_eq!(view.duration_label(), "1:35");

    let rows = brew.state().step_rows();
    let shown: Vec<_> = rows
        .iter()
        .map(|r| {
            (
                r.number_label.as_str(),
                r.name.as_str(),
                r.duration_label.as_str(),
            )
        })
        .collect();
    assert_eq!(
        shown,
        [
            ("1", "Step 1", "30s"),
            ("2", "Step 2", "60s"),
            ("3", "Step 3", "5s")
        ]
    );
    assert_eq!(statuses(&brew), [Pending, Pending, Pending]);
    Ok(())
}

#[test]
fn open_an_unknown_id_shows_empty_recipe() -> TestResult {
    let catalog = catalog(vec![recipe("a", &[5])?])?;
    let mut brew = BrewController::new();

    assert_eq!(brew.open(&id("missing")?, &catalog), []);
    assert_eq!(brew.state(), &BrewState::Empty);
    assert_eq!(BrewState::EMPTY_LABEL, "Empty recipe");
    assert!(brew.state().step_rows().is_empty());
    Ok(())
}

#[test]
fn a_new_controller_shows_empty_recipe() {
    let brew = BrewController::new();
    assert_eq!(brew.state(), &BrewState::Empty);
}

#[test]
fn open_while_running_stops_the_brew_silently() -> TestResult {
    let (mut brew, catalog) = opened(&[5])?;
    brew.start(0)?;

    assert_eq!(brew.open(&id("b")?, &catalog), [SCREEN_OFF]);
    assert!(is_idle(&brew));
    assert_eq!(brew.state().view().map(|v| v.name()), Some("Recipe b"));
    Ok(())
}

// start and tick

#[test]
fn a_brew_runs_tick_by_tick_to_the_end() -> TestResult {
    let (mut brew, _catalog) = opened(&[2, 1])?;

    assert_eq!(brew.start(0)?, [SCREEN_ON]);
    assert_eq!(timer(&brew), running(0, "00:02"));
    assert_eq!(statuses(&brew), [Current, Pending]);

    assert_eq!(brew.tick(), []);
    assert_eq!(timer(&brew), running(0, "00:01"));

    // Boundary: the next step shows its full duration at once.
    assert_eq!(brew.tick(), [BUZZ, Effect::PlayStepSound]);
    assert_eq!(timer(&brew), running(1, "00:01"));
    assert_eq!(statuses(&brew), [Done, Current]);

    // After the last step: screen-on off, ended, vibrate, no sound.
    assert_eq!(brew.tick(), [SCREEN_OFF, BUZZ]);
    assert_eq!(timer(&brew), None);
    assert!(is_idle(&brew));
    assert_eq!(statuses(&brew), [Pending, Pending]);
    Ok(())
}

#[test]
fn the_running_step_carries_its_name_and_sweep() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 0])?;
    brew.start(0)?;
    let step = brew.state().running().ok_or("not running")?;
    assert_eq!(step.name(), "Step 1");
    assert_eq!(step.remaining(), StepDuration::try_parse(3)?);
    assert_eq!(step.sweep(), Some(StepDuration::try_parse(3)?));

    brew.start(1)?;
    let step = brew.state().running().ok_or("not running")?;
    assert_eq!(step.sweep(), None, "a 0-second step has no sweep");
    Ok(())
}

#[test]
fn a_zero_second_first_step_shows_00_00_for_one_tick() -> TestResult {
    let (mut brew, _catalog) = opened(&[0, 2])?;

    brew.start(0)?;
    assert_eq!(timer(&brew), running(0, "00:00"));

    assert_eq!(brew.tick(), [BUZZ, Effect::PlayStepSound]);
    assert_eq!(timer(&brew), running(1, "00:02"));
    Ok(())
}

#[test]
fn a_zero_second_last_step_shows_00_00_for_one_tick_then_ends() -> TestResult {
    let (mut brew, _catalog) = opened(&[1, 0])?;
    brew.start(0)?;

    assert_eq!(brew.tick(), [BUZZ, Effect::PlayStepSound]);
    assert_eq!(timer(&brew), running(1, "00:00"));

    assert_eq!(brew.tick(), [SCREEN_OFF, BUZZ]);
    assert!(is_idle(&brew));
    Ok(())
}

#[test]
fn a_zero_second_middle_step_shows_00_00_for_one_tick() -> TestResult {
    let (mut brew, _catalog) = opened(&[1, 0, 1])?;
    brew.start(0)?;

    assert_eq!(brew.tick(), [BUZZ, Effect::PlayStepSound]);
    assert_eq!(timer(&brew), running(1, "00:00"));
    assert_eq!(brew.tick(), [BUZZ, Effect::PlayStepSound]);
    assert_eq!(timer(&brew), running(2, "00:01"));
    assert_eq!(brew.tick(), [SCREEN_OFF, BUZZ]);
    Ok(())
}

#[test]
fn a_single_zero_second_step_ends_after_one_tick() -> TestResult {
    let (mut brew, _catalog) = opened(&[0])?;
    assert_eq!(brew.start(0)?, [SCREEN_ON]);
    assert_eq!(timer(&brew), running(0, "00:00"));
    assert_eq!(brew.tick(), [SCREEN_OFF, BUZZ]);
    assert!(is_idle(&brew));
    Ok(())
}

#[test]
fn the_longest_step_shows_99_59() -> TestResult {
    let (mut brew, _catalog) = opened(&[5999])?;
    brew.start(0)?;
    assert_eq!(timer(&brew), running(0, "99:59"));
    brew.tick();
    assert_eq!(timer(&brew), running(0, "99:58"));
    Ok(())
}

#[test]
fn start_from_a_middle_step() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 2, 1])?;

    assert_eq!(brew.start(1)?, [SCREEN_ON]);
    assert_eq!(timer(&brew), running(1, "00:02"));
    assert_eq!(statuses(&brew), [Done, Current, Pending]);

    assert_eq!(brew.tick(), []);
    assert_eq!(brew.tick(), [BUZZ, Effect::PlayStepSound]);
    assert_eq!(timer(&brew), running(2, "00:01"));
    assert_eq!(statuses(&brew), [Done, Done, Current]);

    assert_eq!(brew.tick(), [SCREEN_OFF, BUZZ]);
    assert!(is_idle(&brew));
    Ok(())
}

#[test]
fn start_from_the_last_step() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 1])?;
    brew.start(1)?;
    assert_eq!(statuses(&brew), [Done, Current]);
    assert_eq!(brew.tick(), [SCREEN_OFF, BUZZ]);
    Ok(())
}

#[test]
fn start_past_the_last_step_is_an_error_and_changes_nothing() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 2])?;
    let before = brew.state().clone();

    assert_eq!(
        brew.start(2),
        Err(BrewError::StepOutOfRange { index: 2, len: 2 })
    );
    assert_eq!(brew.state(), &before);

    brew.start(0)?;
    brew.tick();
    let running_before = brew.state().clone();
    assert!(brew.start(7).is_err());
    assert_eq!(brew.state(), &running_before, "the running brew goes on");
    Ok(())
}

#[test]
fn start_on_a_recipe_without_steps_is_an_error() -> TestResult {
    let (mut brew, _catalog) = opened(&[])?;
    assert_eq!(
        brew.start(0),
        Err(BrewError::StepOutOfRange { index: 0, len: 0 })
    );
    assert!(is_idle(&brew));
    Ok(())
}

#[test]
fn start_with_no_recipe_open_is_an_error() {
    let mut brew = BrewController::new();
    assert_eq!(brew.start(0), Err(BrewError::NoRecipe));
    assert_eq!(brew.state(), &BrewState::Empty);
}

#[test]
fn start_while_running_restarts_from_the_chosen_step() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.tick();

    assert_eq!(brew.start(1)?, [SCREEN_ON]);
    assert_eq!(timer(&brew), running(1, "00:02"));
    assert_eq!(brew.start(0)?, [SCREEN_ON]);
    assert_eq!(timer(&brew), running(0, "00:03"));
    Ok(())
}

#[test]
fn restart_after_the_brew_has_ended() -> TestResult {
    let (mut brew, _catalog) = opened(&[1])?;
    brew.start(0)?;
    assert_eq!(brew.tick(), [SCREEN_OFF, BUZZ]);

    assert_eq!(brew.start(0)?, [SCREEN_ON]);
    assert_eq!(timer(&brew), running(0, "00:01"));
    assert_eq!(brew.tick(), [SCREEN_OFF, BUZZ]);
    Ok(())
}

#[test]
fn ticks_while_not_running_do_nothing() -> TestResult {
    let mut empty = BrewController::new();
    assert_eq!(empty.tick(), []);
    assert_eq!(empty.state(), &BrewState::Empty);

    let (mut brew, _catalog) = opened(&[1])?;
    let idle = brew.state().clone();
    assert_eq!(brew.tick(), []);
    assert_eq!(brew.state(), &idle);

    brew.start(0)?;
    brew.tick();
    let ended = brew.state().clone();
    assert_eq!(ended, idle, "an ended brew looks like one never started");
    assert_eq!(brew.tick(), []);
    assert_eq!(brew.state(), &ended);
    Ok(())
}

// reset and leave

#[test]
fn reset_mid_brew_stops_and_turns_screen_on_off() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.tick();

    assert_eq!(brew.reset(), [SCREEN_OFF]);
    assert!(is_idle(&brew));
    assert_eq!(statuses(&brew), [Pending, Pending]);
    assert_eq!(brew.tick(), []);
    Ok(())
}

#[test]
fn reset_while_not_running_does_nothing() -> TestResult {
    let mut empty = BrewController::new();
    assert_eq!(empty.reset(), []);
    assert_eq!(empty.state(), &BrewState::Empty);

    let (mut brew, _catalog) = opened(&[3])?;
    let idle = brew.state().clone();
    assert_eq!(brew.reset(), []);
    assert_eq!(brew.state(), &idle);
    Ok(())
}

#[test]
fn leave_mid_brew_stops_silently() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.tick();

    assert_eq!(brew.leave(), [SCREEN_OFF], "no vibration, no sound");
    assert_eq!(brew.state(), &BrewState::Empty);
    assert_eq!(brew.tick(), []);
    Ok(())
}

#[test]
fn leave_while_idle_has_no_effects() -> TestResult {
    let (mut brew, _catalog) = opened(&[3])?;
    assert_eq!(brew.leave(), []);
    assert_eq!(brew.state(), &BrewState::Empty);
    Ok(())
}

// catalog changes

#[test]
fn an_edit_to_the_open_recipe_mid_brew_resets_to_the_new_version() -> TestResult {
    let (mut brew, mut catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.tick();

    let edited = recipe("a", &[10])?.renamed(RecipeName::try_parse("Ristretto")?);
    let change = catalog.upsert(edited)?;

    assert_eq!(brew.on_catalog_change(&change, &catalog), [SCREEN_OFF]);
    assert!(is_idle(&brew));
    let view = brew.state().view().ok_or("no view")?;
    assert_eq!(view.name(), "Ristretto");
    assert_eq!(view.coffee_label(), "10g");
    assert_eq!(view.duration_label(), "0:10");
    assert_eq!(statuses(&brew), [Pending]);
    Ok(())
}

#[test]
fn an_edit_to_the_open_recipe_while_idle_refreshes_the_view() -> TestResult {
    let (mut brew, mut catalog) = opened(&[3])?;
    let change = catalog.upsert(recipe("a", &[3, 4])?.described_as("New".to_owned()))?;

    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert!(is_idle(&brew));
    let view = brew.state().view().ok_or("no view")?;
    assert_eq!(view.description(), "New");
    assert_eq!(statuses(&brew), [Pending, Pending]);
    Ok(())
}

#[test]
fn an_edit_to_another_recipe_changes_nothing() -> TestResult {
    let (mut brew, mut catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.tick();
    let before = brew.state().clone();

    let change = catalog.upsert(recipe("b", &[9, 9])?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &before);

    let change = catalog.upsert(recipe("c", &[1])?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &before);

    assert_eq!(brew.tick(), [], "still running");
    assert_eq!(timer(&brew), running(0, "00:01"));
    assert_eq!(brew.tick(), [BUZZ, Effect::PlayStepSound]);
    Ok(())
}

#[test]
fn deleting_the_open_recipe_mid_brew_shows_empty_recipe() -> TestResult {
    let (mut brew, mut catalog) = opened(&[3, 2])?;
    brew.start(0)?;

    let change = catalog.remove(id("a")?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), [SCREEN_OFF]);
    assert_eq!(brew.state(), &BrewState::Empty);
    assert_eq!(brew.tick(), []);
    Ok(())
}

#[test]
fn deleting_the_open_recipe_while_idle_shows_empty_recipe() -> TestResult {
    let (mut brew, mut catalog) = opened(&[3])?;
    let change = catalog.remove(id("a")?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &BrewState::Empty);
    Ok(())
}

#[test]
fn deleting_another_recipe_changes_nothing() -> TestResult {
    let (mut brew, mut catalog) = opened(&[3])?;
    brew.start(0)?;
    let before = brew.state().clone();

    let change = catalog.remove(id("b")?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &before);
    Ok(())
}

#[test]
fn catalog_changes_with_no_recipe_open_do_nothing() -> TestResult {
    let mut catalog = catalog(vec![recipe("a", &[3])?])?;
    let mut brew = BrewController::new();

    let change = catalog.upsert(recipe("a", &[4])?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &BrewState::Empty);
    Ok(())
}
