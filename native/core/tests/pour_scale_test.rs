//! `PourScale`: the one rounding rule for step pours in a brew at another
//! water, and `WaterAmount`'s optional text entry for a step's pour.

use coffee_timer_core::model::{
    ModelError, Pour, PourScale, RecipeStep, StepDuration, StepId, WaterAmount,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Steps pouring `grams` each, in order; `None` pours nothing.
fn steps(grams: &[Option<i64>]) -> Result<Vec<RecipeStep>, ModelError> {
    grams
        .iter()
        .enumerate()
        .map(|(i, &water)| {
            Ok(RecipeStep::new(
                StepId::try_parse(&format!("s{i}"))?,
                format!("Step {}", i + 1),
                StepDuration::try_parse(30)?,
                water.map(WaterAmount::try_parse).transpose()?,
            ))
        })
        .collect()
}

/// `(pour grams, pour-to grams)` per step.
fn pours(scale: PourScale, grams: &[Option<i64>]) -> Result<Vec<Option<(u32, u32)>>, ModelError> {
    Ok(scale
        .pours(&steps(grams)?)
        .into_iter()
        .map(|pour| pour.map(|p| (p.grams(), p.to_grams())))
        .collect())
}

fn scale(recipe_water: i64, brew_water: u32) -> Result<PourScale, ModelError> {
    Ok(PourScale::new(
        WaterAmount::try_parse(recipe_water)?,
        brew_water,
    ))
}

#[test]
fn at_the_recipes_own_water_every_pour_stays_as_stored() -> TestResult {
    assert_eq!(
        pours(scale(250, 250)?, &[None, Some(50), Some(200)])?,
        [None, Some((50, 50)), Some((200, 250))]
    );
    Ok(())
}

#[test]
fn pours_scale_with_the_water_and_targets_add_up() -> TestResult {
    // 250 g -> 270 g: every pour × 1.08.
    assert_eq!(
        pours(scale(250, 270)?, &[Some(50), None, Some(200)])?,
        [Some((54, 54)), None, Some((216, 270))]
    );
    Ok(())
}

#[test]
fn running_totals_round_half_up_and_pours_are_their_differences() -> TestResult {
    // 300 g -> 250 g: totals 100, 200, 300 scale to 83.3, 166.7, 250 and
    // round to 83, 167, 250; each pour is the step between two totals.
    assert_eq!(
        pours(scale(300, 250)?, &[Some(100), Some(100), Some(100)])?,
        [Some((83, 83)), Some((84, 167)), Some((83, 250))]
    );
    assert_eq!(scale(2, 1)?.scaled(1), 1, "0.5 rounds up");
    assert_eq!(scale(4, 1)?.scaled(1), 0, "0.25 rounds down");
    Ok(())
}

#[test]
fn pours_need_not_add_up_to_the_recipes_water() -> TestResult {
    // A recipe of 300 g that pours only 100 g in steps: the pour still
    // scales by the whole water, 330 / 300.
    assert_eq!(pours(scale(300, 330)?, &[Some(100)])?, [Some((110, 110))]);
    Ok(())
}

#[test]
fn a_brew_may_scale_past_what_a_recipe_can_store() -> TestResult {
    assert_eq!(
        pours(scale(2000, 3000)?, &[Some(2000)])?,
        [Some((3000, 3000))]
    );
    Ok(())
}

#[test]
fn a_pour_reads_its_grams_and_its_target() -> TestResult {
    let pour: Pour = scale(250, 250)?
        .pours(&steps(&[Some(50), Some(100)])?)
        .get(1)
        .copied()
        .flatten()
        .ok_or("no pour")?;
    assert_eq!(pour.label(), "100g");
    assert_eq!(pour.to_label(), "Pour to 150g");
    Ok(())
}

#[test]
fn optional_water_text_is_none_when_blank_and_strict_otherwise() {
    assert_eq!(WaterAmount::try_parse_optional_text(""), Ok(None));
    assert_eq!(WaterAmount::try_parse_optional_text("  "), Ok(None));
    assert_eq!(
        WaterAmount::try_parse_optional_text(" 50 ").map(|w| w.map(WaterAmount::grams)),
        Ok(Some(50))
    );
    assert_eq!(
        WaterAmount::try_parse_optional_text("0"),
        Err(ModelError::WaterAmountOutOfRange(0))
    );
    assert_eq!(
        WaterAmount::try_parse_optional_text("lots"),
        Err(ModelError::WaterAmountNotANumber("lots".to_owned()))
    );
}
