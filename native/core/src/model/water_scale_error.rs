use std::fmt;

use crate::model::WaterAmount;

/// Why a recipe cannot take a new total water with its pours scaled to it
/// ([`crate::model::Recipe::with_water_scaled_to`]): an amount it would
/// store is outside [`WaterAmount`]'s bounds. The recipe is unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaterScaleError {
    /// The new total water.
    Water { grams: u32 },
    /// The scaled pour of the step at 1-based `position`.
    Pour { position: usize, grams: u32 },
}

impl fmt::Display for WaterScaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (min, max) = (WaterAmount::MIN_GRAMS, WaterAmount::MAX_GRAMS);
        match self {
            Self::Water { grams } => {
                write!(f, "{grams} g of water is outside {min}–{max} g")
            }
            Self::Pour { position, grams } => {
                write!(
                    f,
                    "step {position} would pour {grams} g, outside {min}–{max} g"
                )
            }
        }
    }
}

impl std::error::Error for WaterScaleError {}
