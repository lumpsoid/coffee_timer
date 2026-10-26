//! `GrindClicks`: parsing, bounds and labels.

use coffee_timer_core::model::{GrindClicks, ModelError};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn blank_text_is_no_grind() -> TestResult {
    assert_eq!(GrindClicks::try_parse_optional_text("")?, None);
    assert_eq!(GrindClicks::try_parse_optional_text("  ")?, None);
    Ok(())
}

#[test]
fn text_parses_whole_clicks_within_the_bounds() -> TestResult {
    let clicks = |raw| -> Result<Option<u32>, ModelError> {
        Ok(GrindClicks::try_parse_optional_text(raw)?.map(GrindClicks::clicks))
    };
    assert_eq!(clicks(" 22 ")?, Some(22));
    assert_eq!(clicks("0")?, Some(0));
    assert_eq!(clicks("99")?, Some(99));
    Ok(())
}

#[test]
fn text_out_of_range_or_not_a_whole_number_is_refused() {
    for raw in ["100", "-1"] {
        let error = GrindClicks::try_parse_optional_text(raw).map_err(|e| e.to_string());
        assert_eq!(error, Err("0–99 clicks".to_owned()), "{raw}");
    }
    for raw in ["fine", "2.5", "1e1"] {
        assert_eq!(
            GrindClicks::try_parse_optional_text(raw),
            Err(ModelError::GrindClicksNotANumber(raw.to_owned())),
            "{raw}"
        );
    }
    assert_eq!(
        ModelError::GrindClicksNotANumber(String::new()).to_string(),
        "Enter clicks"
    );
}

#[test]
fn stored_clicks_are_clamped() {
    assert_eq!(GrindClicks::clamped(-5).clicks(), 0);
    assert_eq!(GrindClicks::clamped(500).clicks(), 99);
    assert_eq!(GrindClicks::clamped(22).clicks(), 22);
}

#[test]
fn the_label_is_the_bare_count() -> TestResult {
    assert_eq!(GrindClicks::try_parse(22)?.label(), "22");
    assert_eq!(GrindClicks::try_parse(0)?.label(), "0");
    Ok(())
}
