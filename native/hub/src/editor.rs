//! Editor state ↔ editor signals.

use coffee_timer_core::editor::{
    Draft, DurationNudge, EditorState, EditorStepRow, StepPreset, TextureOption,
};

use crate::signals::{
    AddStepButton, DurationNudgeButton, DurationNudgeChoice, EditorDraft, EditorStep,
    EditorTexture, EditorUpdate, StepPresetChoice,
};

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
            pour_speed: draft.pour_speed().to_owned(),
            pour_speed_error: draft.pour_speed_error(),
            grind: draft.grind().to_owned(),
            grind_error: draft.grind_error(),
            textures: draft
                .texture_options()
                .into_iter()
                .map(EditorTexture::from)
                .collect(),
            total_label: draft.total_label(),
            pour_hint: draft.pour_hint(),
            steps: draft
                .step_rows()
                .into_iter()
                .map(EditorStep::from)
                .collect(),
            add_step_buttons: StepPreset::ALL
                .into_iter()
                .map(|preset| AddStepButton {
                    preset: preset.into(),
                    label: preset.button_label().to_owned(),
                })
                .collect(),
            duration_nudge_buttons: DurationNudge::ALL
                .into_iter()
                .map(|nudge| DurationNudgeButton {
                    nudge: nudge.into(),
                    label: nudge.button_label(),
                    description: nudge.description(),
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
            duration_revision: row.duration_revision,
            water_text: row.water_text,
            water_error: row.water_error,
        }
    }
}

impl From<TextureOption> for EditorTexture {
    fn from(option: TextureOption) -> Self {
        Self {
            texture: option.texture.map(Into::into),
            paper: option.paper.into(),
            label: option.label.to_owned(),
            chosen: option.chosen,
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

impl From<DurationNudge> for DurationNudgeChoice {
    fn from(nudge: DurationNudge) -> Self {
        match nudge {
            DurationNudge::MinusLarge => Self::MinusLarge,
            DurationNudge::MinusSmall => Self::MinusSmall,
            DurationNudge::PlusSmall => Self::PlusSmall,
            DurationNudge::PlusLarge => Self::PlusLarge,
        }
    }
}

impl From<DurationNudgeChoice> for DurationNudge {
    fn from(choice: DurationNudgeChoice) -> Self {
        match choice {
            DurationNudgeChoice::MinusLarge => Self::MinusLarge,
            DurationNudgeChoice::MinusSmall => Self::MinusSmall,
            DurationNudgeChoice::PlusSmall => Self::PlusSmall,
            DurationNudgeChoice::PlusLarge => Self::PlusLarge,
        }
    }
}

#[cfg(test)]
mod tests {
    use coffee_timer_core::model::LabelTexture;

    use crate::signals::LabelTexture as ShownTexture;

    #[test]
    fn every_choice_survives_the_bridge() {
        for texture in LabelTexture::CHOICES {
            let shown = ShownTexture::from(texture);
            assert_eq!(LabelTexture::from(shown), texture);
        }
    }
}
