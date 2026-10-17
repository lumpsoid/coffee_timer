/// Where the faint coffee-ring stain sits on a recipe's list label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StainCorner {
    BottomRight,
    TopLeft,
}

impl StainCorner {
    pub const ALL: [Self; 2] = [Self::BottomRight, Self::TopLeft];
}
