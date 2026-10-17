//! Port of `test/domain/settings_test.dart`.

use coffee_timer_core::model::{AccentColor, AccentOption, ModelError, Settings, ThemeSelection};

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

#[test]
fn theme_try_parse_accepts_storage_values_only() {
    for theme in ThemeSelection::ALL {
        assert_eq!(ThemeSelection::try_parse(theme.storage_value()), Ok(theme));
    }
    for raw in ["", "Light", " dark", "nope"] {
        assert_eq!(
            ThemeSelection::try_parse(raw),
            Err(ModelError::UnknownTheme(raw.to_owned()))
        );
    }
    assert_eq!(
        ModelError::UnknownTheme("nope".to_owned()).to_string(),
        "Theme \"nope\" is not light, dark or system"
    );
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

// Settings::from_storage

#[test]
fn from_storage_reads_valid_values() -> TestResult {
    let settings = Settings::from_storage(Some("dark"), Some("#FFF44336"));
    assert_eq!(settings.theme(), ThemeSelection::Dark);
    assert_eq!(settings.accent(), AccentColor::try_from_hex("#FFF44336")?);
    Ok(())
}

#[test]
fn from_storage_of_nothing_is_the_fallback() {
    assert_eq!(Settings::from_storage(None, None), Settings::FALLBACK);
}

#[test]
fn from_storage_falls_back_per_field() {
    let bad_accent = Settings::from_storage(Some("light"), Some("#nothex"));
    assert_eq!(bad_accent.theme(), ThemeSelection::Light);
    assert_eq!(bad_accent.accent(), AccentColor::FALLBACK);

    let bad_theme = Settings::from_storage(Some("sepia"), Some("#FF4CAF50"));
    assert_eq!(bad_theme.theme(), ThemeSelection::System);
    assert_eq!(bad_theme.accent().argb(), 0xFF4C_AF50);
}

// AccentOption

#[test]
fn accent_options_match_the_legacy_picker() {
    let expected = [
        ("Blue", "#FF2196F3"),
        ("Red", "#FFF44336"),
        ("Green", "#FF4CAF50"),
        ("Purple", "#FF9C27B0"),
        ("Orange", "#FFFF9800"),
        ("Pink", "#FFE91E63"),
        ("Indigo", "#FF3F51B5"),
        ("Teal", "#FF009688"),
        ("Cyan", "#FF00BCD4"),
        ("Lime", "#FFCDDC39"),
    ];
    let actual: Vec<_> = AccentOption::ALL
        .iter()
        .map(|o| (o.name(), o.color().to_hex()))
        .collect();
    let expected: Vec<_> = expected
        .iter()
        .map(|&(name, hex)| (name, hex.to_owned()))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn fallback_accent_is_the_first_option() {
    assert_eq!(AccentOption::ALL[0].color(), AccentColor::FALLBACK);
}

#[test]
fn accent_option_named_ignores_case_and_rejects_unknown_names() {
    assert_eq!(
        AccentOption::named("Teal").map(AccentOption::name),
        Some("Teal")
    );
    assert_eq!(AccentOption::named("teal"), AccentOption::named("TEAL"));
    assert_eq!(
        AccentOption::named("teal").map(AccentOption::color),
        Some(AccentColor::from_argb32(0xFF00_9688))
    );
    assert_eq!(AccentOption::named("Mauve"), None);
    assert_eq!(AccentOption::named(""), None);
}
