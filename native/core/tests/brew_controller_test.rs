//! `BrewController`: the brew page and its step timer, tick by tick. Legacy
//! reference: `lib/core/recipe/actors/recipe_actor.dart` (timer) and
//! `lib/recipe_details/pages/recipe_details_page.dart` (what is shown).

use coffee_timer_core::brew::{
    BrewController, BrewError, BrewNotice, BrewState, BrewView, CoffeeSetting, CuePolicy, Effect,
    RatioChange, RatioSetting, SaveRatioError, StepStatus,
};
use coffee_timer_core::catalog::{CatalogChange, RecipeCatalog, RecipeSummary};
use coffee_timer_core::fakes::FakeRecipeStore;
use coffee_timer_core::model::{
    BrewMetrics, CoffeeDose, LabelStyle, ModelError, Pin, PreAlertLead, Recipe, RecipeId,
    RecipeName, RecipeStep, Settings, StepDuration, StepId, Toggle, WaterAmount, WaterScaleError,
};

use StepStatus::{Current, Done, Pending};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// The default settings: sound and vibration on, a 3 s pre-alert.
const SETTINGS: Settings = Settings::FALLBACK;

const BUZZ: Effect = Effect::Vibrate { ms: 100 };
const SCREEN_ON: Effect = Effect::KeepScreenOn(true);
const SCREEN_OFF: Effect = Effect::KeepScreenOn(false);
const HIDE: Effect = Effect::HideBrewNotification;
/// A running brew stops silently.
const STOPPED: [Effect; 2] = [SCREEN_OFF, HIDE];
/// The finish buzz: two long buzzes, unlike a step boundary's one.
fn finish_buzz() -> Effect {
    Effect::VibratePattern {
        pattern_ms: vec![0, 400, 200, 400],
    }
}

/// After the last step: screen-on off, the finish buzz, the sound, then
/// the notification gone.
fn ended() -> [Effect; 4] {
    [SCREEN_OFF, finish_buzz(), Effect::PlayStepSound, HIDE]
}

/// The brew notification: `title`, and the next step's name if any.
fn show(title: &str, next: Option<&str>) -> Effect {
    Effect::ShowBrewNotification(BrewNotice {
        title: title.to_owned(),
        next: next.map(|name| format!("Next: {name}")),
    })
}

/// A recipe `id`, 20 g coffee to 100 g water (`1:5`), whose steps are named
/// `Step 1`, `Step 2`, … and last `step_seconds` each.
fn recipe(id: &str, step_seconds: &[i64]) -> Result<Recipe, ModelError> {
    dosed(id, 20, 100, step_seconds)
}

/// [`recipe`] with `coffee` and `water` grams.
fn dosed(id: &str, coffee: i64, water: i64, step_seconds: &[i64]) -> Result<Recipe, ModelError> {
    let steps = step_seconds
        .iter()
        .enumerate()
        .map(|(i, &seconds)| {
            Ok(RecipeStep::new(
                StepId::try_parse(&format!("{id}-s{i}"))?,
                format!("Step {}", i + 1),
                StepDuration::try_parse(seconds)?,
                None,
            ))
        })
        .collect::<Result<_, ModelError>>()?;
    Ok(Recipe::new(
        RecipeId::try_parse(id)?,
        RecipeName::try_parse(&format!("Recipe {id}"))?,
        format!("About {id}"),
        CoffeeDose::try_parse(coffee)?,
        WaterAmount::try_parse(water)?,
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

/// The time a finished brew shows; `None` unless finished.
fn finished(brew: &BrewController) -> Option<&str> {
    brew.state().finished().map(|done| done.total_label())
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
    assert_eq!(view.coffee_label(), "20g");
    assert_eq!(view.water_label(), "100g");
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
fn open_carries_the_ratio_and_the_list_label_style() -> TestResult {
    let (brew, catalog) = opened(&[30, 60, 5])?;
    let view = brew.state().view().ok_or("no view")?;
    let recipe = catalog.get(&id("a")?).ok_or("no recipe")?;
    assert_eq!(view.ratio_label(), "1:5");
    assert_eq!(
        view.style(),
        LabelStyle::of(recipe, &BrewMetrics::default())
    );
    Ok(())
}

#[test]
fn steps_lie_end_to_end_on_the_track_by_duration() -> TestResult {
    let (brew, _catalog) = opened(&[30, 60, 10])?;
    let track: Vec<_> = brew
        .state()
        .step_rows()
        .iter()
        .map(|r| (r.track_start, r.track_end))
        .collect();
    assert_eq!(track, [(0.0, 0.3), (0.3, 0.9), (0.9, 1.0)]);
    Ok(())
}

#[test]
fn zero_second_steps_share_the_track_equally() -> TestResult {
    let (brew, _catalog) = opened(&[0, 0])?;
    let track: Vec<_> = brew
        .state()
        .step_rows()
        .iter()
        .map(|r| (r.track_start, r.track_end))
        .collect();
    assert_eq!(track, [(0.0, 0.5), (0.5, 1.0)]);
    Ok(())
}

#[test]
fn position_label_counts_from_one() -> TestResult {
    let (brew, _catalog) = opened(&[30, 60, 5])?;
    let view = brew.state().view().ok_or("no view")?;
    assert_eq!(view.position_label(0), "Step 1 of 3");
    assert_eq!(view.position_label(2), "Step 3 of 3");
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

    assert_eq!(brew.open(&id("b")?, &catalog), STOPPED);
    assert!(is_idle(&brew));
    assert_eq!(brew.state().view().map(|v| v.name()), Some("Recipe b"));
    Ok(())
}

// start and tick

#[test]
fn a_brew_runs_tick_by_tick_to_the_end() -> TestResult {
    let (mut brew, _catalog) = opened(&[2, 1])?;

    assert_eq!(
        brew.start(0)?,
        [SCREEN_ON, show("Step 1 · 00:02", Some("Step 2"))]
    );
    assert_eq!(timer(&brew), running(0, "00:02"));
    assert_eq!(statuses(&brew), [Current, Pending]);

    assert_eq!(
        brew.tick(&SETTINGS),
        [show("Step 1 · 00:01", Some("Step 2"))]
    );
    assert_eq!(timer(&brew), running(0, "00:01"));

    // Boundary: the next step shows its full duration at once.
    assert_eq!(
        brew.tick(&SETTINGS),
        [BUZZ, Effect::PlayStepSound, show("Step 2 · 00:01", None)]
    );
    assert_eq!(timer(&brew), running(1, "00:01"));
    assert_eq!(statuses(&brew), [Done, Current]);

    // After the last step: finished, every step done, the time brewed.
    assert_eq!(brew.tick(&SETTINGS), ended());
    assert_eq!(timer(&brew), None);
    assert_eq!(finished(&brew), Some("0:03"));
    assert_eq!(statuses(&brew), [Done, Done]);
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

    assert_eq!(
        brew.tick(&SETTINGS),
        [BUZZ, Effect::PlayStepSound, show("Step 2 · 00:02", None)]
    );
    assert_eq!(timer(&brew), running(1, "00:02"));
    Ok(())
}

#[test]
fn a_zero_second_last_step_shows_00_00_for_one_tick_then_ends() -> TestResult {
    let (mut brew, _catalog) = opened(&[1, 0])?;
    brew.start(0)?;

    assert_eq!(
        brew.tick(&SETTINGS),
        [BUZZ, Effect::PlayStepSound, show("Step 2 · 00:00", None)]
    );
    assert_eq!(timer(&brew), running(1, "00:00"));

    assert_eq!(brew.tick(&SETTINGS), ended());
    assert!(finished(&brew).is_some());
    Ok(())
}

#[test]
fn a_zero_second_middle_step_shows_00_00_for_one_tick() -> TestResult {
    let (mut brew, _catalog) = opened(&[1, 0, 1])?;
    brew.start(0)?;

    assert_eq!(
        brew.tick(&SETTINGS),
        [
            BUZZ,
            Effect::PlayStepSound,
            show("Step 2 · 00:00", Some("Step 3"))
        ]
    );
    assert_eq!(timer(&brew), running(1, "00:00"));
    assert_eq!(
        brew.tick(&SETTINGS),
        [BUZZ, Effect::PlayStepSound, show("Step 3 · 00:01", None)]
    );
    assert_eq!(timer(&brew), running(2, "00:01"));
    assert_eq!(brew.tick(&SETTINGS), ended());
    Ok(())
}

#[test]
fn a_single_zero_second_step_ends_after_one_tick() -> TestResult {
    let (mut brew, _catalog) = opened(&[0])?;
    assert_eq!(brew.start(0)?, [SCREEN_ON, show("Step 1 · 00:00", None)]);
    assert_eq!(timer(&brew), running(0, "00:00"));
    assert_eq!(brew.tick(&SETTINGS), ended());
    assert!(finished(&brew).is_some());
    Ok(())
}

#[test]
fn the_longest_step_shows_99_59() -> TestResult {
    let (mut brew, _catalog) = opened(&[5999])?;
    brew.start(0)?;
    assert_eq!(timer(&brew), running(0, "99:59"));
    brew.tick(&SETTINGS);
    assert_eq!(timer(&brew), running(0, "99:58"));
    Ok(())
}

#[test]
fn start_from_a_middle_step() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 2, 1])?;

    assert_eq!(
        brew.start(1)?,
        [SCREEN_ON, show("Step 2 · 00:02", Some("Step 3"))]
    );
    assert_eq!(timer(&brew), running(1, "00:02"));
    assert_eq!(statuses(&brew), [Done, Current, Pending]);

    assert_eq!(
        brew.tick(&SETTINGS),
        [show("Step 2 · 00:01", Some("Step 3"))]
    );
    assert_eq!(
        brew.tick(&SETTINGS),
        [BUZZ, Effect::PlayStepSound, show("Step 3 · 00:01", None)]
    );
    assert_eq!(timer(&brew), running(2, "00:01"));
    assert_eq!(statuses(&brew), [Done, Done, Current]);

    assert_eq!(brew.tick(&SETTINGS), ended());
    assert!(finished(&brew).is_some());
    Ok(())
}

#[test]
fn start_from_the_last_step() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 1])?;
    brew.start(1)?;
    assert_eq!(statuses(&brew), [Done, Current]);
    assert_eq!(brew.tick(&SETTINGS), ended());
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
    brew.tick(&SETTINGS);
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
    brew.tick(&SETTINGS);

    assert_eq!(brew.start(1)?, [SCREEN_ON, show("Step 2 · 00:02", None)]);
    assert_eq!(timer(&brew), running(1, "00:02"));
    assert_eq!(
        brew.start(0)?,
        [SCREEN_ON, show("Step 1 · 00:03", Some("Step 2"))]
    );
    assert_eq!(timer(&brew), running(0, "00:03"));
    Ok(())
}

#[test]
fn restart_after_the_brew_has_ended() -> TestResult {
    let (mut brew, _catalog) = opened(&[1])?;
    brew.start(0)?;
    assert_eq!(brew.tick(&SETTINGS), ended());

    assert_eq!(brew.start(0)?, [SCREEN_ON, show("Step 1 · 00:01", None)]);
    assert_eq!(timer(&brew), running(0, "00:01"));
    assert_eq!(brew.tick(&SETTINGS), ended());
    Ok(())
}

#[test]
fn ticks_while_not_running_do_nothing() -> TestResult {
    let mut empty = BrewController::new();
    assert_eq!(empty.tick(&SETTINGS), []);
    assert_eq!(empty.state(), &BrewState::Empty);

    let (mut brew, _catalog) = opened(&[1])?;
    let idle = brew.state().clone();
    assert_eq!(brew.tick(&SETTINGS), []);
    assert_eq!(brew.state(), &idle);

    brew.start(0)?;
    brew.tick(&SETTINGS);
    let ended = brew.state().clone();
    assert_ne!(ended, idle, "an ended brew is finished, not idle");
    assert_eq!(brew.tick(&SETTINGS), []);
    assert_eq!(brew.state(), &ended);
    Ok(())
}

// reset and leave

#[test]
fn reset_mid_brew_stops_and_turns_screen_on_off() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.tick(&SETTINGS);

    assert_eq!(brew.reset(), STOPPED);
    assert!(is_idle(&brew));
    assert_eq!(statuses(&brew), [Pending, Pending]);
    assert_eq!(brew.tick(&SETTINGS), []);
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
    brew.tick(&SETTINGS);

    assert_eq!(brew.leave(), STOPPED, "no vibration, no sound");
    assert_eq!(brew.state(), &BrewState::Empty);
    assert_eq!(brew.tick(&SETTINGS), []);
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
    brew.tick(&SETTINGS);

    let edited = dosed("a", 10, 160, &[10])?.renamed(RecipeName::try_parse("Ristretto")?);
    let change = catalog.upsert(edited)?;

    assert_eq!(brew.on_catalog_change(&change, &catalog), STOPPED);
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
    brew.tick(&SETTINGS);
    let before = brew.state().clone();

    let change = catalog.upsert(recipe("b", &[9, 9])?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &before);

    let change = catalog.upsert(recipe("c", &[1])?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &before);

    assert_eq!(
        brew.tick(&SETTINGS),
        [show("Step 1 · 00:01", Some("Step 2"))],
        "still running"
    );
    assert_eq!(timer(&brew), running(0, "00:01"));
    assert_eq!(
        brew.tick(&SETTINGS),
        [BUZZ, Effect::PlayStepSound, show("Step 2 · 00:02", None)]
    );
    Ok(())
}

#[test]
fn a_save_that_leaves_the_open_recipe_as_shown_changes_nothing() -> TestResult {
    let (mut brew, mut catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.tick(&SETTINGS);
    let before = brew.state().clone();

    let change = catalog.upsert(recipe("a", &[3, 2])?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &before, "still running, same step");
    Ok(())
}

#[test]
fn deleting_the_open_recipe_mid_brew_shows_empty_recipe() -> TestResult {
    let (mut brew, mut catalog) = opened(&[3, 2])?;
    brew.start(0)?;

    let change = catalog.remove(id("a")?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), STOPPED);
    assert_eq!(brew.state(), &BrewState::Empty);
    assert_eq!(brew.tick(&SETTINGS), []);
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

/// The running step's start number.
fn run(brew: &BrewController) -> Option<u64> {
    brew.state().running().map(|step| step.run())
}

#[test]
fn a_restart_of_the_same_step_is_a_new_run() -> TestResult {
    let (mut brew, _catalog) = opened(&[10, 10])?;
    let _effects = brew.start(0)?;
    let first = run(&brew);
    let _effects = brew.start(0)?;
    assert_eq!(timer(&brew), running(0, "00:10"));
    assert_ne!(run(&brew), first);
    assert_eq!(run(&brew), first.map(|r| r + 1));
    Ok(())
}

#[test]
fn ticks_and_step_changes_keep_the_run() -> TestResult {
    let (mut brew, _catalog) = opened(&[1, 10])?;
    let _effects = brew.start(0)?;
    let first = run(&brew);
    let _effects = brew.tick(&SETTINGS);
    assert_eq!(timer(&brew), running(1, "00:10"));
    assert_eq!(run(&brew), first);
    let _effects = brew.tick(&SETTINGS);
    assert_eq!(run(&brew), first);
    Ok(())
}

#[test]
fn a_rejected_start_does_not_count_a_run() -> TestResult {
    let (mut brew, _catalog) = opened(&[10])?;
    let _effects = brew.start(0)?;
    let first = run(&brew);
    assert!(brew.start(5).is_err());
    assert_eq!(run(&brew), first);
    let _effects = brew.start(0)?;
    assert_eq!(run(&brew), first.map(|r| r + 1));
    Ok(())
}

// ratio

/// `(coffee, water, ratio)` labels of the open recipe.
fn dose(brew: &BrewController) -> Option<(String, String, String)> {
    brew.state().view().map(|view| {
        (
            view.coffee_label().to_owned(),
            view.water_label().to_owned(),
            view.ratio_label().to_owned(),
        )
    })
}

fn shown(coffee: &str, water: &str, ratio: &str) -> Option<(String, String, String)> {
    Some((coffee.to_owned(), water.to_owned(), ratio.to_owned()))
}

fn ratio_setting(brew: &BrewController) -> Option<RatioSetting> {
    brew.state().view().map(BrewView::ratio)
}

#[test]
fn open_shows_the_recipes_own_ratio() -> TestResult {
    let (brew, _catalog) = opened(&[30, 60, 5])?;
    assert_eq!(dose(&brew), shown("20g", "100g", "1:5"));
    assert!(matches!(
        ratio_setting(&brew),
        Some(RatioSetting::Recipe(_))
    ));
    Ok(())
}

#[test]
fn more_or_less_water_keeps_the_coffee_and_moves_the_water() -> TestResult {
    let (mut brew, _catalog) = opened(&[30, 60, 5])?;

    brew.adjust_ratio(RatioChange::MoreWater)?;
    assert_eq!(dose(&brew), shown("20g", "110g", "1:5.5"));
    assert!(matches!(
        ratio_setting(&brew),
        Some(RatioSetting::Adjusted(_))
    ));

    brew.adjust_ratio(RatioChange::LessWater)?;
    brew.adjust_ratio(RatioChange::LessWater)?;
    assert_eq!(dose(&brew), shown("20g", "90g", "1:4.5"));
    Ok(())
}

#[test]
fn stepping_back_to_the_recipes_ratio_is_no_adjustment() -> TestResult {
    let (mut brew, _catalog) = opened(&[30])?;
    brew.adjust_ratio(RatioChange::MoreWater)?;
    brew.adjust_ratio(RatioChange::LessWater)?;
    assert!(matches!(
        ratio_setting(&brew),
        Some(RatioSetting::Recipe(_))
    ));
    Ok(())
}

#[test]
fn back_to_the_recipe_restores_its_ratio() -> TestResult {
    let (mut brew, _catalog) = opened(&[30])?;
    brew.adjust_ratio(RatioChange::MoreWater)?;
    brew.back_to_recipe_dose()?;
    assert_eq!(dose(&brew), shown("20g", "100g", "1:5"));
    assert!(matches!(
        ratio_setting(&brew),
        Some(RatioSetting::Recipe(_))
    ));
    Ok(())
}

#[test]
fn the_ratio_stops_at_its_bounds() -> TestResult {
    let (mut brew, _catalog) = opened(&[30])?;
    for _ in 0..20 {
        brew.adjust_ratio(RatioChange::LessWater)?;
    }
    assert_eq!(dose(&brew), shown("20g", "20g", "1:1"));
    for _ in 0..100 {
        brew.adjust_ratio(RatioChange::MoreWater)?;
    }
    assert_eq!(dose(&brew), shown("20g", "600g", "1:30"));
    Ok(())
}

#[test]
fn adjusting_mid_brew_keeps_the_timer_running() -> TestResult {
    let (mut brew, _catalog) = opened(&[30, 10])?;
    brew.start(0)?;
    brew.tick(&SETTINGS);
    brew.adjust_ratio(RatioChange::MoreWater)?;
    assert_eq!(timer(&brew), running(0, "00:29"));
    assert_eq!(statuses(&brew), [Current, Pending]);
    assert_eq!(dose(&brew), shown("20g", "110g", "1:5.5"));
    Ok(())
}

#[test]
fn the_adjusted_ratio_is_for_this_brew_only() -> TestResult {
    let (mut brew, catalog) = opened(&[30])?;
    brew.adjust_ratio(RatioChange::MoreWater)?;
    brew.open(&id("a")?, &catalog);
    assert_eq!(dose(&brew), shown("20g", "100g", "1:5"));

    brew.adjust_ratio(RatioChange::MoreWater)?;
    brew.leave();
    brew.open(&id("a")?, &catalog);
    assert_eq!(dose(&brew), shown("20g", "100g", "1:5"));
    Ok(())
}

#[test]
fn an_edit_to_the_open_recipe_keeps_the_adjusted_ratio() -> TestResult {
    let (mut brew, mut catalog) = opened(&[30])?;
    brew.adjust_ratio(RatioChange::MoreWater)?;
    let change = catalog.upsert(dosed("a", 30, 150, &[30, 30])?)?;
    brew.on_catalog_change(&change, &catalog);
    assert_eq!(dose(&brew), shown("30g", "165g", "1:5.5"));
    Ok(())
}

#[test]
fn the_figures_are_the_recipes_stored_dose_not_its_step_count() -> TestResult {
    let catalog = catalog(vec![dosed("a", 18, 36, &[30, 25])?])?;
    let mut brew = BrewController::new();
    brew.open(&id("a")?, &catalog);
    assert_eq!(dose(&brew), shown("18g", "36g", "1:2"));
    brew.adjust_ratio(RatioChange::MoreWater)?;
    assert_eq!(dose(&brew), shown("18g", "45g", "1:2.5"));
    Ok(())
}

#[test]
fn a_recipe_without_steps_still_has_its_ratio() -> TestResult {
    let (mut brew, _catalog) = opened(&[])?;
    assert!(matches!(
        ratio_setting(&brew),
        Some(RatioSetting::Recipe(_))
    ));
    brew.adjust_ratio(RatioChange::LessWater)?;
    assert_eq!(dose(&brew), shown("20g", "90g", "1:4.5"));
    Ok(())
}

#[test]
fn adjusting_with_no_recipe_open_is_an_error() {
    let mut brew = BrewController::new();
    assert_eq!(
        brew.adjust_ratio(RatioChange::MoreWater),
        Err(BrewError::NoRecipe)
    );
    assert_eq!(brew.state(), &BrewState::Empty);
}

// the coffee for this brew

fn coffee_setting(brew: &BrewController) -> Option<CoffeeSetting> {
    brew.state().view().map(BrewView::coffee)
}

fn grams(coffee: i64) -> Result<CoffeeDose, ModelError> {
    CoffeeDose::try_parse(coffee)
}

#[test]
fn less_coffee_takes_the_water_down_at_the_recipes_ratio() -> TestResult {
    let (mut brew, _catalog) = opened(&[30])?;
    brew.set_coffee(grams(12)?)?;
    assert_eq!(dose(&brew), shown("12g", "60g", "1:5"));
    assert!(matches!(
        coffee_setting(&brew),
        Some(CoffeeSetting::Adjusted(_))
    ));
    assert!(matches!(
        ratio_setting(&brew),
        Some(RatioSetting::Recipe(_))
    ));
    Ok(())
}

#[test]
fn the_water_follows_the_recipes_exact_ratio_not_its_rounded_label() -> TestResult {
    let (mut brew, _catalog) = pour_over()?;
    // 15 g to 250 g reads 1:16.7. Doubling the coffee doubles the water,
    // 500 g; the rounded label would make it 30 × 16.7 = 501 g.
    brew.set_coffee(grams(30)?)?;
    assert_eq!(dose(&brew), shown("30g", "500g", "1:16.7"));
    assert_eq!(
        pour_labels(&brew),
        [None, Some("100g".to_owned()), Some("400g".to_owned())]
    );
    let view = brew.state().view().ok_or("no view")?;
    assert_eq!(view.pour_to_label(2).as_deref(), Some("Pour to 500g"));
    Ok(())
}

#[test]
fn coming_back_to_the_recipes_coffee_is_no_adjustment() -> TestResult {
    let (mut brew, _catalog) = opened(&[30])?;
    brew.set_coffee(grams(12)?)?;
    brew.set_coffee(grams(20)?)?;
    assert_eq!(dose(&brew), shown("20g", "100g", "1:5"));
    assert!(matches!(
        coffee_setting(&brew),
        Some(CoffeeSetting::Recipe(_))
    ));
    Ok(())
}

#[test]
fn a_changed_coffee_and_a_changed_ratio_combine() -> TestResult {
    let (mut brew, _catalog) = opened(&[30])?;
    brew.set_coffee(grams(10)?)?;
    brew.adjust_ratio(RatioChange::MoreWater)?;
    assert_eq!(
        dose(&brew),
        shown("10g", "55g", "1:5.5"),
        "the coffee stays"
    );
    brew.set_coffee(grams(20)?)?;
    assert_eq!(
        dose(&brew),
        shown("20g", "110g", "1:5.5"),
        "the ratio stays"
    );
    Ok(())
}

#[test]
fn back_to_the_recipe_dose_restores_its_coffee_and_ratio() -> TestResult {
    let (mut brew, _catalog) = opened(&[30])?;
    brew.set_coffee(grams(10)?)?;
    brew.adjust_ratio(RatioChange::MoreWater)?;
    brew.back_to_recipe_dose()?;
    assert_eq!(dose(&brew), shown("20g", "100g", "1:5"));
    let view = brew.state().view().ok_or("no view")?;
    assert!(!view.is_adjusted());
    Ok(())
}

#[test]
fn changing_the_coffee_mid_brew_keeps_the_timer_running() -> TestResult {
    let (mut brew, _catalog) = opened(&[30, 10])?;
    brew.start(0)?;
    brew.tick(&SETTINGS);
    brew.set_coffee(grams(10)?)?;
    assert_eq!(timer(&brew), running(0, "00:29"));
    assert_eq!(dose(&brew), shown("10g", "50g", "1:5"));
    Ok(())
}

#[test]
fn the_changed_coffee_is_for_this_brew_only() -> TestResult {
    let (mut brew, catalog) = opened(&[30])?;
    brew.set_coffee(grams(10)?)?;
    brew.leave();
    brew.open(&id("a")?, &catalog);
    assert_eq!(dose(&brew), shown("20g", "100g", "1:5"));
    Ok(())
}

#[test]
fn an_edit_to_the_open_recipe_keeps_the_changed_coffee() -> TestResult {
    let (mut brew, mut catalog) = opened(&[30])?;
    brew.set_coffee(grams(10)?)?;
    let change = catalog.upsert(dosed("a", 30, 150, &[30, 30])?)?;
    brew.on_catalog_change(&change, &catalog);
    assert_eq!(dose(&brew), shown("10g", "50g", "1:5"));
    Ok(())
}

#[test]
fn changing_the_coffee_with_no_recipe_open_is_an_error() -> TestResult {
    let mut brew = BrewController::new();
    assert_eq!(brew.set_coffee(grams(10)?), Err(BrewError::NoRecipe));
    assert_eq!(brew.back_to_recipe_dose(), Err(BrewError::NoRecipe));
    assert_eq!(brew.state(), &BrewState::Empty);
    Ok(())
}

// pause and resume

/// `(held step index, remaining MM:SS)` while paused; `None` otherwise.
fn held(brew: &BrewController) -> Option<(usize, String)> {
    brew.state()
        .paused()
        .map(|step| (step.index(), step.remaining_label()))
}

#[test]
fn pause_holds_the_step_and_its_time_and_lets_the_screen_sleep() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.tick(&SETTINGS);

    assert_eq!(
        brew.pause()?,
        [SCREEN_OFF, show("Step 1 · 00:02 · Paused", Some("Step 2"))]
    );
    assert!(!brew.state().is_running());
    assert_eq!(timer(&brew), None);
    assert_eq!(held(&brew), running(0, "00:02"));
    assert_eq!(statuses(&brew), [StepStatus::Paused, Pending]);
    Ok(())
}

#[test]
fn ticks_while_paused_do_nothing() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.pause()?;
    let before = brew.state().clone();
    assert_eq!(brew.tick(&SETTINGS), []);
    assert_eq!(brew.state(), &before);
    Ok(())
}

#[test]
fn resume_counts_down_from_where_it_was_held_in_the_same_run() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.tick(&SETTINGS);
    let first = run(&brew);
    brew.pause()?;

    assert_eq!(
        brew.resume()?,
        [SCREEN_ON, show("Step 1 · 00:02", Some("Step 2"))]
    );
    assert_eq!(timer(&brew), running(0, "00:02"));
    assert_eq!(held(&brew), None);
    assert_eq!(run(&brew), first, "resuming is not a new run");
    assert_eq!(statuses(&brew), [Current, Pending]);

    brew.tick(&SETTINGS);
    assert_eq!(
        brew.tick(&SETTINGS),
        [BUZZ, Effect::PlayStepSound, show("Step 2 · 00:02", None)]
    );
    Ok(())
}

#[test]
fn pause_unless_running_is_an_error_and_changes_nothing() -> TestResult {
    let mut empty = BrewController::new();
    assert_eq!(empty.pause(), Err(BrewError::NotRunning));
    assert_eq!(empty.state(), &BrewState::Empty);

    let (mut brew, _catalog) = opened(&[3])?;
    assert_eq!(brew.pause(), Err(BrewError::NotRunning));
    assert!(is_idle(&brew));

    brew.start(0)?;
    brew.pause()?;
    let before = brew.state().clone();
    assert_eq!(brew.pause(), Err(BrewError::NotRunning));
    assert_eq!(brew.state(), &before);
    Ok(())
}

#[test]
fn resume_unless_paused_is_an_error_and_changes_nothing() -> TestResult {
    let (mut brew, _catalog) = opened(&[3])?;
    assert_eq!(brew.resume(), Err(BrewError::NotPaused));
    assert!(is_idle(&brew));

    brew.start(0)?;
    let before = brew.state().clone();
    assert_eq!(brew.resume(), Err(BrewError::NotPaused));
    assert_eq!(brew.state(), &before);
    Ok(())
}

#[test]
fn reset_while_paused_goes_idle_and_hides_the_notification() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.pause()?;
    assert_eq!(brew.reset(), [HIDE], "the screen already sleeps");
    assert!(is_idle(&brew));
    assert_eq!(statuses(&brew), [Pending, Pending]);
    Ok(())
}

#[test]
fn leave_or_open_while_paused_stops_silently() -> TestResult {
    let (mut brew, catalog) = opened(&[3])?;
    brew.start(0)?;
    brew.pause()?;
    assert_eq!(brew.open(&id("b")?, &catalog), [HIDE]);
    assert!(is_idle(&brew));

    brew.start(0)?;
    brew.pause()?;
    assert_eq!(brew.leave(), [HIDE]);
    assert_eq!(brew.state(), &BrewState::Empty);
    Ok(())
}

#[test]
fn start_while_paused_restarts_from_the_chosen_step_as_a_new_run() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.pause()?;
    let first = brew.state().paused().map(|step| step.run());

    assert_eq!(brew.start(1)?, [SCREEN_ON, show("Step 2 · 00:02", None)]);
    assert_eq!(timer(&brew), running(1, "00:02"));
    assert_eq!(run(&brew), first.map(|r| r + 1));
    Ok(())
}

#[test]
fn adjusting_while_paused_keeps_it_paused() -> TestResult {
    let (mut brew, _catalog) = opened(&[30, 10])?;
    brew.start(0)?;
    brew.tick(&SETTINGS);
    brew.pause()?;
    brew.adjust_ratio(RatioChange::MoreWater)?;
    assert_eq!(held(&brew), running(0, "00:29"));
    assert_eq!(dose(&brew), shown("20g", "110g", "1:5.5"));
    Ok(())
}

#[test]
fn an_edit_to_the_open_recipe_while_paused_resets_to_the_new_version() -> TestResult {
    let (mut brew, mut catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.pause()?;
    brew.adjust_ratio(RatioChange::MoreWater)?;

    let change = catalog.upsert(dosed("a", 10, 160, &[10])?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), [HIDE]);
    assert!(is_idle(&brew));
    assert_eq!(statuses(&brew), [Pending]);
    assert_eq!(dose(&brew), shown("10g", "55g", "1:5.5"));
    Ok(())
}

#[test]
fn deleting_the_open_recipe_while_paused_shows_empty_recipe() -> TestResult {
    let (mut brew, mut catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.pause()?;
    let change = catalog.remove(id("a")?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), [HIDE]);
    assert_eq!(brew.state(), &BrewState::Empty);
    Ok(())
}

#[test]
fn a_change_to_another_recipe_leaves_a_paused_brew_alone() -> TestResult {
    let (mut brew, mut catalog) = opened(&[3])?;
    brew.start(0)?;
    brew.pause()?;
    let before = brew.state().clone();
    let change = catalog.remove(id("b")?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &before);
    Ok(())
}

// step pours

/// Recipe `p`, 15 g coffee to 250 g water: Grind 5 s (no pour), Bloom 2 s
/// pouring 50 g, Pour 3 s pouring 200 g.
fn pour_over() -> Result<(BrewController, RecipeCatalog), Box<dyn std::error::Error>> {
    let step = |i: usize, name: &str, seconds: i64, water: Option<i64>| {
        Ok::<_, ModelError>(RecipeStep::new(
            StepId::try_parse(&format!("p-s{i}"))?,
            name.to_owned(),
            StepDuration::try_parse(seconds)?,
            water.map(WaterAmount::try_parse).transpose()?,
        ))
    };
    let recipe = Recipe::new(
        id("p")?,
        RecipeName::try_parse("Pour Over")?,
        String::new(),
        CoffeeDose::try_parse(15)?,
        WaterAmount::try_parse(250)?,
        vec![
            step(0, "Grind", 5, None)?,
            step(1, "Bloom", 2, Some(50))?,
            step(2, "Pour", 3, Some(200))?,
        ],
    );
    let catalog = catalog(vec![recipe])?;
    let mut brew = BrewController::new();
    let _effects = brew.open(&id("p")?, &catalog);
    Ok((brew, catalog))
}

#[test]
fn pinning_the_open_recipe_leaves_a_running_brew_alone() -> TestResult {
    let (mut brew, mut catalog) = pour_over()?;
    let _effects = brew.start(0)?;
    let _effects = brew.tick(&SETTINGS);
    let before = brew.state().clone();

    let change = catalog.set_pin(&id("p")?, Pin::Pinned)?;

    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &before, "the recipe it brews is unchanged");
    Ok(())
}

fn pour_labels(brew: &BrewController) -> Vec<Option<String>> {
    brew.state()
        .step_rows()
        .into_iter()
        .map(|row| row.pour_label)
        .collect()
}

/// The "pour to" target of the step the timer is on, running or paused.
fn pour_to(brew: &BrewController) -> Option<String> {
    let step = brew.state().running().or(brew.state().paused())?;
    brew.state().view()?.pour_to_label(step.index())
}

#[test]
fn the_step_list_shows_each_pour() -> TestResult {
    let (brew, _catalog) = pour_over()?;
    assert_eq!(
        pour_labels(&brew),
        [None, Some("50g".to_owned()), Some("200g".to_owned())]
    );
    Ok(())
}

#[test]
fn the_current_step_shows_the_running_total_to_pour_to() -> TestResult {
    let (mut brew, _catalog) = pour_over()?;
    let _effects = brew.start(0)?;
    assert_eq!(pour_to(&brew), None, "grinding pours nothing");
    for _ in 0..5 {
        let _effects = brew.tick(&SETTINGS);
    }
    assert_eq!(timer(&brew), running(1, "00:02"));
    assert_eq!(pour_to(&brew).as_deref(), Some("Pour to 50g"));
    let _effects = brew.pause()?;
    assert_eq!(pour_to(&brew).as_deref(), Some("Pour to 50g"), "paused");
    let _effects = brew.resume()?;
    for _ in 0..2 {
        let _effects = brew.tick(&SETTINGS);
    }
    assert_eq!(timer(&brew), running(2, "00:03"));
    assert_eq!(pour_to(&brew).as_deref(), Some("Pour to 250g"));
    Ok(())
}

#[test]
fn an_adjusted_ratio_scales_the_pours_and_their_targets() -> TestResult {
    let (mut brew, _catalog) = pour_over()?;
    // 1:16.7 -> 1:17: 15 g of coffee to 255 g of water, every pour × 1.02.
    brew.adjust_ratio(RatioChange::MoreWater)?;
    let view = brew.state().view().ok_or("no view")?;
    assert_eq!(view.water_label(), "255g");
    assert_eq!(
        pour_labels(&brew),
        [None, Some("51g".to_owned()), Some("204g".to_owned())]
    );
    assert_eq!(view.pour_to_label(1).as_deref(), Some("Pour to 51g"));
    assert_eq!(view.pour_to_label(2).as_deref(), Some("Pour to 255g"));
    brew.back_to_recipe_dose()?;
    assert_eq!(
        pour_labels(&brew),
        [None, Some("50g".to_owned()), Some("200g".to_owned())]
    );
    Ok(())
}

// finished

/// Recipe `a` (`step_seconds`), run from step `from` to its end.
fn brewed(
    step_seconds: &[i64],
    from: usize,
) -> Result<(BrewController, RecipeCatalog), Box<dyn std::error::Error>> {
    let (mut brew, catalog) = opened(step_seconds)?;
    let _effects = brew.start(from)?;
    while brew.state().is_running() {
        let _effects = brew.tick(&SETTINGS);
    }
    Ok((brew, catalog))
}

#[test]
fn the_last_tick_finishes_with_the_finish_buzz_and_the_sound() -> TestResult {
    let (mut brew, _catalog) = opened(&[1])?;
    brew.start(0)?;
    assert_eq!(
        brew.tick(&SETTINGS),
        [SCREEN_OFF, finish_buzz(), Effect::PlayStepSound, HIDE,]
    );
    assert!(!brew.state().is_running());
    assert_eq!(finished(&brew), Some("0:01"));
    assert_eq!(statuses(&brew), [Done]);
    Ok(())
}

#[test]
fn finished_shows_the_time_brewed_from_the_step_it_started_at() -> TestResult {
    let (brew, _catalog) = brewed(&[30, 60, 5], 0)?;
    assert_eq!(finished(&brew), Some("1:35"));
    assert_eq!(statuses(&brew), [Done, Done, Done]);

    let (brew, _catalog) = brewed(&[30, 60, 5], 1)?;
    assert_eq!(finished(&brew), Some("1:05"), "the first step was skipped");
    assert_eq!(statuses(&brew), [Done, Done, Done]);
    Ok(())
}

#[test]
fn finished_neither_ticks_nor_pauses_nor_resumes() -> TestResult {
    let (mut brew, _catalog) = brewed(&[2], 0)?;
    let before = brew.state().clone();
    assert_eq!(brew.tick(&SETTINGS), []);
    assert_eq!(brew.pause(), Err(BrewError::NotRunning));
    assert_eq!(brew.resume(), Err(BrewError::NotPaused));
    assert_eq!(brew.state(), &before);
    Ok(())
}

#[test]
fn reset_while_finished_goes_idle_with_no_effects() -> TestResult {
    let (mut brew, _catalog) = brewed(&[2, 1], 0)?;
    assert_eq!(brew.reset(), []);
    assert!(is_idle(&brew));
    assert_eq!(statuses(&brew), [Pending, Pending]);
    Ok(())
}

#[test]
fn start_while_finished_brews_again_as_a_new_run() -> TestResult {
    let (mut brew, _catalog) = brewed(&[2, 1], 0)?;
    assert_eq!(
        brew.start(0)?,
        [SCREEN_ON, show("Step 1 · 00:02", Some("Step 2"))]
    );
    assert_eq!(timer(&brew), running(0, "00:02"));
    assert_eq!(run(&brew), Some(2));
    assert_eq!(statuses(&brew), [Current, Pending]);

    let (mut brew, _catalog) = brewed(&[2, 1], 0)?;
    assert_eq!(brew.start(1)?, [SCREEN_ON, show("Step 2 · 00:01", None)]);
    assert_eq!(statuses(&brew), [Done, Current]);
    Ok(())
}

#[test]
fn leave_or_open_while_finished_has_no_effects() -> TestResult {
    let (mut brew, catalog) = brewed(&[1], 0)?;
    assert_eq!(brew.open(&id("b")?, &catalog), []);
    assert!(is_idle(&brew));

    let (mut brew, _catalog) = brewed(&[1], 0)?;
    assert_eq!(brew.leave(), []);
    assert_eq!(brew.state(), &BrewState::Empty);
    Ok(())
}

#[test]
fn an_edit_to_the_open_recipe_while_finished_shows_the_new_version_idle() -> TestResult {
    let (mut brew, mut catalog) = brewed(&[1], 0)?;
    let change = catalog.upsert(recipe("a", &[3, 4])?.described_as("New".to_owned()))?;

    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert!(is_idle(&brew));
    let view = brew.state().view().ok_or("no view")?;
    assert_eq!(view.description(), "New");
    assert_eq!(statuses(&brew), [Pending, Pending]);
    Ok(())
}

#[test]
fn deleting_the_open_recipe_while_finished_shows_empty_recipe() -> TestResult {
    let (mut brew, mut catalog) = brewed(&[1], 0)?;
    let change = catalog.remove(id("a")?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &BrewState::Empty);
    Ok(())
}

#[test]
fn a_change_to_another_recipe_leaves_a_finished_brew_alone() -> TestResult {
    let (mut brew, mut catalog) = brewed(&[1], 0)?;
    let before = brew.state().clone();
    let change = catalog.upsert(recipe("b", &[9])?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &before);
    Ok(())
}

#[test]
fn finished_shows_what_the_scale_reads_at_the_ratio_shown() -> TestResult {
    let (mut brew, _catalog) = pour_over()?;
    let _effects = brew.start(0)?;
    while brew.state().is_running() {
        let _effects = brew.tick(&SETTINGS);
    }
    let poured = |brew: &BrewController| brew.state().view().and_then(BrewView::poured_label);
    assert_eq!(finished(&brew), Some("0:10"));
    assert_eq!(poured(&brew).as_deref(), Some("Poured to 250g"));

    // A ratio change keeps the brew finished, like it keeps idle idle.
    brew.adjust_ratio(RatioChange::MoreWater)?;
    assert_eq!(finished(&brew), Some("0:10"));
    assert_eq!(poured(&brew).as_deref(), Some("Poured to 255g"));
    assert_eq!(statuses(&brew), [Done, Done, Done]);
    Ok(())
}

#[test]
fn a_recipe_that_pours_nothing_has_no_poured_label() -> TestResult {
    let (brew, _catalog) = brewed(&[1], 0)?;
    let view = brew.state().view().ok_or("no view")?;
    assert_eq!(view.poured_label(), None);
    Ok(())
}

// pre-alert

/// The pre-alert: two quick taps a few seconds before a step ends, unlike
/// a step boundary's single buzz.
fn pre_alert() -> Effect {
    Effect::VibratePattern {
        pattern_ms: vec![0, 50, 100, 50],
    }
}

/// Ticks a running brew to its end; the numbers of the ticks (from 1) that
/// pre-alerted, and the effects of the last tick.
fn pre_alert_ticks(brew: &mut BrewController) -> (Vec<u32>, Vec<Effect>) {
    pre_alert_ticks_with(brew, &SETTINGS)
}

/// [`pre_alert_ticks`] under `settings`.
fn pre_alert_ticks_with(brew: &mut BrewController, settings: &Settings) -> (Vec<u32>, Vec<Effect>) {
    let mut ticks = Vec::new();
    let mut last = Vec::new();
    let mut tick = 0;
    while brew.state().is_running() {
        tick += 1;
        last = brew.tick(settings);
        let alerts = last.iter().filter(|&effect| *effect == pre_alert()).count();
        assert!(alerts <= 1, "tick {tick} pre-alerted {alerts} times");
        if alerts == 1 {
            ticks.push(tick);
        }
    }
    (ticks, last)
}

#[test]
fn the_default_lead_is_three_seconds() {
    assert_eq!(SETTINGS.pre_alert(), PreAlertLead::Seconds3);
    assert_eq!(CuePolicy::PRE_ALERT_PATTERN_MS, [0, 50, 100, 50]);
}

#[test]
fn the_pre_alert_fires_once_on_the_tick_that_leaves_three_seconds() -> TestResult {
    let (mut brew, _catalog) = opened(&[5, 5])?;
    brew.start(0)?;
    assert_eq!(
        brew.tick(&SETTINGS),
        [show("Step 1 · 00:04", Some("Step 2"))]
    );
    assert_eq!(
        brew.tick(&SETTINGS),
        [pre_alert(), show("Step 1 · 00:03", Some("Step 2"))],
        "the buzz comes before the notification, which stays last"
    );
    assert_eq!(
        brew.tick(&SETTINGS),
        [show("Step 1 · 00:02", Some("Step 2"))]
    );
    Ok(())
}

#[test]
fn each_long_enough_step_pre_alerts_once_and_the_end_only_finishes() -> TestResult {
    let (mut brew, _catalog) = opened(&[5, 5])?;
    brew.start(0)?;
    let (ticks, last) = pre_alert_ticks(&mut brew);
    // The last step pre-alerts too, 3 s before the end; the finish buzz
    // fires alone at the end, so the two never stack.
    assert_eq!(ticks, [2, 7]);
    assert_eq!(last, ended());
    Ok(())
}

#[test]
fn a_step_no_longer_than_the_lead_gets_no_pre_alert() -> TestResult {
    // 3 s and 2 s and 0 s are too short; the 4 s step pre-alerts once its
    // first second has passed: tick 3 + 2 + 1 (the 0 s step) + 1.
    let (mut brew, _catalog) = opened(&[3, 2, 0, 4])?;
    brew.start(0)?;
    let (ticks, last) = pre_alert_ticks(&mut brew);
    assert_eq!(ticks, [7]);
    assert_eq!(last, ended());
    Ok(())
}

#[test]
fn pausing_after_the_pre_alert_does_not_repeat_it() -> TestResult {
    let (mut brew, _catalog) = opened(&[5])?;
    brew.start(0)?;
    brew.tick(&SETTINGS);
    assert!(brew.tick(&SETTINGS).contains(&pre_alert()));

    let paused = brew.pause()?;
    let resumed = brew.resume()?;
    assert!(!paused.contains(&pre_alert()) && !resumed.contains(&pre_alert()));
    let (ticks, last) = pre_alert_ticks(&mut brew);
    assert_eq!(ticks, [] as [u32; 0], "already given before the pause");
    assert_eq!(last, ended());
    Ok(())
}

#[test]
fn pausing_just_before_the_pre_alert_gives_it_once_after_resuming() -> TestResult {
    let (mut brew, _catalog) = opened(&[5])?;
    brew.start(0)?;
    assert!(!brew.tick(&SETTINGS).contains(&pre_alert()));
    brew.pause()?;
    assert_eq!(brew.tick(&SETTINGS), [], "nothing ticks while paused");
    brew.resume()?;
    brew.pause()?;
    brew.resume()?;

    let (ticks, _last) = pre_alert_ticks(&mut brew);
    assert_eq!(ticks, [1], "the first tick after resuming leaves 3 s");
    Ok(())
}

#[test]
fn a_brew_started_from_a_middle_step_pre_alerts_from_there() -> TestResult {
    let (mut brew, _catalog) = opened(&[5, 6, 2])?;
    brew.start(1)?;
    let (ticks, last) = pre_alert_ticks(&mut brew);
    assert_eq!(
        ticks,
        [3],
        "6 s step at 3 s left; the 2 s step is too short"
    );
    assert_eq!(last, ended());
    Ok(())
}

#[test]
fn a_restart_gives_the_restarted_step_its_pre_alert_again() -> TestResult {
    let (mut brew, _catalog) = opened(&[5])?;
    brew.start(0)?;
    brew.tick(&SETTINGS);
    brew.tick(&SETTINGS);
    brew.start(0)?;
    let (ticks, _last) = pre_alert_ticks(&mut brew);
    assert_eq!(ticks, [2], "a new run counts the step down again");
    Ok(())
}

#[test]
fn the_pre_alert_setting_sets_the_lead_or_turns_it_off() -> TestResult {
    let catalog = catalog(vec![recipe("a", &[6, 1])?, recipe("b", &[11, 1])?])?;
    let brewed_with =
        |recipe: &str, lead: PreAlertLead| -> Result<Vec<u32>, Box<dyn std::error::Error>> {
            let mut brew = BrewController::new();
            brew.open(&id(recipe)?, &catalog);
            brew.start(0)?;
            Ok(pre_alert_ticks_with(&mut brew, &SETTINGS.with_pre_alert(lead)).0)
        };
    assert_eq!(brewed_with("a", PreAlertLead::Seconds3)?, [3]);
    assert_eq!(brewed_with("a", PreAlertLead::Seconds5)?, [1]);
    assert_eq!(
        brewed_with("a", PreAlertLead::Seconds10)?,
        [] as [u32; 0],
        "the step is shorter than the lead"
    );
    assert_eq!(brewed_with("b", PreAlertLead::Seconds10)?, [1]);
    assert_eq!(brewed_with("a", PreAlertLead::Off)?, [] as [u32; 0], "off");
    assert_eq!(brewed_with("b", PreAlertLead::Off)?, [] as [u32; 0], "off");
    Ok(())
}

// sound and vibration settings

const SOUND: Effect = Effect::PlayStepSound;

/// Whether `effect` buzzes.
fn buzzes(effect: &Effect) -> bool {
    matches!(
        effect,
        Effect::Vibrate { .. } | Effect::VibratePattern { .. }
    )
}

/// Every effect of a whole brew of `[5, 5]` under `settings`, tick by
/// tick.
fn brew_effects(settings: &Settings) -> Result<Vec<Vec<Effect>>, Box<dyn std::error::Error>> {
    let (mut brew, _catalog) = opened(&[5, 5])?;
    brew.start(0)?;
    let mut ticks = Vec::new();
    while brew.state().is_running() {
        ticks.push(brew.tick(settings));
    }
    Ok(ticks)
}

#[test]
fn every_sound_and_vibration_combination_gives_only_its_cues() -> TestResult {
    let notify = |title: &str, next: Option<&str>| show(title, next);
    for sound in Toggle::ALL {
        for vibration in Toggle::ALL {
            for lead in PreAlertLead::ALL {
                let settings = SETTINGS
                    .with_sound(sound)
                    .with_vibration(vibration)
                    .with_pre_alert(lead);
                let ticks = brew_effects(&settings)?;
                let all: Vec<&Effect> = ticks.iter().flatten().collect();
                let case = format!("sound {sound:?}, vibration {vibration:?}, {lead:?}");
                assert_eq!(
                    all.iter().any(|effect| **effect == SOUND),
                    sound.is_on(),
                    "{case}: sound"
                );
                assert_eq!(
                    all.iter().any(|effect| buzzes(effect)),
                    vibration.is_on(),
                    "{case}: buzz"
                );
                assert_eq!(
                    all.contains(&&pre_alert()),
                    vibration.is_on() && lead == PreAlertLead::Seconds3,
                    "{case}: pre-alert (only 3 s fits a 5 s step)"
                );

                let buzz = vibration.is_on().then_some(BUZZ);
                let sounded = sound.is_on().then_some(SOUND);
                let boundary: Vec<Effect> = buzz
                    .into_iter()
                    .chain(sounded.clone())
                    .chain([notify("Step 2 · 00:05", None)])
                    .collect();
                assert_eq!(ticks[4], boundary, "{case}: the step boundary");

                let finish_buzz = vibration.is_on().then(finish_buzz);
                let end: Vec<Effect> = [SCREEN_OFF]
                    .into_iter()
                    .chain(finish_buzz)
                    .chain(sounded)
                    .chain([HIDE])
                    .collect();
                assert_eq!(ticks.last(), Some(&end), "{case}: the finish");
            }
        }
    }
    Ok(())
}

#[test]
fn the_default_settings_give_every_cue() -> TestResult {
    let ticks = brew_effects(&SETTINGS)?;
    assert_eq!(
        ticks[1],
        [pre_alert(), show("Step 1 · 00:03", Some("Step 2"))]
    );
    assert_eq!(ticks[4], [BUZZ, SOUND, show("Step 2 · 00:05", None)]);
    assert_eq!(ticks.last().map(Vec::as_slice), Some(ended().as_slice()));
    Ok(())
}

#[test]
fn a_setting_changed_mid_brew_applies_from_the_next_tick() -> TestResult {
    let muted = SETTINGS.with_sound(Toggle::Off).with_vibration(Toggle::Off);
    let (mut brew, _catalog) = opened(&[2, 4, 1])?;
    brew.start(0)?;
    brew.tick(&SETTINGS);
    assert_eq!(
        brew.tick(&muted),
        [show("Step 2 · 00:04", Some("Step 3"))],
        "muted from this tick: a silent step boundary"
    );
    assert_eq!(brew.tick(&muted), [show("Step 2 · 00:03", Some("Step 3"))]);
    brew.tick(&SETTINGS);
    brew.tick(&SETTINGS);
    assert_eq!(
        brew.tick(&SETTINGS),
        [BUZZ, SOUND, show("Step 3 · 00:01", None)],
        "back on for the next boundary"
    );
    Ok(())
}

#[test]
fn muting_never_touches_the_screen_or_notification_effects() -> TestResult {
    let muted = SETTINGS.with_sound(Toggle::Off).with_vibration(Toggle::Off);
    let (mut brew, _catalog) = opened(&[1])?;
    assert_eq!(brew.start(0)?, [SCREEN_ON, show("Step 1 · 00:01", None)]);
    assert_eq!(brew.tick(&muted), [SCREEN_OFF, HIDE]);
    Ok(())
}

// next-step preview

/// What the page shows under the dial for the step the timer is on,
/// running or paused; none otherwise.
fn next_label(brew: &BrewController) -> Option<String> {
    let step = brew.state().running().or(brew.state().paused())?;
    Some(brew.state().view()?.next_label(step.index()))
}

#[test]
fn the_next_step_shows_its_name_and_duration() -> TestResult {
    let (mut brew, _catalog) = opened(&[30, 75, 5])?;
    assert_eq!(next_label(&brew), None, "idle has no timer");
    brew.start(0)?;
    assert_eq!(next_label(&brew).as_deref(), Some("Next: Step 2 · 1:15"));
    brew.pause()?;
    assert_eq!(
        next_label(&brew).as_deref(),
        Some("Next: Step 2 · 1:15"),
        "paused"
    );
    brew.resume()?;
    for _ in 0..30 {
        brew.tick(&SETTINGS);
    }
    assert_eq!(next_label(&brew).as_deref(), Some("Next: Step 3 · 0:05"));
    Ok(())
}

#[test]
fn the_last_step_reads_last_step() -> TestResult {
    let (mut brew, _catalog) = opened(&[30, 5])?;
    brew.start(1)?;
    assert_eq!(next_label(&brew).as_deref(), Some("Last step"));
    assert_eq!(BrewView::LAST_STEP_LABEL, "Last step");
    while brew.state().is_running() {
        brew.tick(&SETTINGS);
    }
    assert_eq!(next_label(&brew), None, "finished has no timer");
    Ok(())
}

#[test]
fn a_pouring_next_step_shows_its_pour_at_the_ratio_shown() -> TestResult {
    let (mut brew, _catalog) = pour_over()?;
    brew.start(0)?;
    assert_eq!(
        next_label(&brew).as_deref(),
        Some("Next: Bloom · 0:02 · 50g")
    );
    brew.adjust_ratio(RatioChange::MoreWater)?;
    assert_eq!(
        next_label(&brew).as_deref(),
        Some("Next: Bloom · 0:02 · 51g")
    );
    brew.start(1)?;
    assert_eq!(
        next_label(&brew).as_deref(),
        Some("Next: Pour · 0:03 · 204g")
    );
    brew.start(2)?;
    assert_eq!(next_label(&brew).as_deref(), Some("Last step"));
    Ok(())
}

// saving the adjusted ratio

/// Each stored step pour of recipe `p` in `catalog`.
fn stored_pours(catalog: &RecipeCatalog) -> Result<Vec<Option<u32>>, ModelError> {
    Ok(catalog
        .get(&id("p")?)
        .map(|recipe| {
            recipe
                .steps()
                .iter()
                .map(|step| step.water().map(WaterAmount::grams))
                .collect()
        })
        .unwrap_or_default())
}

#[test]
fn saving_the_ratio_stores_the_water_and_pours_shown() -> TestResult {
    let (mut brew, mut catalog) = pour_over()?;
    brew.adjust_ratio(RatioChange::MoreWater)?;
    let pours_shown = pour_labels(&brew);
    assert_eq!(
        pours_shown,
        [None, Some("51g".to_owned()), Some("204g".to_owned())]
    );

    let change = brew.save_ratio(&mut catalog)?;
    assert_eq!(change, CatalogChange::Upserted(id("p")?));

    let saved = catalog.get(&id("p")?).ok_or("no recipe")?;
    assert_eq!(saved.coffee().grams(), 15, "the coffee stays");
    assert_eq!(saved.water().grams(), 255);
    assert_eq!(stored_pours(&catalog)?, [None, Some(51), Some(204)]);

    // The page shows the saved recipe at its own ratio, the same figures.
    let view = brew.state().view().ok_or("no view")?;
    assert!(matches!(view.ratio(), RatioSetting::Recipe(_)));
    assert_eq!(dose(&brew), shown("15g", "255g", "1:17"));
    assert_eq!(pour_labels(&brew), pours_shown);
    assert_eq!(view.pour_to_label(2).as_deref(), Some("Pour to 255g"));
    Ok(())
}

#[test]
fn saving_the_ratio_updates_the_recipe_list_row() -> TestResult {
    let (mut brew, mut catalog) = pour_over()?;
    brew.adjust_ratio(RatioChange::LessWater)?;
    let change = brew.save_ratio(&mut catalog)?;
    let CatalogChange::Upserted(saved) = &change else {
        return Err("expected an upsert".into());
    };
    let row = RecipeSummary::of(
        catalog.get(saved).ok_or("no recipe")?,
        Pin::Unpinned,
        &BrewMetrics::default(),
    );
    // 1:16.7 -> 1:16.5: 247.5 g rounds to 248 g.
    assert_eq!(row.water_label, "248 g");
    assert_eq!(catalog.index_of(saved), Some(0), "it keeps its place");
    Ok(())
}

#[test]
fn saving_the_ratio_keeps_a_running_brew_where_it_is() -> TestResult {
    let (mut brew, mut catalog) = pour_over()?;
    let _effects = brew.start(0)?;
    for _ in 0..6 {
        let _effects = brew.tick(&SETTINGS);
    }
    assert_eq!(timer(&brew), running(1, "00:01"));
    brew.adjust_ratio(RatioChange::MoreWater)?;

    let change = brew.save_ratio(&mut catalog)?;
    assert_eq!(timer(&brew), running(1, "00:01"), "still on Bloom");
    assert_eq!(run(&brew), Some(1), "the same run");
    assert_eq!(statuses(&brew), [Done, Current, Pending]);
    assert_eq!(pour_to(&brew).as_deref(), Some("Pour to 51g"));

    // The shell passes the change on; the page already shows it.
    let before = brew.state().clone();
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &before);

    assert_eq!(
        brew.tick(&SETTINGS),
        [BUZZ, Effect::PlayStepSound, show("Pour · 00:03", None)]
    );
    assert_eq!(pour_to(&brew).as_deref(), Some("Pour to 255g"));
    Ok(())
}

#[test]
fn saving_the_ratio_keeps_a_paused_or_finished_brew() -> TestResult {
    let (mut brew, mut catalog) = pour_over()?;
    let _effects = brew.start(1)?;
    let _effects = brew.pause()?;
    brew.adjust_ratio(RatioChange::MoreWater)?;
    let held_before = brew.state().paused().cloned();
    let change = brew.save_ratio(&mut catalog)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state().paused().cloned(), held_before);

    let (mut brew, mut catalog) = pour_over()?;
    let _effects = brew.start(0)?;
    while brew.state().is_running() {
        let _effects = brew.tick(&SETTINGS);
    }
    brew.adjust_ratio(RatioChange::MoreWater)?;
    let change = brew.save_ratio(&mut catalog)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(finished(&brew), Some("0:10"));
    assert_eq!(statuses(&brew), [Done, Done, Done]);
    let view = brew.state().view().ok_or("no view")?;
    assert_eq!(view.poured_label().as_deref(), Some("Poured to 255g"));
    Ok(())
}

#[test]
fn saving_more_water_than_a_recipe_holds_is_an_error() -> TestResult {
    // 100 g of coffee at 1:19 is 1900 g; two steps of 0.5 later, 2000 g.
    let catalog_recipes = vec![dosed("a", 100, 1900, &[5])?];
    let mut catalog = catalog(catalog_recipes.clone())?;
    let mut brew = BrewController::new();
    let _effects = brew.open(&id("a")?, &catalog);
    for _ in 0..3 {
        brew.adjust_ratio(RatioChange::MoreWater)?;
    }
    assert_eq!(dose(&brew), shown("100g", "2050g", "1:20.5"));
    let before = brew.state().clone();

    let error = brew
        .save_ratio(&mut catalog)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error,
        SaveRatioError::OutOfRange(WaterScaleError::Water { grams: 2050 })
    );
    assert_eq!(
        error.to_string(),
        "Can't save to the recipe: 2050 g of water is outside 1–2000 g"
    );
    assert_eq!(brew.state(), &before, "still adjusted");
    assert_eq!(catalog.state(), catalog_recipes.as_slice());

    brew.adjust_ratio(RatioChange::LessWater)?;
    brew.save_ratio(&mut catalog)?;
    assert_eq!(
        catalog.get(&id("a")?).map(|r| r.water().grams()),
        Some(2000),
        "2000 g still fits"
    );
    Ok(())
}

#[test]
fn saving_a_scaled_pour_out_of_range_is_an_error() -> TestResult {
    // The pours need not add up to the water: 2000 g poured of 250 g.
    let flood = Recipe::new(
        id("f")?,
        RecipeName::try_parse("Flood")?,
        String::new(),
        CoffeeDose::try_parse(15)?,
        WaterAmount::try_parse(250)?,
        vec![RecipeStep::new(
            StepId::try_parse("f-s0")?,
            "Flood".to_owned(),
            StepDuration::try_parse(5)?,
            Some(WaterAmount::try_parse(2000)?),
        )],
    );
    let mut catalog = catalog(vec![flood.clone()])?;
    let mut brew = BrewController::new();
    let _effects = brew.open(&id("f")?, &catalog);
    brew.adjust_ratio(RatioChange::MoreWater)?;
    assert_eq!(pour_labels(&brew), [Some("2040g".to_owned())]);

    assert_eq!(
        brew.save_ratio(&mut catalog),
        Err(SaveRatioError::OutOfRange(WaterScaleError::Pour {
            position: 1,
            grams: 2040
        }))
    );
    assert_eq!(catalog.get(&id("f")?), Some(&flood));
    Ok(())
}

#[test]
fn saving_a_changed_coffee_stores_the_coffee_water_and_pours_shown() -> TestResult {
    let (mut brew, mut catalog) = pour_over()?;
    brew.set_coffee(grams(9)?)?;
    let change = brew.save_ratio(&mut catalog)?;
    assert_eq!(change, CatalogChange::Upserted(id("p")?));

    let saved = catalog.get(&id("p")?).ok_or("no recipe")?;
    assert_eq!(saved.coffee().grams(), 9);
    assert_eq!(saved.water().grams(), 150);
    assert_eq!(stored_pours(&catalog)?, [None, Some(30), Some(120)]);

    let view = brew.state().view().ok_or("no view")?;
    assert!(!view.is_adjusted(), "the recipe's own dose now");
    assert_eq!(dose(&brew), shown("9g", "150g", "1:16.7"));
    Ok(())
}

#[test]
fn saving_coffee_that_needs_more_water_than_a_recipe_holds_is_an_error() -> TestResult {
    let (mut brew, mut catalog) = pour_over()?;
    brew.set_coffee(grams(200)?)?;
    assert_eq!(dose(&brew), shown("200g", "3333g", "1:16.7"));
    assert_eq!(
        brew.save_ratio(&mut catalog),
        Err(SaveRatioError::OutOfRange(WaterScaleError::Water {
            grams: 3333
        }))
    );
    assert_eq!(catalog.get(&id("p")?).map(|r| r.coffee().grams()), Some(15));
    Ok(())
}

#[test]
fn saving_the_recipes_own_ratio_or_none_is_an_error() -> TestResult {
    let (mut brew, mut catalog) = pour_over()?;
    let before = brew.state().clone();
    assert_eq!(
        brew.save_ratio(&mut catalog),
        Err(SaveRatioError::NotAdjusted)
    );
    assert_eq!(brew.state(), &before);

    let _effects = brew.leave();
    assert_eq!(brew.save_ratio(&mut catalog), Err(SaveRatioError::NoRecipe));
    Ok(())
}

#[test]
fn a_failed_ratio_save_changes_nothing() -> TestResult {
    let recipes = vec![dosed("a", 20, 100, &[5])?];
    let mut catalog = RecipeCatalog::new(Box::new(
        FakeRecipeStore::with_recipes(recipes.clone()).failing_writes(),
    ));
    catalog.load()?;
    let mut brew = BrewController::new();
    let _effects = brew.open(&id("a")?, &catalog);
    brew.adjust_ratio(RatioChange::MoreWater)?;
    let before = brew.state().clone();

    let error = brew
        .save_ratio(&mut catalog)
        .err()
        .ok_or("expected an error")?;
    assert!(matches!(error, SaveRatioError::Store(_)), "{error}");
    assert!(error.to_string().starts_with("Failed to save recipe: "));
    assert_eq!(brew.state(), &before);
    assert_eq!(catalog.state(), recipes.as_slice());
    Ok(())
}
