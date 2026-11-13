import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:material_ui/material_ui.dart';

/// One line of the brew page's method: a round stamp, the step's name, a
/// dotted leader, the water it pours (if any) and its duration, printed in
/// the recipe label's [colors]. The stamp follows [BrewStep.status]: a check
/// when done, a play arrow when current, a pause sign when paused, the step's
/// number while pending. The current line is picked out in the accent, a
/// paused one in faded ink; done lines fade.
class StepListTile extends StatelessWidget {
  const StepListTile({
    required this.step,
    required this.colors,
    required this.onLongPress,
    super.key,
  });

  final BrewStep step;
  final LabelColors colors;
  final VoidCallback onLongPress;

  @override
  Widget build(BuildContext context) {
    final status = step.status;
    final current = status == BrewStepStatus.current;
    final paused = status == BrewStepStatus.paused;
    final stampFill = switch (status) {
      BrewStepStatus.done => colors.ink,
      BrewStepStatus.current => colors.accent,
      BrewStepStatus.paused => colors.ink.withValues(alpha: 0.55),
      BrewStepStatus.pending => null,
    };
    final stamp = switch (status) {
      BrewStepStatus.done => Icon(
        Icons.check,
        key: const ValueKey('check'),
        size: 14,
        color: colors.paper,
      ),
      BrewStepStatus.current => Icon(
        Icons.play_arrow,
        key: const ValueKey('play'),
        size: 16,
        color: colors.paper,
      ),
      BrewStepStatus.paused => Icon(
        Icons.pause,
        key: const ValueKey('pause'),
        size: 16,
        color: colors.paper,
      ),
      BrewStepStatus.pending => Text(
        step.numberLabel,
        key: ValueKey('pending_${step.numberLabel}'),
        style: TextStyle(
          fontFamily: 'Oswald',
          fontSize: 12,
          fontVariations: const [FontVariation.weight(500)],
          color: colors.ink,
        ),
      ),
    };

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onLongPress: onLongPress,
      child: AnimatedOpacity(
        duration: const Duration(milliseconds: 300),
        opacity: status == BrewStepStatus.done ? 0.5 : 1,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 300),
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 9),
          decoration: BoxDecoration(
            color: switch (status) {
              BrewStepStatus.current => colors.accent.withValues(alpha: 0.18),
              BrewStepStatus.paused => colors.ink.withValues(alpha: 0.08),
              BrewStepStatus.done ||
              BrewStepStatus.pending => colors.accent.withValues(alpha: 0),
            },
            borderRadius: BorderRadius.circular(6),
          ),
          child: Row(
            children: [
              AnimatedContainer(
                duration: const Duration(milliseconds: 300),
                width: 24,
                height: 24,
                alignment: Alignment.center,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: stampFill,
                  border: Border.all(
                    width: 1.5,
                    color: stampFill ?? colors.ink.withValues(alpha: 0.55),
                  ),
                ),
                child: AnimatedSwitcher(
                  duration: const Duration(milliseconds: 200),
                  child: stamp,
                ),
              ),
              const SizedBox(width: 12),
              // The name takes what it needs (short of the whole width) and
              // the leader the rest, so every duration ends flush right.
              Expanded(
                child: LayoutBuilder(
                  builder: (context, box) => Row(
                    children: [
                      ConstrainedBox(
                        constraints: BoxConstraints(
                          maxWidth: (box.maxWidth - 24).clamp(0, box.maxWidth),
                        ),
                        child: Text(
                          step.name,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: TextStyle(
                            fontFamily: 'Fraunces',
                            fontSize: 17,
                            height: 1.2,
                            fontVariations: [
                              FontVariation.weight(
                                current || paused ? 600 : 400,
                              ),
                              const FontVariation.opticalSize(17),
                            ],
                            color: colors.ink,
                          ),
                        ),
                      ),
                      Expanded(child: _Leader(color: colors.ink)),
                    ],
                  ),
                ),
              ),
              if (step.pourLabel case final pour?)
                Padding(
                  padding: const EdgeInsets.only(right: 10),
                  child: Text(
                    pour,
                    style: TextStyle(
                      fontFamily: 'Oswald',
                      fontSize: 13,
                      letterSpacing: 0.8,
                      fontVariations: const [FontVariation.weight(500)],
                      fontFeatures: const [FontFeature.tabularFigures()],
                      color: colors.ink.withValues(alpha: 0.7),
                    ),
                  ),
                ),
              Text(
                step.durationLabel,
                style: TextStyle(
                  fontFamily: 'Oswald',
                  fontSize: 15,
                  letterSpacing: 0.9,
                  fontVariations: const [FontVariation.weight(500)],
                  fontFeatures: const [FontFeature.tabularFigures()],
                  color: current ? colors.accent : colors.ink,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// A dotted line from a step's name to its duration.
class _Leader extends StatelessWidget {
  const _Leader({required this.color});

  final Color color;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(8, 10, 8, 0),
      child: CustomPaint(
        size: const Size.fromHeight(2),
        painter: _DotsPainter(color.withValues(alpha: 0.35)),
      ),
    );
  }
}

class _DotsPainter extends CustomPainter {
  const _DotsPainter(this.color);

  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    final dot = Paint()..color = color;
    for (var x = 1.0; x < size.width; x += 4) {
      canvas.drawCircle(Offset(x, size.height / 2), 0.9, dot);
    }
  }

  @override
  bool shouldRepaint(_DotsPainter oldDelegate) => oldDelegate.color != color;
}
