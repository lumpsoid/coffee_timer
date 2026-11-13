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
    /// Square grid with a heavier line every few cells.
    Graph,
    /// Ruled lines with a margin line.
    Ledger,
    /// Small dots on a square grid.
    DotGrid,
    /// Two sets of crossing diagonal lines.
    Crosshatch,
    /// Short planks laid in a herringbone.
    Herringbone,
    /// Stacked wavy lines.
    Ripple,
    /// Wavy wood-grain lines with knots.
    Woodgrain,
    /// Fine laid lines crossed by sparse chain lines.
    Laid,
    /// No pattern: bare paper.
    Plain,
}

impl LabelTexture {
    /// The hash pool: the textures a recipe with no override is picked from.
    /// Growing this list reshuffles every recipe without a chosen texture,
    /// so change it only deliberately. `Plain` is deliberately left out, so
    /// it is reachable only through a recipe's override.
    pub const ALL: [Self; 14] = [
        Self::Kraft,
        Self::Linen,
        Self::Halftone,
        Self::Pinstripe,
        Self::Contour,
        Self::Burlap,
        Self::Graph,
        Self::Ledger,
        Self::DotGrid,
        Self::Crosshatch,
        Self::Herringbone,
        Self::Ripple,
        Self::Woodgrain,
        Self::Laid,
    ];

    /// Every texture a recipe can choose, in gallery order: the hashed
    /// textures, then `Plain`.
    pub const CHOICES: [Self; 15] = [
        Self::Kraft,
        Self::Linen,
        Self::Halftone,
        Self::Pinstripe,
        Self::Contour,
        Self::Burlap,
        Self::Graph,
        Self::Ledger,
        Self::DotGrid,
        Self::Crosshatch,
        Self::Herringbone,
        Self::Ripple,
        Self::Woodgrain,
        Self::Laid,
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
            Self::Graph => "Graph",
            Self::Ledger => "Ledger",
            Self::DotGrid => "Dot grid",
            Self::Crosshatch => "Crosshatch",
            Self::Herringbone => "Herringbone",
            Self::Ripple => "Ripple",
            Self::Woodgrain => "Woodgrain",
            Self::Laid => "Laid paper",
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
            Self::Graph => "graph",
            Self::Ledger => "ledger",
            Self::DotGrid => "dot_grid",
            Self::Crosshatch => "crosshatch",
            Self::Herringbone => "herringbone",
            Self::Ripple => "ripple",
            Self::Woodgrain => "woodgrain",
            Self::Laid => "laid",
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
