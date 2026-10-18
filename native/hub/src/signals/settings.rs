use rinf::{DartSignal, RustSignal, SignalPiece};
use serde::{Deserialize, Serialize};

/// The settings holder is subscribed: answer with the current
/// `SettingsUpdate`.
#[derive(Deserialize, DartSignal)]
pub struct SettingsReady;

#[derive(Deserialize, DartSignal)]
pub struct SetTheme {
    pub theme: ThemeChoice,
}

/// Any 32-bit ARGB colour; the picker offers `SettingsUpdate::accents`.
#[derive(Deserialize, DartSignal)]
pub struct SetAccent {
    pub argb: u32,
}

/// Turn the brew's step sound on or off.
#[derive(Deserialize, DartSignal)]
pub struct SetSound {
    pub on: bool,
}

/// Turn the brew's vibration on or off.
#[derive(Deserialize, DartSignal)]
pub struct SetVibration {
    pub on: bool,
}

/// One of `SettingsUpdate::pre_alerts`' `seconds`; any other is refused.
#[derive(Deserialize, DartSignal)]
pub struct SetPreAlert {
    pub seconds: u32,
}

/// The settings and the choices offered. Dart replaces its state with each
/// one; the app theme follows it.
#[derive(Serialize, RustSignal)]
pub struct SettingsUpdate {
    pub theme: ThemeChoice,
    /// ARGB.
    pub accent: u32,
    /// The theme picker's options, in order.
    pub themes: Vec<ThemeChoice>,
    /// The accent picker's options, in order.
    pub accents: Vec<AccentChoice>,
    /// Whether a brew plays its step sound.
    pub sound: bool,
    /// Whether a brew buzzes.
    pub vibration: bool,
    /// The pre-alert lead, one of `pre_alerts`' `seconds`; 0 is off.
    pub pre_alert_seconds: u32,
    /// The pre-alert picker's options, in order.
    pub pre_alerts: Vec<PreAlertChoice>,
}

#[derive(Serialize, Deserialize, SignalPiece, Clone, Copy)]
pub enum ThemeChoice {
    Light,
    Dark,
    System,
}

/// A pre-alert option: its seconds (0 is off) and its label (`Off`,
/// `3 s`).
#[derive(Serialize, SignalPiece)]
pub struct PreAlertChoice {
    pub seconds: u32,
    pub label: String,
}

#[derive(Serialize, SignalPiece)]
pub struct AccentChoice {
    pub name: String,
    pub argb: u32,
}
