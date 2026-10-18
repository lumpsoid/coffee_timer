/// The "add step" buttons. Each adds a step row with the preset's name, a
/// `0` second duration and a blank grams field (no pour). No preset guesses
/// an amount: an "Add water" row is left for the brewer to type its grams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepPreset {
    AddCoffee,
    AddWater,
    Wait,
    /// A row with no name; it saves as `Step N`.
    Blank,
}

impl StepPreset {
    /// Every preset, in the legacy button order.
    pub const ALL: [Self; 4] = [Self::AddCoffee, Self::AddWater, Self::Wait, Self::Blank];

    /// The button text.
    pub fn button_label(self) -> &'static str {
        match self {
            Self::AddCoffee => "Add coffee",
            Self::AddWater => "Add water",
            Self::Wait => "Wait",
            Self::Blank => "Add Step",
        }
    }

    /// The step name the new row starts with.
    pub fn step_name(self) -> &'static str {
        match self {
            Self::AddCoffee => "Add coffee",
            Self::AddWater => "Add water",
            Self::Wait => "Wait",
            Self::Blank => "",
        }
    }
}
