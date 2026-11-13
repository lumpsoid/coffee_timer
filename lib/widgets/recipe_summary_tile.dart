import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:coffee_timer/widgets/label_paper.dart';
import 'package:coffee_timer/widgets/label_parts.dart';
import 'package:coffee_timer/widgets/roast_meter.dart';
import 'package:flutter/material.dart';

/// One row in the recipe list, drawn as a printed coffee-bag label: the name
/// in a stamped badge, the description as a tagline, the brew figures in a
/// spec grid. A pinned recipe carries a small pin in the corner opposite
/// the roast meter. Pure cosmetics: the figures arrive as labels computed
/// in Rust (`20 g`, `1:35`, `22`) and the look as a [RecipeLabel]; only
/// the captions are UI text. A recipe without a grind shows no CLICKS spec.
class RecipeSummaryTile extends StatelessWidget {
  const RecipeSummaryTile({
    required this.name,
    required this.description,
    required this.coffeeLabel,
    required this.waterLabel,
    required this.timeLabel,
    required this.label,
    this.grindLabel,
    this.pinned = false,
    this.onTap,
    this.onLongPress,
    super.key,
  });

  final String name;
  final String description;
  final String coffeeLabel;
  final String waterLabel;
  final String timeLabel;
  final String? grindLabel;
  final RecipeLabel label;

  /// Whether to show the pin marker.
  final bool pinned;
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
          child: LabelPaper(
            colors: colors,
            texture: label.texture,
            // No ring on list cards for now: only the brew page has one.
            stain: null,
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
                    grindLabel: grindLabel,
                  ),
                ),
                Positioned(
                  top: 14,
                  right: 16,
                  child: RoastMeter.of(label.tone, colors),
                ),
                if (pinned)
                  Positioned(
                    top: 11,
                    left: 16,
                    child: LabelPinMark(colors: colors),
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
    required this.grindLabel,
  });

  final LabelColors colors;
  final String name;
  final String description;
  final String coffeeLabel;
  final String waterLabel;
  final String timeLabel;
  final String? grindLabel;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // Clear of the roast meter on the right, centred by the same inset
        // on the left.
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 34),
          child: LabelBadge(name: name, colors: colors),
        ),
        if (description.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(top: 6),
            child: LabelTagline(text: description, colors: colors),
          ),
        const SizedBox(height: 12),
        Divider(
          height: 1,
          thickness: 1,
          color: colors.ink.withValues(alpha: 0.3),
        ),
        const SizedBox(height: 10),
        LabelSpecs(
          colors: colors,
          specs: [
            (caption: 'COFFEE', value: coffeeLabel, highlight: false),
            (caption: 'WATER', value: waterLabel, highlight: false),
            (caption: 'TIME', value: timeLabel, highlight: true),
            if (grindLabel case final grind?)
              (caption: 'CLICKS', value: grind, highlight: false),
          ],
        ),
      ],
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
