//! Port of `test/domain/settings_test.dart`.

use coffee_timer_core::model::{
    AccentColor, AccentOption, ModelError, PreAlertLead, Settings, StoredSettings, ThemeSelection,
    Toggle,
};

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
    let settings = Settings::from_storage(StoredSettings {
        theme: Some("dark"),
        accent: Some("#FFF44336"),
        sound: Some("off"),
        vibration: Some("off"),
        pre_alert: Some("10"),
    });
    assert_eq!(settings.theme(), ThemeSelection::Dark);
    assert_eq!(settings.accent(), AccentColor::try_from_hex("#FFF44336")?);
    assert_eq!(settings.sound(), Toggle::Off);
    assert_eq!(settings.vibration(), Toggle::Off);
    assert_eq!(settings.pre_alert(), PreAlertLead::Seconds10);
    Ok(())
}

#[test]
fn from_storage_of_nothing_is_the_fallback() {
    assert_eq!(
        Settings::from_storage(StoredSettings::default()),
        Settings::FALLBACK
    );
}

#[test]
fn from_storage_falls_back_per_field() {
    let bad_accent = Settings::from_storage(StoredSettings {
        theme: Some("light"),
        accent: Some("#nothex"),
        ..StoredSettings::default()
    });
    assert_eq!(bad_accent.theme(), ThemeSelection::Light);
    assert_eq!(bad_accent.accent(), AccentColor::FALLBACK);

    let bad_theme = Settings::from_storage(StoredSettings {
        theme: Some("sepia"),
        accent: Some("#FF4CAF50"),
        ..StoredSettings::default()
    });
    assert_eq!(bad_theme.theme(), ThemeSelection::System);
    assert_eq!(bad_theme.accent().argb(), 0xFF4C_AF50);
}

#[test]
fn from_storage_falls_back_per_cue_field() {
    for bad in ["", "On", "true", "1", " off", "maybe"] {
        let settings = Settings::from_storage(StoredSettings {
            sound: Some(bad),
            vibration: Some("off"),
            ..StoredSettings::default()
        });
        assert_eq!(settings.sound(), Toggle::On, "sound {bad:?}");
        assert_eq!(settings.vibration(), Toggle::Off, "the good field stays");

        let settings = Settings::from_storage(StoredSettings {
            sound: Some("off"),
            vibration: Some(bad),
            ..StoredSettings::default()
        });
        assert_eq!(settings.vibration(), Toggle::On, "vibration {bad:?}");
        assert_eq!(settings.sound(), Toggle::Off, "the good field stays");
    }
    for bad in ["", "4", "-3", "3.0", "three", " 5", "4294967299"] {
        let settings = Settings::from_storage(StoredSettings {
            pre_alert: Some(bad),
            sound: Some("off"),
            ..StoredSettings::default()
        });
        assert_eq!(settings.pre_alert(), PreAlertLead::Seconds3, "{bad:?}");
        assert_eq!(settings.sound(), Toggle::Off, "the good field stays");
    }
}

#[test]
fn every_stored_cue_value_round_trips() {
    for sound in Toggle::ALL {
        for vibration in Toggle::ALL {
            for lead in PreAlertLead::ALL {
                let pre_alert = lead.storage_value();
                let settings = Settings::from_storage(StoredSettings {
                    sound: Some(sound.storage_value()),
                    vibration: Some(vibration.storage_value()),
                    pre_alert: Some(&pre_alert),
                    ..StoredSettings::default()
                });
                assert_eq!(
                    settings,
                    Settings::FALLBACK
                        .with_sound(sound)
                        .with_vibration(vibration)
                        .with_pre_alert(lead)
                );
            }
        }
    }
}

#[test]
fn fallback_cues_are_sound_and_vibration_on_with_a_3_s_pre_alert() {
    assert_eq!(Settings::FALLBACK.sound(), Toggle::On);
    assert_eq!(Settings::FALLBACK.vibration(), Toggle::On);
    assert_eq!(Settings::FALLBACK.pre_alert(), PreAlertLead::Seconds3);
    assert_eq!(PreAlertLead::DEFAULT, PreAlertLead::Seconds3);
}

#[test]
fn with_cue_operations_change_only_their_field() {
    let base = Settings::new(ThemeSelection::Dark, AccentOption::ALL[1].color());
    let next = base.with_sound(Toggle::Off);
    assert_eq!(
        next,
        Settings::new(ThemeSelection::Dark, AccentOption::ALL[1].color()).with_sound(Toggle::Off)
    );
    assert_eq!(
        (next.vibration(), next.pre_alert(), next.theme()),
        (base.vibration(), base.pre_alert(), base.theme())
    );
    let next = base.with_vibration(Toggle::Off);
    assert_eq!(
        (next.vibration(), next.sound(), next.pre_alert()),
        (Toggle::Off, Toggle::On, PreAlertLead::Seconds3)
    );
    let next = base.with_pre_alert(PreAlertLead::Off);
    assert_eq!(
        (next.pre_alert(), next.sound(), next.vibration()),
        (PreAlertLead::Off, Toggle::On, Toggle::On)
    );
    assert_eq!(next.accent(), base.accent());
}

// Toggle

#[test]
fn toggle_try_parse_accepts_on_and_off_only() {
    assert_eq!(Toggle::try_parse("on"), Ok(Toggle::On));
    assert_eq!(Toggle::try_parse("off"), Ok(Toggle::Off));
    for raw in ["", "On", "OFF", "yes", "true"] {
        assert_eq!(
            Toggle::try_parse(raw),
            Err(ModelError::UnknownToggle(raw.to_owned()))
        );
    }
    assert_eq!(
        ModelError::UnknownToggle("yes".to_owned()).to_string(),
        "\"yes\" is not on or off"
    );
}

#[test]
fn toggle_from_bool() {
    assert_eq!(Toggle::from(true), Toggle::On);
    assert_eq!(Toggle::from(false), Toggle::Off);
    assert!(Toggle::On.is_on() && !Toggle::Off.is_on());
}

// PreAlertLead

#[test]
fn pre_alert_options_are_off_3_5_and_10_seconds() {
    let seconds: Vec<u32> = PreAlertLead::ALL
        .into_iter()
        .map(PreAlertLead::seconds)
        .collect();
    assert_eq!(seconds, [0, 3, 5, 10]);
    let labels: Vec<String> = PreAlertLead::ALL
        .into_iter()
        .map(PreAlertLead::label)
        .collect();
    assert_eq!(labels, ["Off", "3 s", "5 s", "10 s"]);
}

#[test]
fn pre_alert_parses_its_seconds_only() {
    for lead in PreAlertLead::ALL {
        assert_eq!(PreAlertLead::try_parse(&lead.storage_value()), Ok(lead));
        assert_eq!(PreAlertLead::try_from_seconds(lead.seconds()), Ok(lead));
    }
    for raw in ["", "1", "4", "-3", "off", "3s"] {
        assert_eq!(
            PreAlertLead::try_parse(raw),
            Err(ModelError::UnknownPreAlert(raw.to_owned()))
        );
    }
    assert_eq!(
        PreAlertLead::try_from_seconds(7),
        Err(ModelError::UnknownPreAlert("7".to_owned()))
    );
    assert_eq!(
        ModelError::UnknownPreAlert("7".to_owned()).to_string(),
        "Pre-alert \"7\" is not 0, 3, 5 or 10 seconds"
    );
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
