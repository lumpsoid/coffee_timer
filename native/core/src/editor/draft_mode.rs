/// Whether the draft is a recipe not yet in the catalog or an existing one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftMode {
    New,
    Existing,
}
