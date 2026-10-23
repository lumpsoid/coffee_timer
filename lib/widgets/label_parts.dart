import 'dart:math' as math;

import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:flutter/material.dart';

/// The recipe name in a stamped badge: spaced capitals between two accent
/// rules, a diamond at each end. Shared by the list card and the brew page.
class LabelBadge extends StatelessWidget {
  const LabelBadge({
    required this.name,
    required this.colors,
    this.fontSize = 22,
    super.key,
  });

  final String name;
  final LabelColors colors;
  final double fontSize;

  @override
  Widget build(BuildContext context) {
    final diamond = Transform.rotate(
      angle: math.pi / 4,
      child: SizedBox.square(
        dimension: 5,
        child: ColoredBox(color: colors.accent),
      ),
    );
    return DecoratedBox(
      decoration: BoxDecoration(
        border: Border.symmetric(
          horizontal: BorderSide(color: colors.accent, width: 2),
        ),
      ),
      child: Padding(
        padding: const EdgeInsets.fromLTRB(4, 5, 4, 4),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            diamond,
            const SizedBox(width: 10),
            Flexible(
              child: Text(
                name.toUpperCase(),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  fontFamily: 'Oswald',
                  fontSize: fontSize,
                  height: 1.1,
                  fontWeight: FontWeight.w600,
                  fontVariations: const [FontVariation.weight(600)],
                  letterSpacing: fontSize * 0.2,
                  color: colors.ink,
                ),
              ),
            ),
            const SizedBox(width: 6),
            diamond,
          ],
        ),
      ),
    );
  }
}

/// A small push pin inked in the label's accent, tilted as if stuck in by
/// hand: the mark of a pinned recipe.
class LabelPinMark extends StatelessWidget {
  const LabelPinMark({required this.colors, super.key});

  final LabelColors colors;

  @override
  Widget build(BuildContext context) {
    return Transform.rotate(
      angle: math.pi / 6,
      child: Icon(
        Icons.push_pin,
        size: 15,
        color: colors.accent.withValues(alpha: 0.9),
        semanticLabel: 'Pinned',
      ),
    );
  }
}

/// The description as a tagline between dashes.
class LabelTagline extends StatelessWidget {
  const LabelTagline({required this.text, required this.colors, super.key});

  final String text;
  final LabelColors colors;

  @override
  Widget build(BuildContext context) {
    return Text(
      '— $text —',
      maxLines: 1,
      overflow: TextOverflow.ellipsis,
      textAlign: TextAlign.center,
      style: TextStyle(
        fontFamily: 'Fraunces',
        fontStyle: FontStyle.italic,
        fontSize: 15,
        height: 1.2,
        fontVariations: const [
          FontVariation.weight(400),
          FontVariation.opticalSize(15),
        ],
        color: colors.ink.withValues(alpha: 0.85),
      ),
    );
  }
}

/// A spaced caption between two hairlines, heading a section: the brew
/// page's method, and the sections of the app's own pages in [ink].
class RuledHeading extends StatelessWidget {
  const RuledHeading({required this.text, required this.ink, super.key});

  final String text;
  final Color ink;

  @override
  Widget build(BuildContext context) {
    final rule = Divider(
      height: 1,
      thickness: 1,
      color: ink.withValues(alpha: 0.35),
    );
    return Row(
      children: [
        Expanded(child: rule),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 10),
          child: Text(
            text.toUpperCase(),
            style: TextStyle(
              fontFamily: 'Oswald',
              fontSize: 11,
              height: 1,
              letterSpacing: 11 * 0.24,
              fontWeight: FontWeight.w500,
              fontVariations: const [FontVariation.weight(500)],
              color: ink.withValues(alpha: 0.7),
            ),
          ),
        ),
        Expanded(child: rule),
      ],
    );
  }
}

/// One figure in a [LabelSpecs] row: a small caption over its value.
typedef LabelSpec = ({String caption, String value, bool highlight});

/// A row of figures split by thin rules, like the spec grid on a bag.
class LabelSpecs extends StatelessWidget {
  const LabelSpecs({required this.specs, required this.colors, super.key});

  final List<LabelSpec> specs;
  final LabelColors colors;

  @override
  Widget build(BuildContext context) {
    final rule = VerticalDivider(
      width: 1,
      thickness: 1,
      color: colors.ink.withValues(alpha: 0.25),
    );
    return IntrinsicHeight(
      child: Row(
        children: [
          for (final (i, spec) in specs.indexed) ...[
            if (i > 0) rule,
            Expanded(
              child: _Spec(colors: colors, spec: spec),
            ),
          ],
        ],
      ),
    );
  }
}

class _Spec extends StatelessWidget {
  const _Spec({required this.colors, required this.spec});

  final LabelColors colors;
  final LabelSpec spec;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(
          spec.caption,
          style: TextStyle(
            fontFamily: 'Oswald',
            fontSize: 10,
            fontWeight: FontWeight.w500,
            fontVariations: const [FontVariation.weight(500)],
            letterSpacing: 2.2,
            color: colors.ink.withValues(alpha: 0.7),
          ),
        ),
        const SizedBox(height: 5),
        Text(
          spec.value,
          style: TextStyle(
            fontFamily: 'Fraunces',
            fontSize: 18,
            height: 1,
            fontWeight: FontWeight.w600,
            fontVariations: const [
              FontVariation.weight(600),
              FontVariation.opticalSize(18),
            ],
            fontFeatures: const [FontFeature.tabularFigures()],
            color: spec.highlight ? colors.accent : colors.ink,
          ),
        ),
      ],
    );
  }
}
