import 'dart:math' as math;

import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:coffee_timer/widgets/label_paper_painter.dart';
import 'package:coffee_timer/widgets/roast_meter.dart';
import 'package:flutter/material.dart';

/// One row in the recipe list, drawn as a printed coffee-bag label: the name
/// in a stamped badge, the description as a tagline, the brew figures in a
/// spec grid. Pure cosmetics: the figures arrive as labels computed in Rust
/// (`20 g`, `1:35`) and the look as a [RecipeLabel]; only the captions are
/// UI text.
class RecipeSummaryTile extends StatelessWidget {
  const RecipeSummaryTile({
    required this.name,
    required this.description,
    required this.coffeeLabel,
    required this.waterLabel,
    required this.timeLabel,
    required this.label,
    this.onTap,
    this.onLongPress,
    super.key,
  });

  final String name;
  final String description;
  final String coffeeLabel;
  final String waterLabel;
  final String timeLabel;
  final RecipeLabel label;
  final VoidCallback? onTap;
  final VoidCallback? onLongPress;

  @override
  Widget build(BuildContext context) {
    final colors = LabelColors.of(label, Theme.of(context).brightness);
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      onLongPress: onLongPress,
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 6, horizontal: 16),
        child: ClipPath(
          clipper: const _NotchedCorners(radius: 9),
          child: CustomPaint(
            painter: LabelPaperPainter(
              colors: colors,
              texture: label.texture,
              stain: label.stain,
            ),
            child: Stack(
              children: [
                Padding(
                  padding: const EdgeInsets.fromLTRB(20, 18, 20, 16),
                  child: _LabelText(
                    colors: colors,
                    name: name,
                    description: description,
                    coffeeLabel: coffeeLabel,
                    waterLabel: waterLabel,
                    timeLabel: timeLabel,
                  ),
                ),
                Positioned(
                  top: 14,
                  right: 16,
                  child: RoastMeter(
                    filled: switch (label.tone) {
                      LabelTone.roast => 3,
                      LabelTone.caramel => 2,
                      LabelTone.crema => 1,
                    },
                    accent: colors.accent,
                    ink: colors.ink,
                    paper: colors.paper,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _LabelText extends StatelessWidget {
  const _LabelText({
    required this.colors,
    required this.name,
    required this.description,
    required this.coffeeLabel,
    required this.waterLabel,
    required this.timeLabel,
  });

  final LabelColors colors;
  final String name;
  final String description;
  final String coffeeLabel;
  final String waterLabel;
  final String timeLabel;

  @override
  Widget build(BuildContext context) {
    final diamond = Transform.rotate(
      angle: math.pi / 4,
      child: SizedBox.square(
        dimension: 5,
        child: ColoredBox(color: colors.accent),
      ),
    );
    return Column(
      children: [
        // Clear of the roast meter on the right, centred by the same inset
        // on the left.
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 34),
          child: DecoratedBox(
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
                        fontSize: 22,
                        height: 1.1,
                        fontWeight: FontWeight.w600,
                        fontVariations: const [FontVariation.weight(600)],
                        letterSpacing: 4.4,
                        color: colors.ink,
                      ),
                    ),
                  ),
                  const SizedBox(width: 6),
                  diamond,
                ],
              ),
            ),
          ),
        ),
        if (description.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(top: 6),
            child: Text(
              '— $description —',
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
            ),
          ),
        const SizedBox(height: 12),
        Divider(
          height: 1,
          thickness: 1,
          color: colors.ink.withValues(alpha: 0.3),
        ),
        const SizedBox(height: 10),
        IntrinsicHeight(
          child: Row(
            children: [
              _Spec(colors: colors, caption: 'COFFEE', value: coffeeLabel),
              _Rule(colors: colors),
              _Spec(colors: colors, caption: 'WATER', value: waterLabel),
              _Rule(colors: colors),
              _Spec(
                colors: colors,
                caption: 'TIME',
                value: timeLabel,
                highlight: true,
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _Spec extends StatelessWidget {
  const _Spec({
    required this.colors,
    required this.caption,
    required this.value,
    this.highlight = false,
  });

  final LabelColors colors;
  final String caption;
  final String value;
  final bool highlight;

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Column(
        children: [
          Text(
            caption,
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
            value,
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
              color: highlight ? colors.accent : colors.ink,
            ),
          ),
        ],
      ),
    );
  }
}

class _Rule extends StatelessWidget {
  const _Rule({required this.colors});

  final LabelColors colors;

  @override
  Widget build(BuildContext context) {
    return VerticalDivider(
      width: 1,
      thickness: 1,
      color: colors.ink.withValues(alpha: 0.25),
    );
  }
}

/// A rectangle with a quarter-circle bitten out of each corner, like a
/// ticket.
class _NotchedCorners extends CustomClipper<Path> {
  const _NotchedCorners({required this.radius});

  final double radius;

  @override
  Path getClip(Size size) {
    final notches = Path();
    for (final corner in [
      Offset.zero,
      Offset(size.width, 0),
      Offset(0, size.height),
      Offset(size.width, size.height),
    ]) {
      notches.addOval(Rect.fromCircle(center: corner, radius: radius));
    }
    return Path.combine(
      PathOperation.difference,
      Path()..addRect(Offset.zero & size),
      notches,
    );
  }

  @override
  bool shouldReclip(_NotchedCorners oldClipper) => oldClipper.radius != radius;
}
