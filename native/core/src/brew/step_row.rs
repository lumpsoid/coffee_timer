use crate::brew::StepStatus;

/// One line of the brew page's step list, ready to show.
#[derive(Debug, Clone, PartialEq)]
pub struct StepRow {
    /// 1-based position, shown while the step is pending (`1`, `2`, …).
    pub number_label: String,
    pub name: String,
    /// `Ns`.
    pub duration_label: String,
    /// The water this step pours at the water shown, `50g`; none for a step
    /// that pours nothing.
    pub pour_label: Option<String>,
    pub status: StepStatus,
    /// Where the step starts on the dial's step track, as a fraction of the
    /// whole brew (`0.0..=1.0`).
    pub track_start: f64,
    /// Where it ends; the next step starts here, the last ends at `1.0`.
    pub track_end: f64,
}
