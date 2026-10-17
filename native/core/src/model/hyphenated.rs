/// `value` in the UUID text layout: 32 lowercase hex digits grouped
/// 8-4-4-4-12 by hyphens. The one home for that layout, shared by the id
/// types' `from_u128`.
pub(crate) fn hyphenated(value: u128) -> String {
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        value >> 96,
        (value >> 80) & 0xFFFF,
        (value >> 64) & 0xFFFF,
        (value >> 48) & 0xFFFF,
        value & 0xFFFF_FFFF_FFFF
    )
}
