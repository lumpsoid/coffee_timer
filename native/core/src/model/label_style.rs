use crate::model::{
    BrewMetrics, LabelShade, LabelTexture, LabelTone, Recipe, RecipeId, StainCorner,
};

/// How a recipe looks in the list, as a printed coffee-bag label.
///
/// The tone follows total brew time. Texture, shade and stain corner are
/// picked from a stable hash of the recipe id, so recipes of the same tone
/// still look different, and a recipe keeps its look when renamed or
/// reordered. The hash is FNV-1a, fixed here rather than `std`'s hasher,
/// so a label never changes between builds. A texture the recipe chose
/// (see [`Recipe::with_label_texture`]) replaces the hashed one; the hash
/// still consumes its texture bits, so shade and stain do not move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabelStyle {
    tone: LabelTone,
    texture: LabelTexture,
    shade: LabelShade,
    stain: StainCorner,
}

impl LabelStyle {
    pub fn of(recipe: &Recipe, metrics: &BrewMetrics) -> Self {
        let mut bits = usize::try_from(fnv1a(recipe.id().as_str())).unwrap_or_default();
        // Taken even when overridden: the shade and stain bits must not move.
        let hashed = take(&LabelTexture::ALL, &mut bits);
        Self {
            tone: LabelTone::for_brew_seconds(metrics.total_seconds(recipe)),
            texture: recipe.label_texture().unwrap_or(hashed),
            shade: take(&LabelShade::ALL, &mut bits),
            stain: take(&StainCorner::ALL, &mut bits),
        }
    }

    /// The texture the hash gives `id`: what a recipe with no chosen texture
    /// is printed on.
    pub fn hashed_texture(id: &RecipeId) -> LabelTexture {
        let mut bits = usize::try_from(fnv1a(id.as_str())).unwrap_or_default();
        take(&LabelTexture::ALL, &mut bits)
    }

    pub fn tone(&self) -> LabelTone {
        self.tone
    }

    pub fn texture(&self) -> LabelTexture {
        self.texture
    }

    pub fn shade(&self) -> LabelShade {
        self.shade
    }

    pub fn stain(&self) -> StainCorner {
        self.stain
    }
}

/// 32-bit FNV-1a over `text`'s UTF-8 bytes.
fn fnv1a(text: &str) -> u32 {
    text.bytes().fold(0x811C_9DC5, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    })
}

/// The choice `bits` selects from `all`, consuming that choice's share of
/// `bits` so the next pick is independent of this one.
fn take<T: Copy, const N: usize>(all: &[T; N], bits: &mut usize) -> T {
    let choice = all[*bits % N];
    *bits /= N;
    choice
}
