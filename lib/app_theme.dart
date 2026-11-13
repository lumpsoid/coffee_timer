import 'package:material_ui/material_ui.dart';

/// The app's chrome set in the recipe label's type: titles, buttons and
/// captions in tracked Oswald like the label's badge and captions, running
/// text in Fraunces like its tagline and figures. Colours seed from the
/// accent setting.
ThemeData appTheme(Color seed, Brightness brightness) {
  final base = brightness == Brightness.dark
      ? ThemeData.dark()
      : ThemeData.light();
  final scheme = ColorScheme.fromSeed(
    seedColor: seed,
    brightness: brightness,
  );
  final text = base.textTheme;
  return base.copyWith(
    colorScheme: scheme,
    textTheme: text.copyWith(
      displayLarge: _oswald(text.displayLarge),
      displayMedium: _oswald(text.displayMedium),
      displaySmall: _oswald(text.displaySmall),
      headlineLarge: _oswald(text.headlineLarge),
      headlineMedium: _oswald(text.headlineMedium),
      headlineSmall: _oswald(text.headlineSmall),
      titleLarge: _oswald(text.titleLarge),
      titleMedium: _oswald(text.titleMedium),
      titleSmall: _oswald(text.titleSmall),
      labelLarge: _oswald(text.labelLarge),
      labelMedium: _oswald(text.labelMedium),
      labelSmall: _oswald(text.labelSmall),
      bodyLarge: _fraunces(text.bodyLarge),
      bodyMedium: _fraunces(text.bodyMedium),
      bodySmall: _fraunces(text.bodySmall),
    ),
    appBarTheme: base.appBarTheme.copyWith(
      centerTitle: true,
      // The label badge's spaced capitals; titles are upper-cased where set.
      titleTextStyle: TextStyle(
        fontFamily: 'Oswald',
        fontSize: 20,
        height: 1.1,
        fontWeight: FontWeight.w600,
        fontVariations: const [FontVariation.weight(600)],
        letterSpacing: 4,
        color: scheme.onSurface,
      ),
    ),
  );
}

/// Oswald at the style's own size and weight, tracked out like the label's
/// captions.
TextStyle? _oswald(TextStyle? style) {
  if (style == null) return null;
  final weight = style.fontWeight ?? FontWeight.w400;
  return style.copyWith(
    fontFamily: 'Oswald',
    fontWeight: weight,
    fontVariations: [FontVariation.weight(weight.value.toDouble())],
    letterSpacing: (style.fontSize ?? 14) * 0.08,
  );
}

TextStyle? _fraunces(TextStyle? style) {
  if (style == null) return null;
  final weight = style.fontWeight ?? FontWeight.w400;
  final size = style.fontSize ?? 14;
  return style.copyWith(
    fontFamily: 'Fraunces',
    fontWeight: weight,
    fontVariations: [
      FontVariation.weight(weight.value.toDouble()),
      FontVariation.opticalSize(size),
    ],
    letterSpacing: 0,
  );
}
