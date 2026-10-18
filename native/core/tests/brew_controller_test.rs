//! `BrewController`: the brew page and its step timer, tick by tick. Legacy
//! reference: `lib/core/recipe/actors/recipe_actor.dart` (timer) and
//! `lib/recipe_details/pages/recipe_details_page.dart` (what is shown).

use coffee_timer_core::brew::{
    BrewController, BrewError, BrewNotice, BrewState, BrewView, Effect, RatioChange, RatioSetting,
    StepStatus,
};
use coffee_timer_core::catalog::RecipeCatalog;
use coffee_timer_core::fakes::FakeRecipeStore;
use coffee_timer_core::model::{
    BrewMetrics, CoffeeDose, LabelStyle, ModelError, Recipe, RecipeId, RecipeName, RecipeStep,
    StepDuration, StepId, WaterAmount,
};

use StepStatus::{Current, Done, Pending};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const BUZZ: Effect = Effect::Vibrate { ms: 100 };
const SCREEN_ON: Effect = Effect::KeepScreenOn(true);
const SCREEN_OFF: Effect = Effect::KeepScreenOn(false);
const HIDE: Effect = Effect::HideBrewNotification;
/// A running brew stops silently.
const STOPPED: [Effect; 2] = [SCREEN_OFF, HIDE];
/// After the last step: screen-on off, vibrate, no sound, notification gone.
const ENDED: [Effect; 3] = [SCREEN_OFF, BUZZ, HIDE];

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

    assert_eq!(brew.tick(), [show("Step 1 · 00:01", Some("Step 2"))]);
    assert_eq!(timer(&brew), running(0, "00:01"));

    // Boundary: the next step shows its full duration at once.
    assert_eq!(
        brew.tick(),
        [BUZZ, Effect::PlayStepSound, show("Step 2 · 00:01", None)]
    );
    assert_eq!(timer(&brew), running(1, "00:01"));
    assert_eq!(statuses(&brew), [Done, Current]);

    // After the last step: screen-on off, ended, vibrate, no sound.
    assert_eq!(brew.tick(), ENDED);
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

    assert_eq!(
        brew.tick(),
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
        brew.tick(),
        [BUZZ, Effect::PlayStepSound, show("Step 2 · 00:00", None)]
    );
    assert_eq!(timer(&brew), running(1, "00:00"));

    assert_eq!(brew.tick(), ENDED);
    assert!(is_idle(&brew));
    Ok(())
}

#[test]
fn a_zero_second_middle_step_shows_00_00_for_one_tick() -> TestResult {
    let (mut brew, _catalog) = opened(&[1, 0, 1])?;
    brew.start(0)?;

    assert_eq!(
        brew.tick(),
        [
            BUZZ,
            Effect::PlayStepSound,
            show("Step 2 · 00:00", Some("Step 3"))
        ]
    );
    assert_eq!(timer(&brew), running(1, "00:00"));
    assert_eq!(
        brew.tick(),
        [BUZZ, Effect::PlayStepSound, show("Step 3 · 00:01", None)]
    );
    assert_eq!(timer(&brew), running(2, "00:01"));
    assert_eq!(brew.tick(), ENDED);
    Ok(())
}

#[test]
fn a_single_zero_second_step_ends_after_one_tick() -> TestResult {
    let (mut brew, _catalog) = opened(&[0])?;
    assert_eq!(brew.start(0)?, [SCREEN_ON, show("Step 1 · 00:00", None)]);
    assert_eq!(timer(&brew), running(0, "00:00"));
    assert_eq!(brew.tick(), ENDED);
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

    assert_eq!(
        brew.start(1)?,
        [SCREEN_ON, show("Step 2 · 00:02", Some("Step 3"))]
    );
    assert_eq!(timer(&brew), running(1, "00:02"));
    assert_eq!(statuses(&brew), [Done, Current, Pending]);

    assert_eq!(brew.tick(), [show("Step 2 · 00:01", Some("Step 3"))]);
    assert_eq!(
        brew.tick(),
        [BUZZ, Effect::PlayStepSound, show("Step 3 · 00:01", None)]
    );
    assert_eq!(timer(&brew), running(2, "00:01"));
    assert_eq!(statuses(&brew), [Done, Done, Current]);

    assert_eq!(brew.tick(), ENDED);
    assert!(is_idle(&brew));
    Ok(())
}

#[test]
fn start_from_the_last_step() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 1])?;
    brew.start(1)?;
    assert_eq!(statuses(&brew), [Done, Current]);
    assert_eq!(brew.tick(), ENDED);
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
    assert_eq!(brew.tick(), ENDED);

    assert_eq!(brew.start(0)?, [SCREEN_ON, show("Step 1 · 00:01", None)]);
    assert_eq!(timer(&brew), running(0, "00:01"));
    assert_eq!(brew.tick(), ENDED);
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

    assert_eq!(brew.reset(), STOPPED);
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

    assert_eq!(brew.leave(), STOPPED, "no vibration, no sound");
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
    brew.tick();
    let before = brew.state().clone();

    let change = catalog.upsert(recipe("b", &[9, 9])?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &before);

    let change = catalog.upsert(recipe("c", &[1])?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), []);
    assert_eq!(brew.state(), &before);

    assert_eq!(
        brew.tick(),
        [show("Step 1 · 00:01", Some("Step 2"))],
        "still running"
    );
    assert_eq!(timer(&brew), running(0, "00:01"));
    assert_eq!(
        brew.tick(),
        [BUZZ, Effect::PlayStepSound, show("Step 2 · 00:02", None)]
    );
    Ok(())
}

#[test]
fn deleting_the_open_recipe_mid_brew_shows_empty_recipe() -> TestResult {
    let (mut brew, mut catalog) = opened(&[3, 2])?;
    brew.start(0)?;

    let change = catalog.remove(id("a")?)?;
    assert_eq!(brew.on_catalog_change(&change, &catalog), STOPPED);
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
    let _effects = brew.tick();
    assert_eq!(timer(&brew), running(1, "00:10"));
    assert_eq!(run(&brew), first);
    let _effects = brew.tick();
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
    brew.adjust_ratio(RatioChange::Recipe)?;
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
    brew.tick();
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
    brew.tick();

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
    assert_eq!(brew.tick(), []);
    assert_eq!(brew.state(), &before);
    Ok(())
}

#[test]
fn resume_counts_down_from_where_it_was_held_in_the_same_run() -> TestResult {
    let (mut brew, _catalog) = opened(&[3, 2])?;
    brew.start(0)?;
    brew.tick();
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

    brew.tick();
    assert_eq!(
        brew.tick(),
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
    brew.tick();
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
        let _effects = brew.tick();
    }
    assert_eq!(timer(&brew), running(1, "00:02"));
    assert_eq!(pour_to(&brew).as_deref(), Some("Pour to 50g"));
    let _effects = brew.pause()?;
    assert_eq!(pour_to(&brew).as_deref(), Some("Pour to 50g"), "paused");
    let _effects = brew.resume()?;
    for _ in 0..2 {
        let _effects = brew.tick();
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
    brew.adjust_ratio(RatioChange::Recipe)?;
    assert_eq!(
        pour_labels(&brew),
        [None, Some("50g".to_owned()), Some("200g".to_owned())]
    );
    Ok(())
}
