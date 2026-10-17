use crate::editor::Draft;

/// The whole editor page. `Closed` after a save or delete succeeds: the page
/// is done and goes back.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum EditorState {
    /// No draft: never opened, closed, saved, or deleted.
    #[default]
    Closed,
    Open(Draft),
}

impl EditorState {
    pub fn draft(&self) -> Option<&Draft> {
        match self {
            Self::Closed => None,
            Self::Open(draft) => Some(draft),
        }
    }

    pub(crate) fn draft_mut(&mut self) -> Option<&mut Draft> {
        match self {
            Self::Closed => None,
            Self::Open(draft) => Some(draft),
        }
    }
}
