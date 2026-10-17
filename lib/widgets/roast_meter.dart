import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:flutter/material.dart';

/// Three coffee beans, the first [filled] of them inked in [accent]; the
/// rest are outlines in [ink]. Shown in a recipe label's corner.
class RoastMeter extends StatelessWidget {
  const RoastMeter({
    required this.filled,
    required this.accent,
    required this.ink,
    required this.paper,
    super.key,
  });

  /// The meter for a label of [tone]: three beans for roast, two for
  /// caramel, one for crema.
  RoastMeter.of(LabelTone tone, LabelColors colors, {Key? key})
    : this(
        filled: switch (tone) {
          LabelTone.roast => 3,
          LabelTone.caramel => 2,
          LabelTone.crema => 1,
        },
        accent: colors.accent,
        ink: colors.ink,
        paper: colors.paper,
        key: key,
      );

  final int filled;
  final Color accent;
  final Color ink;
  final Color paper;

  @override
  Widget build(BuildContext context) {
    return CustomPaint(
      size: const Size(33, 12),
      painter: _BeansPainter(filled, accent, ink, paper),
    );
  }
}

class _BeansPainter extends CustomPainter {
  const _BeansPainter(this.filled, this.accent, this.ink, this.paper);

  final int filled;
  final Color accent;
  final Color ink;
  final Color paper;

  @override
  void paint(Canvas canvas, Size size) {
    for (var i = 0; i < 3; i++) {
      final on = i < filled;
      final left = i * 12.0;
      final bean = Rect.fromLTWH(left + 0.8, 0.8, 7.4, 10.4);
      final outline = Paint()
        ..style = on ? PaintingStyle.fill : PaintingStyle.stroke
        ..strokeWidth = 1
        ..color = on ? accent : ink.withValues(alpha: 0.35);
      canvas.drawOval(bean, outline);
      final crease = Path()
        ..moveTo(left + 4.6, 1.2)
        ..cubicTo(left + 3, 4, left + 6, 8, left + 4.4, 10.8);
      canvas.drawPath(
        crease,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1.1
          ..color = on ? paper : ink.withValues(alpha: 0.35),
      );
    }
  }

  @override
  bool shouldRepaint(_BeansPainter oldDelegate) =>
      oldDelegate.filled != filled ||
      oldDelegate.accent != accent ||
      oldDelegate.ink != ink ||
      oldDelegate.paper != paper;
}
