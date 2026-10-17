//! Port of `test/domain/recipe_test.dart`.

use coffee_timer_core::model::{
    BrewMetrics, ModelError, Recipe, RecipeId, RecipeName, RecipeStep, StepDuration, StepId,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn step(seconds: i64) -> Result<RecipeStep, ModelError> {
    Ok(RecipeStep::new(
        StepId::try_parse(&format!("s{seconds}"))?,
        "step".to_owned(),
        StepDuration::clamped(seconds),
    ))
}

fn recipe(step_seconds: &[i64]) -> Result<Recipe, ModelError> {
    Ok(Recipe::new(
        RecipeId::try_parse("r")?,
        RecipeName::try_parse("Recipe")?,
        String::new(),
        step_seconds
            .iter()
            .map(|&s| step(s))
            .collect::<Result<_, _>>()?,
    ))
}

fn durations(seconds: &[i64]) -> Vec<StepDuration> {
    seconds.iter().map(|&s| StepDuration::clamped(s)).collect()
}

// RecipeId / StepId

#[test]
fn ids_reject_empty_keep_non_empty_and_compare_by_value() -> TestResult {
    assert_eq!(RecipeId::try_parse(""), Err(ModelError::EmptyRecipeId));
    assert_eq!(StepId::try_parse(""), Err(ModelError::EmptyStepId));
    assert_eq!(RecipeId::try_parse("a")?, RecipeId::try_parse("a")?);
    assert_ne!(RecipeId::try_parse("a")?, RecipeId::try_parse("b")?);
    Ok(())
}

// RecipeName

#[test]
fn recipe_name_trims_and_rejects_blank() -> TestResult {
    assert_eq!(RecipeName::try_parse("  V60  ")?.as_str(), "V60");
    assert_eq!(
        RecipeName::try_parse("   "),
        Err(ModelError::BlankRecipeName)
    );
    assert_eq!(RecipeName::try_parse(""), Err(ModelError::BlankRecipeName));
    Ok(())
}

#[test]
fn recipe_name_untitled_fallback_is_a_valid_name() {
    assert_eq!(RecipeName::untitled().as_str(), "Untitled");
}

#[test]
fn recipe_name_error_text_matches_legacy_validator() {
    assert_eq!(
        ModelError::BlankRecipeName.to_string(),
        "Enter a recipe name"
    );
}

// StepDuration

#[test]
fn step_duration_try_parse_enforces_bounds_strictly() -> TestResult {
    let max = i64::from(StepDuration::MAX_STEP_SECONDS);
    assert_eq!(
        StepDuration::try_parse(-1),
        Err(ModelError::StepDurationOutOfRange(-1))
    );
    assert_eq!(StepDuration::try_parse(0)?.seconds(), 0);
    assert_eq!(StepDuration::try_parse(max)?.seconds(), 5999);
    assert_eq!(
        StepDuration::try_parse(max + 1),
        Err(ModelError::StepDurationOutOfRange(max + 1))
    );
    Ok(())
}

#[test]
fn step_duration_error_text_matches_legacy_validator() {
    assert_eq!(
        ModelError::StepDurationOutOfRange(6000).to_string(),
        "0–5999 seconds"
    );
}

#[test]
fn step_duration_clamped_never_leaves_the_range() {
    assert_eq!(StepDuration::clamped(-5).seconds(), 0);
    assert_eq!(StepDuration::clamped(90).seconds(), 90);
    assert_eq!(
        StepDuration::clamped(10000).seconds(),
        StepDuration::MAX_STEP_SECONDS
    );
}

#[test]
fn step_duration_mmss_zero_pads_both_parts() {
    assert_eq!(StepDuration::clamped(0).mmss(), "00:00");
    assert_eq!(StepDuration::clamped(65).mmss(), "01:05");
    assert_eq!(StepDuration::clamped(600).mmss(), "10:00");
}

#[test]
fn step_duration_seconds_label_is_plain_seconds() {
    assert_eq!(StepDuration::clamped(0).seconds_label(), "0s");
    assert_eq!(StepDuration::clamped(95).seconds_label(), "95s");
}

// RecipeStep::label_or_default

#[test]
fn label_or_default_keeps_trimmed_input_falls_back_to_step_n() {
    assert_eq!(RecipeStep::label_or_default("  Bloom ", 1), "Bloom");
    assert_eq!(RecipeStep::label_or_default("", 3), "Step 3");
    assert_eq!(RecipeStep::label_or_default("   ", 2), "Step 2");
}

// Recipe mutations

#[test]
fn renamed_and_described_as_keep_the_id() -> TestResult {
    let base = recipe(&[30, 45])?;
    let renamed = base.clone().renamed(RecipeName::try_parse("V60")?);
    assert_eq!(renamed.name().as_str(), "V60");
    assert_eq!(renamed.id(), base.id());
    assert_eq!(base.name().as_str(), "Recipe", "original untouched");

    assert_eq!(
        base.described_as("pour slowly".to_owned()).description(),
        "pour slowly"
    );
    Ok(())
}

#[test]
fn with_step_added_appends() -> TestResult {
    let base = recipe(&[30, 45])?;
    let added = base.clone().with_step_added(step(10)?);
    assert_eq!(added.steps().len(), 3);
    assert_eq!(base.steps().len(), 2);
    Ok(())
}

#[test]
fn with_step_removed_at_drops_the_step_at_the_index() -> TestResult {
    let base = recipe(&[30, 45])?;
    let removed = base.clone().with_step_removed_at(0)?;
    assert_eq!(removed.steps().len(), 1);
    assert_eq!(removed.steps()[0].duration().seconds(), 45);
    assert_eq!(base.steps().len(), 2);
    Ok(())
}

#[test]
fn with_step_removed_at_rejects_an_index_past_the_end() -> TestResult {
    let base = recipe(&[30, 45])?;
    assert_eq!(
        base.with_step_removed_at(2),
        Err(ModelError::StepIndexOutOfRange { index: 2, len: 2 })
    );
    Ok(())
}

#[test]
fn blank_starts_empty_and_builds_up() -> TestResult {
    let built = Recipe::blank(RecipeId::try_parse("x")?, RecipeName::try_parse("Aero")?)
        .described_as("quick".to_owned())
        .with_step_added(step(20)?);
    assert_eq!(built.description(), "quick");
    assert_eq!(built.steps().len(), 1);
    assert_eq!(built.steps()[0].duration().seconds(), 20);
    Ok(())
}

// BrewMetrics

#[test]
fn duration_label_of_formats_a_raw_durations_list() {
    let metrics = BrewMetrics::default();
    assert_eq!(metrics.duration_label_of(durations(&[65])), "1:05");
    assert_eq!(metrics.duration_label_of(durations(&[])), "0:00");
    assert_eq!(metrics.duration_label_of(durations(&[5999, 5999])), "99:99");
}

#[test]
fn grams_scale_with_step_count() -> TestResult {
    let metrics = BrewMetrics::default();
    let r = recipe(&[30, 30, 30])?;
    assert_eq!(metrics.coffee_grams(&r), 30);
    assert_eq!(metrics.water_grams(&r), 150);
    Ok(())
}

#[test]
fn total_seconds_sums_steps() -> TestResult {
    let metrics = BrewMetrics::default();
    assert_eq!(metrics.total_seconds(&recipe(&[30, 45, 15])?), 90);
    assert_eq!(metrics.total_seconds(&recipe(&[])?), 0);
    Ok(())
}

#[test]
fn total_seconds_clamps_at_the_cap() -> TestResult {
    // three near-max steps blow past 6000
    let r = recipe(&[5999, 5999, 5999])?;
    assert_eq!(BrewMetrics::default().total_seconds(&r), 6000);
    Ok(())
}

#[test]
fn duration_label_formats_m_ss_with_unpadded_minutes() -> TestResult {
    let metrics = BrewMetrics::default();
    assert_eq!(metrics.duration_label(&recipe(&[0])?), "0:00");
    assert_eq!(metrics.duration_label(&recipe(&[65])?), "1:05");
    assert_eq!(metrics.duration_label(&recipe(&[600])?), "10:00");
    Ok(())
}

#[test]
fn duration_label_reads_99_99_at_the_cap() -> TestResult {
    let r = recipe(&[5999, 5999])?;
    assert_eq!(BrewMetrics::default().duration_label(&r), "99:99");
    Ok(())
}
