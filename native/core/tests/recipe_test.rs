//! Port of `test/domain/recipe_test.dart`.

use coffee_timer_core::model::{
    BrewMetrics, BrewRatio, CoffeeDose, ModelError, PourScale, Recipe, RecipeId, RecipeName,
    RecipeStep, StepDuration, StepId, WaterAmount, WaterScaleError,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn step(seconds: i64) -> Result<RecipeStep, ModelError> {
    Ok(RecipeStep::new(
        StepId::try_parse(&format!("s{seconds}"))?,
        "step".to_owned(),
        StepDuration::clamped(seconds),
        None,
    ))
}

fn dose(grams: i64) -> Result<CoffeeDose, ModelError> {
    CoffeeDose::try_parse(grams)
}

fn water(grams: i64) -> Result<WaterAmount, ModelError> {
    WaterAmount::try_parse(grams)
}

/// `water_grams` per gram of `coffee`.
fn ratio(coffee: i64, water_grams: i64) -> Result<BrewRatio, ModelError> {
    Ok(BrewRatio::of(dose(coffee)?, water(water_grams)?))
}

fn recipe(step_seconds: &[i64]) -> Result<Recipe, ModelError> {
    Ok(Recipe::new(
        RecipeId::try_parse("r")?,
        RecipeName::try_parse("Recipe")?,
        String::new(),
        dose(20)?,
        water(100)?,
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
fn ids_from_u128_are_hyphenated_lowercase_uuids() {
    let value = 0x0192_3D4E_5F60_7A8B_9CAD_BECF_D0E1_F203;
    assert_eq!(
        RecipeId::from_u128(value).as_str(),
        "01923d4e-5f60-7a8b-9cad-becfd0e1f203"
    );
    assert_eq!(
        StepId::from_u128(value).as_str(),
        "01923d4e-5f60-7a8b-9cad-becfd0e1f203"
    );
    assert_eq!(
        RecipeId::from_u128(0).as_str(),
        "00000000-0000-0000-0000-000000000000"
    );
    assert_eq!(
        RecipeId::from_u128(u128::MAX).as_str(),
        "ffffffff-ffff-ffff-ffff-ffffffffffff"
    );
}

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
fn recipe_name_copy_of_adds_the_copy_suffix() -> TestResult {
    assert_eq!(RecipeName::try_parse("V60")?.copy_of().as_str(), "V60 copy");
    assert_eq!(
        RecipeName::try_parse("V60 copy")?.copy_of().as_str(),
        "V60 copy copy"
    );
    Ok(())
}

#[test]
fn recipe_name_copy_of_a_name_at_the_bound_cuts_the_base_not_the_suffix() -> TestResult {
    let max = RecipeName::MAX_GENERATED_CHARS;
    let fits = "a".repeat(max - " copy".len());
    assert_eq!(
        RecipeName::try_parse(&fits)?.copy_of().as_str(),
        format!("{fits} copy")
    );
    let long = "b".repeat(max + 10);
    let copy = RecipeName::try_parse(&long)?.copy_of();
    assert_eq!(copy.as_str().chars().count(), max);
    assert_eq!(copy.as_str(), format!("{} copy", "b".repeat(max - 5)));
    Ok(())
}

#[test]
fn recipe_name_copy_of_counts_characters_and_drops_a_space_left_by_the_cut() -> TestResult {
    let max = RecipeName::MAX_GENERATED_CHARS;
    // The cut falls right after a space: the base keeps no trailing space.
    let spaced = format!("{} tail", "é".repeat(max - 6));
    let copy = RecipeName::try_parse(&spaced)?.copy_of();
    assert_eq!(copy.as_str(), format!("{} copy", "é".repeat(max - 6)));
    Ok(())
}

#[test]
fn recipe_name_from_storage_trims_and_falls_back_to_untitled() {
    assert_eq!(RecipeName::from_storage("  V60  ").as_str(), "V60");
    assert_eq!(RecipeName::from_storage("   "), RecipeName::untitled());
    assert_eq!(RecipeName::from_storage(""), RecipeName::untitled());
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
fn step_duration_try_parse_text_reads_trimmed_signed_integers_strictly() -> TestResult {
    assert_eq!(StepDuration::try_parse_text("45")?.seconds(), 45);
    assert_eq!(StepDuration::try_parse_text("  45 ")?.seconds(), 45);
    assert_eq!(StepDuration::try_parse_text("+5")?.seconds(), 5);
    assert_eq!(StepDuration::try_parse_text("0")?.seconds(), 0);
    assert_eq!(StepDuration::try_parse_text("5999")?.seconds(), 5999);
    assert_eq!(
        StepDuration::try_parse_text("-1"),
        Err(ModelError::StepDurationOutOfRange(-1))
    );
    assert_eq!(
        StepDuration::try_parse_text("6000"),
        Err(ModelError::StepDurationOutOfRange(6000))
    );
    for not_a_number in ["", "   ", "abc", "1.5", "1 2", "99999999999999999999"] {
        assert_eq!(
            StepDuration::try_parse_text(not_a_number),
            Err(ModelError::StepDurationNotANumber(not_a_number.to_owned())),
            "{not_a_number:?}"
        );
    }
    Ok(())
}

#[test]
fn step_duration_not_a_number_text_matches_legacy_validator() {
    assert_eq!(
        ModelError::StepDurationNotANumber("x".to_owned()).to_string(),
        "Enter seconds"
    );
}

#[test]
fn step_duration_clamped_text_is_lenient() {
    assert_eq!(StepDuration::clamped_text(" 65 ").seconds(), 65);
    assert_eq!(StepDuration::clamped_text("10000").seconds(), 5999);
    assert_eq!(StepDuration::clamped_text("-5"), StepDuration::ZERO);
    assert_eq!(StepDuration::clamped_text("abc"), StepDuration::ZERO);
    assert_eq!(StepDuration::clamped_text(""), StepDuration::ZERO);
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
fn step_duration_saturating_sub_stops_at_zero() {
    assert_eq!(StepDuration::clamped(5).saturating_sub(1).seconds(), 4);
    assert_eq!(
        StepDuration::clamped(1).saturating_sub(1),
        StepDuration::ZERO
    );
    assert_eq!(StepDuration::ZERO.saturating_sub(1), StepDuration::ZERO);
    assert!(StepDuration::ZERO.is_zero());
    assert!(!StepDuration::clamped(1).is_zero());
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
    let built = Recipe::blank(
        RecipeId::try_parse("x")?,
        RecipeName::try_parse("Aero")?,
        dose(17)?,
        water(220)?,
    )
    .described_as("quick".to_owned())
    .with_step_added(step(20)?);
    assert_eq!(built.description(), "quick");
    assert_eq!(built.steps().len(), 1);
    assert_eq!(built.steps()[0].duration().seconds(), 20);
    assert_eq!((built.coffee().grams(), built.water().grams()), (17, 220));
    Ok(())
}

// CoffeeDose / WaterAmount

#[test]
fn coffee_dose_accepts_1_to_200_grams() {
    assert_eq!(CoffeeDose::try_parse(1).map(CoffeeDose::grams), Ok(1));
    assert_eq!(CoffeeDose::try_parse(200).map(CoffeeDose::grams), Ok(200));
    assert_eq!(
        CoffeeDose::try_parse(0),
        Err(ModelError::CoffeeDoseOutOfRange(0))
    );
    assert_eq!(
        CoffeeDose::try_parse(201),
        Err(ModelError::CoffeeDoseOutOfRange(201))
    );
}

#[test]
fn water_amount_accepts_1_to_2000_grams() {
    assert_eq!(WaterAmount::try_parse(1).map(WaterAmount::grams), Ok(1));
    assert_eq!(
        WaterAmount::try_parse(2000).map(WaterAmount::grams),
        Ok(2000)
    );
    assert_eq!(
        WaterAmount::try_parse(0),
        Err(ModelError::WaterAmountOutOfRange(0))
    );
    assert_eq!(
        WaterAmount::try_parse(2001),
        Err(ModelError::WaterAmountOutOfRange(2001))
    );
}

#[test]
fn grams_text_is_trimmed_decimal_with_its_own_messages() {
    assert_eq!(
        CoffeeDose::try_parse_text(" 18 ").map(CoffeeDose::grams),
        Ok(18)
    );
    assert_eq!(
        WaterAmount::try_parse_text("250").map(WaterAmount::grams),
        Ok(250)
    );
    let not_a_number = CoffeeDose::try_parse_text("18.5").map_err(|e| e.to_string());
    assert_eq!(not_a_number, Err("Enter grams".to_owned()));
    let not_a_number = WaterAmount::try_parse_text("").map_err(|e| e.to_string());
    assert_eq!(not_a_number, Err("Enter grams".to_owned()));
    let too_much = CoffeeDose::try_parse_text("500").map_err(|e| e.to_string());
    assert_eq!(too_much, Err("1–200 g".to_owned()));
    let too_much = WaterAmount::try_parse_text("0").map_err(|e| e.to_string());
    assert_eq!(too_much, Err("1–2000 g".to_owned()));
}

#[test]
fn stored_grams_are_clamped_into_bounds() {
    assert_eq!(CoffeeDose::clamped(0).grams(), 1);
    assert_eq!(CoffeeDose::clamped(18).grams(), 18);
    assert_eq!(CoffeeDose::clamped(i64::MAX).grams(), 200);
    assert_eq!(WaterAmount::clamped(-5).grams(), 1);
    assert_eq!(WaterAmount::clamped(36).grams(), 36);
    assert_eq!(WaterAmount::clamped(10_000).grams(), 2000);
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

#[test]
fn ratio_label_is_the_stored_water_per_gram_of_stored_coffee() -> TestResult {
    let metrics = BrewMetrics::default();
    // 20 g to 100 g, whatever the steps.
    assert_eq!(metrics.ratio_label(&recipe(&[30, 30])?), "1:5");
    assert_eq!(metrics.ratio_label(&recipe(&[])?), "1:5");
    Ok(())
}

#[test]
fn brew_ratio_rounds_to_a_tenth() -> TestResult {
    assert_eq!(ratio(30, 150)?.label(), "1:5");
    assert_eq!(ratio(30, 100)?.label(), "1:3.3");
    assert_eq!(ratio(15, 250)?.label(), "1:16.7");
    assert_eq!(ratio(18, 36)?.label(), "1:2");
    Ok(())
}

#[test]
fn brew_ratio_steps_onto_the_half_grid_within_bounds() -> TestResult {
    let r = ratio(30, 100)?;
    assert_eq!(r.more_water().label(), "1:3.5");
    assert_eq!(r.less_water().label(), "1:3");
    let whole = ratio(10, 50)?;
    assert_eq!(whole.more_water().label(), "1:5.5");
    assert_eq!(whole.less_water().label(), "1:4.5");
    let weak = ratio(1, 1000)?;
    assert_eq!(weak.more_water().label(), "1:30");
    assert_eq!(weak.less_water().label(), "1:30");
    let strong = ratio(10, 1)?;
    assert_eq!(strong.less_water().label(), "1:1");
    assert_eq!(strong.more_water().label(), "1:1");
    Ok(())
}

#[test]
fn brew_ratio_water_rounds_half_up_to_a_gram() -> TestResult {
    let r = ratio(10, 155)?;
    assert_eq!(r.water_grams(dose(15)?), 233);
    assert_eq!(r.water_grams(dose(1)?), 16);
    Ok(())
}

// with_water_scaled_to

/// 15 g coffee to `water_grams`, with steps pouring `pours` grams each
/// (`None` pours nothing).
fn pouring(water_grams: i64, pours: &[Option<i64>]) -> Result<Recipe, ModelError> {
    let steps = pours
        .iter()
        .enumerate()
        .map(|(i, &grams)| {
            Ok(RecipeStep::new(
                StepId::try_parse(&format!("s{i}"))?,
                format!("Step {}", i + 1),
                StepDuration::try_parse(30)?,
                grams.map(WaterAmount::try_parse).transpose()?,
            ))
        })
        .collect::<Result<_, ModelError>>()?;
    Ok(Recipe::new(
        RecipeId::try_parse("r")?,
        RecipeName::try_parse("V60")?,
        "Clean".to_owned(),
        dose(15)?,
        water(water_grams)?,
        steps,
    ))
}

fn step_pours(recipe: &Recipe) -> Vec<Option<u32>> {
    recipe
        .steps()
        .iter()
        .map(|step| step.water().map(WaterAmount::grams))
        .collect()
}

#[test]
fn scaling_the_water_stores_the_pours_pour_scale_shows() -> TestResult {
    let before = pouring(250, &[None, Some(33), Some(33), Some(184)])?;
    let shown: Vec<Option<u32>> = PourScale::new(before.water(), 255)
        .pours(before.steps())
        .into_iter()
        .map(|pour| pour.map(|p| p.grams()))
        .collect();
    let after = before.clone().with_water_scaled_to(255)?;
    assert_eq!(after.water().grams(), 255);
    assert_eq!(step_pours(&after), shown);
    assert_eq!(step_pours(&after), [None, Some(34), Some(33), Some(188)]);
    // Everything else is kept.
    assert_eq!(after.id(), before.id());
    assert_eq!(after.name(), before.name());
    assert_eq!(after.description(), "Clean");
    assert_eq!(after.coffee(), before.coffee());
    let ids = |r: &Recipe| r.steps().iter().map(|s| s.id().clone()).collect::<Vec<_>>();
    assert_eq!(ids(&after), ids(&before));
    Ok(())
}

#[test]
fn scaling_to_the_same_water_changes_nothing() -> TestResult {
    let before = pouring(250, &[Some(50), Some(200)])?;
    assert_eq!(before.clone().with_water_scaled_to(250)?, before);
    Ok(())
}

#[test]
fn scaling_past_what_a_recipe_holds_is_an_error() -> TestResult {
    let recipe = pouring(250, &[Some(250)])?;
    assert_eq!(
        recipe.clone().with_water_scaled_to(2001),
        Err(WaterScaleError::Water { grams: 2001 })
    );
    assert_eq!(
        recipe.with_water_scaled_to(0),
        Err(WaterScaleError::Water { grams: 0 })
    );
    Ok(())
}

#[test]
fn a_scaled_pour_out_of_range_is_an_error() -> TestResult {
    // Pours need not add up to the water: 2000 g of pours in 1000 g.
    let recipe = pouring(1000, &[None, Some(2000)])?;
    assert_eq!(
        recipe.with_water_scaled_to(1050),
        Err(WaterScaleError::Pour {
            position: 2,
            grams: 2100
        })
    );
    // A pour that rounds away to nothing: 1 g at 0.4 is 0 g.
    let recipe = pouring(100, &[Some(1), Some(1)])?;
    let error = recipe
        .with_water_scaled_to(40)
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error,
        WaterScaleError::Pour {
            position: 1,
            grams: 0
        }
    );
    assert_eq!(error.to_string(), "step 1 would pour 0 g, outside 1–2000 g");
    Ok(())
}
