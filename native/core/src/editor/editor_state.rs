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

    /// The open draft, for an action on it. Every action on the draft starts
    /// here, so the previous action's notice is cleared whatever this one
    /// does.
    pub(crate) fn draft_mut(&mut self) -> Option<&mut Draft> {
        match self {
            Self::Closed => None,
            Self::Open(draft) => {
                draft.clear_notice();
                Some(draft)
            }
        }
    }
}
