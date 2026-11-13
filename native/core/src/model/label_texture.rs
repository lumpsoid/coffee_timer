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
}
