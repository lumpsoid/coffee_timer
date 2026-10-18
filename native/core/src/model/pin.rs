/// Whether the recipe list keeps a recipe at its top. Catalog metadata, not
/// part of the [`crate::model::Recipe`]: it says where the user keeps a
/// recipe, not how it brews, so editing, brewing and duplicating never see
/// it. A new recipe is unpinned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Pin {
    Pinned,
    #[default]
    Unpinned,
}

impl Pin {
    pub fn is_pinned(self) -> bool {
        self == Self::Pinned
    }
}

impl From<bool> for Pin {
    fn from(pinned: bool) -> Self {
        if pinned { Self::Pinned } else { Self::Unpinned }
    }
}
