//! `SettingsController`: load, set_theme and set_accent write through the
//! store first.

use coffee_timer_core::fakes::FakeSettingsStore;
use coffee_timer_core::model::{AccentColor, AccentOption, Settings, ThemeSelection};
use coffee_timer_core::settings::SettingsController;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn dark_red() -> Settings {
    Settings::new(ThemeSelection::Dark, AccentOption::ALL[1].color())
}

#[test]
fn new_starts_at_fallback_until_loaded() {
    let controller =
        SettingsController::new(Box::new(FakeSettingsStore::with_settings(dark_red())));
    assert_eq!(*controller.state(), Settings::FALLBACK);
}

#[test]
fn load_replaces_the_state() -> TestResult {
    let mut controller =
        SettingsController::new(Box::new(FakeSettingsStore::with_settings(dark_red())));
    controller.load()?;
    assert_eq!(*controller.state(), dark_red());
    Ok(())
}

#[test]
fn load_from_an_empty_store_gives_fallback() -> TestResult {
    let mut controller = SettingsController::new(Box::new(FakeSettingsStore::default()));
    controller.load()?;
    assert_eq!(*controller.state(), Settings::FALLBACK);
    Ok(())
}

#[test]
fn failed_load_leaves_state_unchanged() {
    let store = FakeSettingsStore::with_settings(dark_red()).failing_loads();
    let mut controller = SettingsController::new(Box::new(store));
    assert!(controller.load().is_err());
    assert_eq!(*controller.state(), Settings::FALLBACK);
}

#[test]
fn set_theme_changes_state_and_persists() -> TestResult {
    let mut controller = SettingsController::new(Box::new(FakeSettingsStore::default()));
    controller.set_theme(ThemeSelection::Dark)?;
    assert_eq!(controller.state().theme(), ThemeSelection::Dark);

    controller.load()?;
    assert_eq!(controller.state().theme(), ThemeSelection::Dark, "stored");
    Ok(())
}

#[test]
fn set_accent_changes_state_and_persists() -> TestResult {
    let teal = AccentOption::ALL[7].color();
    let mut controller = SettingsController::new(Box::new(FakeSettingsStore::default()));
    controller.set_accent(teal)?;
    assert_eq!(controller.state().accent(), teal);

    controller.load()?;
    assert_eq!(controller.state().accent(), teal, "stored");
    Ok(())
}

#[test]
fn set_accent_accepts_a_colour_outside_the_options() -> TestResult {
    let custom = AccentColor::try_from_hex("#FF123456")?;
    let mut controller = SettingsController::new(Box::new(FakeSettingsStore::default()));
    controller.set_accent(custom)?;
    assert_eq!(controller.state().accent(), custom);
    Ok(())
}

#[test]
fn failed_write_leaves_state_unchanged() -> TestResult {
    let store = FakeSettingsStore::with_settings(dark_red()).failing_writes();
    let mut controller = SettingsController::new(Box::new(store));
    controller.load()?;

    assert!(controller.set_theme(ThemeSelection::Light).is_err());
    assert!(controller.set_accent(AccentColor::FALLBACK).is_err());
    assert_eq!(*controller.state(), dark_red());
    Ok(())
}
