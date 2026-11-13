import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:material_ui/material_ui.dart';

/// The inks of one printed recipe label: paper, text, accent, and the
/// colour its texture and coffee stain are drawn in. Dark theme shows the
/// same label dimmed, as if the room lights were low.
@immutable
class LabelColors {
  const LabelColors._({
    required this.paper,
    required this.ink,
    required this.accent,
    required this.stain,
  });

  factory LabelColors.of(RecipeLabel label, Brightness brightness) {
    final colors = switch (label.tone) {
      LabelTone.roast => LabelColors._(
        paper: switch (label.shade) {
          LabelShade.base => const Color(0xFF3D2A1F), // dark chocolate
          LabelShade.darker => const Color(0xFF2A211D), // espresso
          LabelShade.lighter => const Color(0xFF4B2B22), // mahogany
        },
        ink: const Color(0xFFF2E4CF),
        accent: const Color(0xFFD39A58),
        stain: const Color(0xFFE6BE8C),
      ),
      LabelTone.caramel => LabelColors._(
        paper: switch (label.shade) {
          LabelShade.base => const Color(0xFFD7A66B),
          LabelShade.darker => const Color(0xFFC99156), // toffee
          LabelShade.lighter => const Color(0xFFDCB07E), // honey
        },
        ink: const Color(0xFF2F1D10),
        accent: const Color(0xFF7A4520),
        stain: const Color(0xFF5A3214),
      ),
      LabelTone.crema => LabelColors._(
        paper: switch (label.shade) {
          LabelShade.base => const Color(0xFFF1E2C6),
          LabelShade.darker => const Color(0xFFEAD7B4), // oat
          LabelShade.lighter => const Color(0xFFF4E7D6), // milk
        },
        ink: const Color(0xFF4A2E1B),
        accent: const Color(0xFFB06F36),
        stain: const Color(0xFF6E4623),
      ),
    };
    return brightness == Brightness.dark ? colors._dimmed() : colors;
  }

  final Color paper;
  final Color ink;
  final Color accent;
  final Color stain;

  @override
  bool operator ==(Object other) =>
      other is LabelColors &&
      other.paper == paper &&
      other.ink == ink &&
      other.accent == accent &&
      other.stain == stain;

  @override
  int get hashCode => Object.hash(paper, ink, accent, stain);

  LabelColors _dimmed() => LabelColors._(
    paper: _dim(paper),
    ink: _dim(ink),
    accent: _dim(accent),
    stain: _dim(stain),
  );

  static Color _dim(Color color) =>
      Color.lerp(color, const Color(0xFF000000), 0.2) ?? color;
}
