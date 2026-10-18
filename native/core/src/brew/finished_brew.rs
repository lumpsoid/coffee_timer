/// How the last run of the brew ended, kept beside the view in
/// [`crate::brew::BrewState::Finished`] until the next start, reset, or
/// reopen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedBrew {
    total_label: String,
}

impl FinishedBrew {
    pub(crate) fn new(total_label: String) -> Self {
        Self { total_label }
    }

    /// The time the run brewed, `M:SS`: the steps from the one it started
    /// at to the last (the whole recipe's total for a full brew).
    pub fn total_label(&self) -> &str {
        &self.total_label
    }
}
