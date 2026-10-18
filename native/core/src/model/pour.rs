/// What one step pours in one brew, with the scale's total so far: the
/// step's own grams and the grams the brewer should have on the scale once
/// it is poured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pour {
    grams: u32,
    to_grams: u32,
}

impl Pour {
    pub(crate) fn new(grams: u32, to_grams: u32) -> Self {
        Self { grams, to_grams }
    }

    /// Grams poured in this step.
    pub fn grams(self) -> u32 {
        self.grams
    }

    /// Grams of water poured so far once this step is done: this step's
    /// and every earlier step's.
    pub fn to_grams(self) -> u32 {
        self.to_grams
    }

    /// This step's pour, `50g`.
    pub fn label(self) -> String {
        format!("{}g", self.grams)
    }

    /// The scale reading to pour up to, `Pour to 150g`.
    pub fn to_label(self) -> String {
        format!("Pour to {}g", self.to_grams)
    }

    /// The scale reading once the whole brew has poured, `Poured to 300g`.
    pub fn poured_to_label(self) -> String {
        format!("Poured to {}g", self.to_grams)
    }
}
