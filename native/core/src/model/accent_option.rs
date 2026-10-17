use crate::model::AccentColor;

/// A named accent the user can pick. The closed list of choices is
/// [`AccentOption::ALL`]; the colours are Flutter's Material 500 shades
/// (`Colors.blue` … `Colors.lime`), in the legacy picker's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccentOption {
    name: &'static str,
    color: AccentColor,
}

impl AccentOption {
    pub const ALL: [Self; 10] = [
        Self::new("Blue", 0xFF21_96F3),
        Self::new("Red", 0xFFF4_4336),
        Self::new("Green", 0xFF4C_AF50),
        Self::new("Purple", 0xFF9C_27B0),
        Self::new("Orange", 0xFFFF_9800),
        Self::new("Pink", 0xFFE9_1E63),
        Self::new("Indigo", 0xFF3F_51B5),
        Self::new("Teal", 0xFF00_9688),
        Self::new("Cyan", 0xFF00_BCD4),
        Self::new("Lime", 0xFFCD_DC39),
    ];

    const fn new(name: &'static str, argb: u32) -> Self {
        Self {
            name,
            color: AccentColor::from_argb32(argb),
        }
    }

    pub fn name(self) -> &'static str {
        self.name
    }

    pub fn color(self) -> AccentColor {
        self.color
    }
}
