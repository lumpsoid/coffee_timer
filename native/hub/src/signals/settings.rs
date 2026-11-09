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

/// Turn the brew's sounds on or off: the master mute over both choices.
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

/// One of `SettingsUpdate::step_sounds`' `id`, to play at each step boundary;
/// any other is refused.
#[derive(Deserialize, DartSignal)]
pub struct SetStepSound {
    pub sound: String,
}

/// One of `SettingsUpdate::finish_sounds`' `id`, to play when the brew finishes;
/// any other is refused.
#[derive(Deserialize, DartSignal)]
pub struct SetFinishSound {
    pub sound: String,
}

/// Play one of `SettingsUpdate::step_sounds` or `finish_sounds` once, as a `PlatformEffect`, even
/// with the sound off. Changes nothing.
#[derive(Deserialize, DartSignal)]
pub struct PreviewSound {
    pub sound: String,
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
    /// Whether a brew plays its sounds.
    pub sound: bool,
    /// Whether a brew buzzes.
    pub vibration: bool,
    /// The pre-alert lead, one of `pre_alerts`' `seconds`; 0 is off.
    pub pre_alert_seconds: u32,
    /// The pre-alert picker's options, in order.
    pub pre_alerts: Vec<PreAlertChoice>,
    /// The sound at each step boundary, one of `step_sounds`' `id`.
    pub step_sound: String,
    /// The sound at the finish, one of `finish_sounds`' `id`.
    pub finish_sound: String,
    /// The step sound picker's options, in order.
    pub step_sounds: Vec<SoundChoice>,
    /// The end-of-brew sound picker's options, in order.
    pub finish_sounds: Vec<SoundChoice>,
    /// The app's version and commit, `0.1.0 (2391c1d)`.
    pub app_version: String,
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

/// A sound option: its id, name (`Temple block`) and length (`1.0 s`).
#[derive(Serialize, SignalPiece)]
pub struct SoundChoice {
    pub id: String,
    pub label: String,
    pub length_label: String,
}

#[derive(Serialize, SignalPiece)]
pub struct AccentChoice {
    pub name: String,
    pub argb: u32,
}
