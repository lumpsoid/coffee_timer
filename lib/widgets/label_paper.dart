import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:coffee_timer/widgets/label_paper_painter.dart';
import 'package:flutter/material.dart';

final _snapshot = SnapshotController(allowSnapshotting: true);

/// [child] printed on a recipe label's paper, sized to [child].
///
/// The paper is tens of thousands of grain specks and fibres. It is painted
/// once into an image and drawn from that, since redrawing it every frame
/// (scrolling, page transitions, the brew dial sweeping) drops frames on
/// some GPUs.
class LabelPaper extends StatelessWidget {
  const LabelPaper({
    required this.colors,
    required this.texture,
    required this.stain,
    required this.child,
    super.key,
  });

  final LabelColors colors;
  final LabelTexture texture;
  final StainCorner stain;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Stack(
      fit: StackFit.passthrough,
      children: [
        Positioned.fill(
          // The snapshot is kept through repaints of its child, so a new
          // look gets a new widget; a new size, a new snapshot.
          child: SnapshotWidget(
            key: ValueKey((colors, texture, stain)),
            controller: _snapshot,
            autoresize: true,
            child: CustomPaint(
              painter: LabelPaperPainter(
                colors: colors,
                texture: texture,
                stain: stain,
              ),
            ),
          ),
        ),
        child,
      ],
    );
  }
}
