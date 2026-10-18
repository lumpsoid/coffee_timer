use std::fmt;

/// Why a model value could not be built or an operation was rejected.
///
/// Texts match the legacy form messages where the legacy app had one
/// (`Enter a recipe name`, `Enter seconds`, `0–5999 seconds`); the grams
/// fields, new since the port, read `Enter grams` and `1–200 g`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    EmptyRecipeId,
    EmptyStepId,
    BlankRecipeName,
    StepDurationNotANumber(String),
    StepDurationOutOfRange(i64),
    CoffeeDoseNotANumber(String),
    CoffeeDoseOutOfRange(i64),
    WaterAmountNotANumber(String),
    WaterAmountOutOfRange(i64),
    AccentOutOfRange(i64),
    MalformedAccentHex(String),
    UnknownTheme(String),
    UnknownToggle(String),
    UnknownPreAlert(String),
    StepIndexOutOfRange { index: usize, len: usize },
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRecipeId => write!(f, "Recipe id is empty"),
            Self::EmptyStepId => write!(f, "Step id is empty"),
            Self::BlankRecipeName => write!(f, "Enter a recipe name"),
            Self::StepDurationNotANumber(_) => write!(f, "Enter seconds"),
            Self::StepDurationOutOfRange(_) => write!(
                f,
                "0–{} seconds",
                crate::model::StepDuration::MAX_STEP_SECONDS
            ),
            Self::CoffeeDoseNotANumber(_) | Self::WaterAmountNotANumber(_) => {
                write!(f, "Enter grams")
            }
            Self::CoffeeDoseOutOfRange(_) => write!(
                f,
                "{}–{} g",
                crate::model::CoffeeDose::MIN_GRAMS,
                crate::model::CoffeeDose::MAX_GRAMS
            ),
            Self::WaterAmountOutOfRange(_) => write!(
                f,
                "{}–{} g",
                crate::model::WaterAmount::MIN_GRAMS,
                crate::model::WaterAmount::MAX_GRAMS
            ),
            Self::AccentOutOfRange(value) => {
                write!(f, "Accent {value} is not a 32-bit ARGB value")
            }
            Self::MalformedAccentHex(raw) => {
                write!(f, "Accent \"{raw}\" is not #AARRGGBB or #RRGGBB")
            }
            Self::UnknownTheme(raw) => {
                write!(f, "Theme \"{raw}\" is not light, dark or system")
            }
            Self::UnknownToggle(raw) => write!(f, "\"{raw}\" is not on or off"),
            Self::UnknownPreAlert(raw) => {
                write!(f, "Pre-alert \"{raw}\" is not 0, 3, 5 or 10 seconds")
            }
            Self::StepIndexOutOfRange { index, len } => {
                write!(f, "Step {index} is out of range for {len} steps")
            }
        }
    }
}

impl std::error::Error for ModelError {}
