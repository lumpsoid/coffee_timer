import 'package:coffee_timer/domain/settings/accent_color.dart';
import 'package:coffee_timer/domain/settings/settings.dart';
import 'package:coffee_timer/domain/settings/theme_selection.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('ThemeSelection.fromStorage', () {
    test('parses known values', () {
      expect(ThemeSelection.fromStorage('light'), ThemeSelection.light);
      expect(ThemeSelection.fromStorage('dark'), ThemeSelection.dark);
      expect(ThemeSelection.fromStorage('system'), ThemeSelection.system);
    });

    test('falls back to system for unknown or missing', () {
      expect(ThemeSelection.fromStorage('nope'), ThemeSelection.system);
      expect(ThemeSelection.fromStorage(null), ThemeSelection.system);
    });
  });

  group('AccentColor', () {
    test('round-trips a full hex string', () {
      final color = AccentColor.tryFromHex('#FF2196F3');
      expect(color, isNotNull);
      expect(color!.argb, 0xFF2196F3);
      expect(color.toHex(), '#FF2196F3');
    });

    test('pads a 6-digit hex to opaque', () {
      expect(AccentColor.tryFromHex('#2196F3')?.argb, 0xFF2196F3);
    });

    test('rejects malformed hex', () {
      expect(AccentColor.tryFromHex('xyz'), isNull);
      expect(AccentColor.tryFromHex('#12'), isNull);
    });

    test('rejects out-of-range argb', () {
      expect(AccentColor.tryFromArgb(-1), isNull);
      expect(AccentColor.tryFromArgb(0x1FFFFFFFF), isNull);
    });
  });

  group('Settings', () {
    test('withTheme changes only the theme', () {
      const base = Settings.fallback;
      final next = base.withTheme(ThemeSelection.dark);
      expect(next.theme, ThemeSelection.dark);
      expect(next.accent, base.accent);
    });

    test('withAccent changes only the accent', () {
      const base = Settings.fallback;
      final blue = AccentColor.tryFromArgb(0xFF0000FF)!;
      final next = base.withAccent(blue);
      expect(next.accent, blue);
      expect(next.theme, base.theme);
    });
  });
}
