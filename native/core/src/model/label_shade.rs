/// A variation within a [`LabelTone`](crate::model::LabelTone), so labels of
/// the same tone don't all share one colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LabelShade {
    Base,
    Darker,
    Lighter,
}

impl LabelShade {
    pub const ALL: [Self; 3] = [Self::Base, Self::Darker, Self::Lighter];
}
