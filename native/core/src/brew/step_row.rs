use crate::brew::StepStatus;

/// One line of the brew page's step list, ready to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRow {
    /// 1-based position, shown while the step is pending (`1`, `2`, …).
    pub number_label: String,
    pub name: String,
    /// `Ns`.
    pub duration_label: String,
    pub status: StepStatus,
}
