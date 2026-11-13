use crate::model::ModelError;

/// The paper a recipe's list label is printed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LabelTexture {
    /// Long paper fibres.
    Kraft,
    /// Fine woven grid.
    Linen,
    /// Print dots fading from one corner.
    Halftone,
    /// Thin diagonal stripes.
    Pinstripe,
    /// Origin-map contour lines.
    Contour,
    /// Coarse jute weave.
    Burlap,
    /// No pattern: bare paper.
    Plain,
}

impl LabelTexture {
    /// The hash pool: the textures a recipe with no override is picked from.
    /// `Plain` is deliberately left out, so it is reachable only through a
    /// recipe's override; growing this list would reshuffle every recipe.
    pub const ALL: [Self; 6] = [
        Self::Kraft,
        Self::Linen,
        Self::Halftone,
        Self::Pinstripe,
        Self::Contour,
        Self::Burlap,
    ];

    /// Every texture a recipe can choose, in gallery order: the hashed six,
    /// then `Plain`.
    pub const CHOICES: [Self; 7] = [
        Self::Kraft,
        Self::Linen,
        Self::Halftone,
        Self::Pinstripe,
        Self::Contour,
        Self::Burlap,
        Self::Plain,
    ];

    /// The name the gallery shows the texture under.
    pub fn label(self) -> &'static str {
        match self {
            Self::Kraft => "Kraft",
            Self::Linen => "Linen",
            Self::Halftone => "Halftone",
            Self::Pinstripe => "Pinstripe",
            Self::Contour => "Contour",
            Self::Burlap => "Burlap",
            Self::Plain => "Plain",
        }
    }

    /// The string persisted in storage.
    pub fn storage_value(self) -> &'static str {
        match self {
            Self::Kraft => "kraft",
            Self::Linen => "linen",
            Self::Halftone => "halftone",
            Self::Pinstripe => "pinstripe",
            Self::Contour => "contour",
            Self::Burlap => "burlap",
            Self::Plain => "plain",
        }
    }

    /// Parses a [`LabelTexture::storage_value`] strictly: anything else is
    /// [`ModelError::UnknownTexture`]. For user input; storage reads use
    /// [`LabelTexture::from_storage`].
    pub fn try_parse(raw: &str) -> Result<Self, ModelError> {
        Self::CHOICES
            .into_iter()
            .find(|texture| texture.storage_value() == raw)
            .ok_or_else(|| ModelError::UnknownTexture(raw.to_owned()))
    }

    /// Parses a stored value. `None` (no override: the hash picks the
    /// texture) for anything unrecognised or missing.
    pub fn from_storage(value: Option<&str>) -> Option<Self> {
        value.and_then(|raw| Self::try_parse(raw).ok())
    }
}
