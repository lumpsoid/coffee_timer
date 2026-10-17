/// Where a step stands in the brew. Every step is `Pending` while idle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Done,
    Current,
    Pending,
}
