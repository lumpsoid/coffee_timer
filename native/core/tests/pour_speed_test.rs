//! `PourSpeed`: parsing, bounds, labels and the time a pour takes.

use coffee_timer_core::model::{ModelError, PourSpeed, WaterAmount};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn the_default_is_five_grams_a_second() {
    assert_eq!(PourSpeed::default(), PourSpeed::DEFAULT);
    assert_eq!(PourSpeed::DEFAULT.label(), "5 g/s");
}

#[test]
fn text_parses_whole_and_tenths_and_rounds_to_a_tenth() -> TestResult {
    assert_eq!(PourSpeed::try_parse_text(" 6 ")?.tenths(), 60);
    assert_eq!(PourSpeed::try_parse_text("4.5")?.text(), "4.5");
    assert_eq!(PourSpeed::try_parse_text("4.46")?.label(), "4.5 g/s");
    Ok(())
}

#[test]
fn text_out_of_range_or_not_a_number_is_refused() {
    for raw in ["0.4", "0", "-3", "30.1", "inf", "NaN"] {
        let error = PourSpeed::try_parse_text(raw).map_err(|e| e.to_string());
        assert_eq!(error, Err("0.5–30 g/s".to_owned()), "{raw}");
    }
    for raw in ["", "fast", "4,5"] {
        assert_eq!(
            PourSpeed::try_parse_text(raw),
            Err(ModelError::PourSpeedNotANumber(raw.to_owned())),
            "{raw}"
        );
    }
    assert_eq!(
        ModelError::PourSpeedNotANumber(String::new()).to_string(),
        "Enter g/s"
    );
}

#[test]
fn stored_tenths_are_clamped() {
    assert_eq!(PourSpeed::clamped_tenths(0), PourSpeed::MIN);
    assert_eq!(PourSpeed::clamped_tenths(9999), PourSpeed::MAX);
    assert_eq!(PourSpeed::clamped_tenths(45).text(), "4.5");
}

#[test]
fn a_pour_lasts_its_grams_over_the_speed_rounded_up() -> TestResult {
    let seconds = |speed: &str, grams: i64| -> Result<u32, ModelError> {
        Ok(PourSpeed::try_parse_text(speed)?
            .duration_of(WaterAmount::try_parse(grams)?)
            .seconds())
    };
    assert_eq!(seconds("5", 250)?, 50);
    assert_eq!(seconds("5", 51)?, 11, "a part second is a whole one");
    assert_eq!(seconds("4.5", 45)?, 10);
    assert_eq!(seconds("30", 1)?, 1);
    assert_eq!(seconds("0.5", 2000)?, 4000);
    Ok(())
}
