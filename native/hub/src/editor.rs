//! Editor state ↔ editor signals.

use coffee_timer_core::editor::{Draft, EditorState, EditorStepRow, StepPreset};

use crate::signals::{AddStepButton, EditorDraft, EditorStep, EditorUpdate, StepPresetChoice};

impl From<&EditorState> for EditorUpdate {
    fn from(state: &EditorState) -> Self {
        match state {
            EditorState::Closed => Self::Closed,
            EditorState::Open(draft) => Self::Open {
                draft: Box::new(EditorDraft::from(&**draft)),
            },
        }
    }
}

impl From<&Draft> for EditorDraft {
    fn from(draft: &Draft) -> Self {
        Self {
            id: draft.id().as_str().to_owned(),
            title: draft.title().to_owned(),
            can_delete: draft.can_delete(),
            name: draft.name().to_owned(),
            name_error: draft.name_error(),
            description: draft.description().to_owned(),
            coffee: draft.coffee().to_owned(),
            coffee_error: draft.coffee_error(),
            water: draft.water().to_owned(),
            water_error: draft.water_error(),
            total_label: draft.total_label(),
            pour_hint: draft.pour_hint(),
            steps: draft.step_rows().into_iter().map(EditorStep::from).collect(),
            add_step_buttons: StepPreset::ALL
                .into_iter()
                .map(|preset| AddStepButton {
                    preset: preset.into(),
                    label: preset.button_label().to_owned(),
                })
                .collect(),
            notice: draft.notice().map(str::to_owned),
        }
    }
}

impl From<EditorStepRow> for EditorStep {
    fn from(row: EditorStepRow) -> Self {
        Self {
            id: row.id.as_str().to_owned(),
            name_label: row.name_label,
            name: row.name,
            duration_text: row.duration_text,
            duration_label: row.duration_label,
            duration_error: row.duration_error,
            water_text: row.water_text,
            water_error: row.water_error,
        }
    }
}

impl From<StepPreset> for StepPresetChoice {
    fn from(preset: StepPreset) -> Self {
        match preset {
            StepPreset::AddCoffee => Self::AddCoffee,
            StepPreset::AddWater => Self::AddWater,
            StepPreset::Wait => Self::Wait,
            StepPreset::Blank => Self::Blank,
        }
    }
}

impl From<StepPresetChoice> for StepPreset {
    fn from(choice: StepPresetChoice) -> Self {
        match choice {
            StepPresetChoice::AddCoffee => Self::AddCoffee,
            StepPresetChoice::AddWater => Self::AddWater,
            StepPresetChoice::Wait => Self::Wait,
            StepPresetChoice::Blank => Self::Blank,
        }
    }
}
