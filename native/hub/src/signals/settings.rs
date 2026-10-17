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
}

#[derive(Serialize, Deserialize, SignalPiece, Clone, Copy)]
pub enum ThemeChoice {
    Light,
    Dark,
    System,
}

#[derive(Serialize, SignalPiece)]
pub struct AccentChoice {
    pub name: String,
    pub argb: u32,
}
