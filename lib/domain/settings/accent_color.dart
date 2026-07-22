/// An ARGB accent colour that is valid by construction.
///
/// There is no public constructor: build one via [tryFromArgb] or [tryFromHex],
/// which are the only entry points and reject out-of-range input. Once you hold
/// an [AccentColor] it is valid by definition — never re-check it.
class AccentColor {
  const AccentColor._(this.argb);

  /// Material Blue 500 — the default accent.
  static const fallback = AccentColor._(0xFF2196F3);

  static const _min = 0x00000000;
  static const _max = 0xFFFFFFFF;

  /// The 32-bit ARGB value, always within [_min].._max.
  final int argb;

  /// Returns an [AccentColor] for [value], or null if it is not a 32-bit ARGB.
  static AccentColor? tryFromArgb(int value) {
    if (value < _min || value > _max) {
      return null;
    }
    return AccentColor._(value);
  }

  /// Parses `#AARRGGBB` or `#RRGGBB` (treated as opaque). Null if malformed.
  static AccentColor? tryFromHex(String raw) {
    var hex = raw.startsWith('#') ? raw.substring(1) : raw;
    if (hex.length == 6) {
      hex = 'FF$hex';
    }
    if (hex.length != 8) {
      return null;
    }
    final value = int.tryParse(hex, radix: 16);
    return value == null ? null : AccentColor._(value);
  }

  /// Serialises to `#AARRGGBB`.
  String toHex() => '#${argb.toRadixString(16).padLeft(8, '0').toUpperCase()}';
}
