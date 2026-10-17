//! Port of `test/domain/settings_test.dart`.

use coffee_timer_core::model::{AccentColor, ModelError, Settings, ThemeSelection};

type TestResult = Result<(), Box<dyn std::error::Error>>;

// ThemeSelection::from_storage

#[test]
fn theme_from_storage_parses_known_values() {
    assert_eq!(
        ThemeSelection::from_storage(Some("light")),
        ThemeSelection::Light
    );
    assert_eq!(
        ThemeSelection::from_storage(Some("dark")),
        ThemeSelection::Dark
    );
    assert_eq!(
        ThemeSelection::from_storage(Some("system")),
        ThemeSelection::System
    );
}

#[test]
fn theme_from_storage_falls_back_to_system_for_unknown_or_missing() {
    assert_eq!(
        ThemeSelection::from_storage(Some("nope")),
        ThemeSelection::System
    );
    assert_eq!(ThemeSelection::from_storage(None), ThemeSelection::System);
}

#[test]
fn theme_storage_value_round_trips() {
    for theme in [
        ThemeSelection::Light,
        ThemeSelection::Dark,
        ThemeSelection::System,
    ] {
        assert_eq!(
            ThemeSelection::from_storage(Some(theme.storage_value())),
            theme
        );
    }
}

// AccentColor

#[test]
fn accent_round_trips_a_full_hex_string() -> TestResult {
    let color = AccentColor::try_from_hex("#FF2196F3")?;
    assert_eq!(color.argb(), 0xFF21_96F3);
    assert_eq!(color.to_hex(), "#FF2196F3");
    Ok(())
}

#[test]
fn accent_pads_a_6_digit_hex_to_opaque() -> TestResult {
    assert_eq!(AccentColor::try_from_hex("#2196F3")?.argb(), 0xFF21_96F3);
    Ok(())
}

#[test]
fn accent_rejects_malformed_hex() {
    assert_eq!(
        AccentColor::try_from_hex("xyz"),
        Err(ModelError::MalformedAccentHex("xyz".to_owned()))
    );
    assert_eq!(
        AccentColor::try_from_hex("#12"),
        Err(ModelError::MalformedAccentHex("#12".to_owned()))
    );
}

#[test]
fn accent_rejects_signed_or_padded_hex() {
    // Legacy `int.tryParse` accepted a sign or padding; any sign or
    // whitespace is rejected.
    for raw in ["#-1234567", "#+1234567", "+FFFFFFF", " FF2196F", "FF2196F "] {
        assert!(AccentColor::try_from_hex(raw).is_err(), "{raw:?}");
    }
}

#[test]
fn accent_rejects_out_of_range_argb() {
    assert_eq!(
        AccentColor::try_from_argb(-1),
        Err(ModelError::AccentOutOfRange(-1))
    );
    assert_eq!(
        AccentColor::try_from_argb(0x1_FFFF_FFFF),
        Err(ModelError::AccentOutOfRange(0x1_FFFF_FFFF))
    );
}

// Settings

#[test]
fn with_theme_changes_only_the_theme() {
    let base = Settings::FALLBACK;
    let next = base.with_theme(ThemeSelection::Dark);
    assert_eq!(next.theme(), ThemeSelection::Dark);
    assert_eq!(next.accent(), base.accent());
}

#[test]
fn with_accent_changes_only_the_accent() -> TestResult {
    let base = Settings::FALLBACK;
    let blue = AccentColor::try_from_argb(0xFF00_00FF)?;
    let next = base.with_accent(blue);
    assert_eq!(next.accent(), blue);
    assert_eq!(next.theme(), base.theme());
    Ok(())
}

#[test]
fn fallback_is_system_with_material_blue() {
    assert_eq!(Settings::FALLBACK.theme(), ThemeSelection::System);
    assert_eq!(Settings::FALLBACK.accent().to_hex(), "#FF2196F3");
}

#[test]
fn fallback_theme_matches_missing_storage() {
    assert_eq!(
        Settings::FALLBACK.theme(),
        ThemeSelection::from_storage(None)
    );
}
