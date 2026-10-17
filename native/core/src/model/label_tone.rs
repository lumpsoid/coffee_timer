/// The colour family of a recipe's list label, from its total brew time:
/// short brews print dark like espresso, long brews light like filter
/// coffee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelTone {
    Roast,
    Caramel,
    Crema,
}

impl LabelTone {
    /// Brews at least this long are `Caramel`; shorter ones are `Roast`.
    pub const CARAMEL_FROM_SECONDS: u32 = 120;
    /// Brews at least this long are `Crema`.
    pub const CREMA_FROM_SECONDS: u32 = 240;

    pub fn for_brew_seconds(total_seconds: u32) -> Self {
        if total_seconds >= Self::CREMA_FROM_SECONDS {
            Self::Crema
        } else if total_seconds >= Self::CARAMEL_FROM_SECONDS {
            Self::Caramel
        } else {
            Self::Roast
        }
    }
}
