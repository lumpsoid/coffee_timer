use crate::model::LabelTexture;

/// One tile of the label texture gallery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureOption {
    /// What choosing it stores: `None` is Auto, the hash pick.
    pub texture: Option<LabelTexture>,
    /// The paper the tile shows: for Auto, the hashed texture.
    pub paper: LabelTexture,
    /// `Auto`, `Kraft`, …
    pub label: &'static str,
    pub chosen: bool,
}
