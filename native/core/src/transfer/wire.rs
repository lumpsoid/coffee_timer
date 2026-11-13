//! The document's JSON shape, version 1. Private: values cross into the
//! model only through `RecipeDocument`, which validates each one.
//! Fields this version does not know are ignored.

use serde::{Deserialize, Serialize};

/// Read first, alone, so a document of another format or version is
/// refused for that before its body is read.
#[derive(Deserialize)]
pub(super) struct Header {
    #[serde(default)]
    pub(super) format: Option<String>,
    #[serde(default)]
    pub(super) version: Option<u64>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct Document {
    pub(super) format: String,
    pub(super) version: u64,
    pub(super) recipes: Vec<RecipeWire>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct RecipeWire {
    #[serde(default)]
    pub(super) name: String,
    #[serde(default)]
    pub(super) description: String,
    pub(super) coffee_grams: i64,
    pub(super) water_grams: i64,
    /// Missing is the default speed: documents written before recipes had
    /// one still read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) pour_grams_per_second: Option<f64>,
    /// Missing is no grind setting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) grind_clicks: Option<i64>,
    /// Missing is no chosen paper: the recipe's id picks it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) label_texture: Option<String>,
    pub(super) steps: Vec<StepWire>,
}

#[derive(Serialize, Deserialize)]
pub(super) struct StepWire {
    /// Blank or missing is `Step N`.
    #[serde(default)]
    pub(super) name: String,
    pub(super) seconds: i64,
    /// Missing is no pour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) water_grams: Option<i64>,
}
