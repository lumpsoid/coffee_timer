use crate::model::StepId;

/// One step card of the editor, ready to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorStepRow {
    /// Row key: address edits to the row with this id.
    pub id: StepId,
    /// Name field label, `Step N name` (1-based position).
    pub name_label: String,
    /// The name as typed.
    pub name: String,
    /// The duration as typed.
    pub duration_text: String,
    /// The typed duration as `MM:SS`, lenient: non-numbers read `00:00`,
    /// out-of-range values are clamped.
    pub duration_label: String,
    /// `Enter seconds` or `0–5999 seconds`, once errors are shown.
    pub duration_error: Option<String>,
}
