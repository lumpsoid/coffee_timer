use crate::model::ModelError;

/// A 32-bit ARGB accent colour, valid by construction. Build one via
/// [`AccentColor::try_from_argb`] or [`AccentColor::try_from_hex`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccentColor(u32);

impl AccentColor {
    /// Material Blue 500 — the default accent.
    pub const FALLBACK: Self = Self(0xFF21_96F3);

    /// Rejects anything outside `0x00000000..=0xFFFFFFFF`.
    pub fn try_from_argb(value: i64) -> Result<Self, ModelError> {
        u32::try_from(value)
            .map(Self)
            .map_err(|_| ModelError::AccentOutOfRange(value))
    }

    /// Parses `#AARRGGBB` or `#RRGGBB` (treated as opaque); the `#` is
    /// optional.
    pub fn try_from_hex(raw: &str) -> Result<Self, ModelError> {
        let malformed = || ModelError::MalformedAccentHex(raw.to_owned());
        let hex = raw.strip_prefix('#').unwrap_or(raw);
        if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(malformed());
        }
        let full = match hex.len() {
            6 => format!("FF{hex}"),
            8 => hex.to_owned(),
            _ => return Err(malformed()),
        };
        u32::from_str_radix(&full, 16)
            .map(Self)
            .map_err(|_| malformed())
    }

    pub fn argb(self) -> u32 {
        self.0
    }

    /// Serialises to `#AARRGGBB`, upper case.
    pub fn to_hex(self) -> String {
        format!("#{:08X}", self.0)
    }
}
