/// The raw stored values of [`crate::model::Settings`], one per key, each
/// missing when its row is. [`crate::model::Settings::from_storage`]
/// decides what each falls back to.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StoredSettings<'a> {
    pub theme: Option<&'a str>,
    /// `#AARRGGBB`.
    pub accent: Option<&'a str>,
    /// `on` or `off`.
    pub sound: Option<&'a str>,
    /// `on` or `off`.
    pub vibration: Option<&'a str>,
    /// `on` or `off`.
    pub finish_knock: Option<&'a str>,
    /// Seconds: `0`, `3`, `5` or `10`.
    pub pre_alert: Option<&'a str>,
    /// A sound's id, `breath-and-bowl`.
    pub step_sound: Option<&'a str>,
    /// A sound's id.
    pub finish_sound: Option<&'a str>,
}
