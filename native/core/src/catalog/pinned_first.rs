use crate::model::{Pin, Recipe};

/// The recipe list's order: the pinned recipes first, then the unpinned
/// ones, each group in catalog (stored) order. A stable partition, so
/// pinning or unpinning a recipe moves only that recipe, and an unpinned
/// recipe goes back to where it stood among the unpinned.
pub struct PinnedFirst;

impl PinnedFirst {
    /// `recipes` in list order, reading each one's pin from `pin_of`.
    pub fn order(recipes: &[Recipe], pin_of: impl Fn(&Recipe) -> Pin) -> Vec<&Recipe> {
        let (pinned, unpinned): (Vec<&Recipe>, Vec<&Recipe>) = recipes
            .iter()
            .partition(|recipe| pin_of(recipe).is_pinned());
        pinned.into_iter().chain(unpinned).collect()
    }
}
