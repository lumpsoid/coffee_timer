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
}

impl LabelTexture {
    pub const ALL: [Self; 6] = [
        Self::Kraft,
        Self::Linen,
        Self::Halftone,
        Self::Pinstripe,
        Self::Contour,
        Self::Burlap,
    ];
}
