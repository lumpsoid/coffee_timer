/// How the brew page changes the ratio of the open recipe, for this brew
/// only. The coffee stays; the water follows the ratio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RatioChange {
    /// One step more water per gram of coffee, `1:5` → `1:5.5`.
    MoreWater,
    /// One step less water per gram of coffee, `1:5` → `1:4.5`.
    LessWater,
    /// Back to the recipe's own ratio.
    Recipe,
}
