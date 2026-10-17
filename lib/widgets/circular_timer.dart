import 'package:coffee_timer/widgets/durational_circular_progress_indicator.dart';
import 'package:flutter/material.dart';

/// [child] inside a ring that sweeps once per [sweep], repeating; a null
/// [sweep] holds the ring still. The sweep starts over whenever
/// [restartKey] changes (a new step, or a fresh start of the same one).
class CircularTimer extends StatelessWidget {
  const CircularTimer({
    required this.sweep,
    required this.restartKey,
    required this.child,
    this.strokeWidth = 10,
    this.backgroundColor = const Color(0xFF424242),
    this.progressColor = Colors.greenAccent,
    this.constraints,
    super.key,
  });

  final Duration? sweep;
  final Object restartKey;
  final Widget child;
  final double strokeWidth;
  final Color backgroundColor;
  final Color progressColor;
  final BoxConstraints? constraints;

  @override
  Widget build(BuildContext context) {
    return Stack(
      alignment: Alignment.center,
      children: [
        DurationalCircularProgressIndicator(
          key: ValueKey(restartKey),
          duration: sweep,
          strokeWidth: strokeWidth,
          backgroundColor: backgroundColor,
          progressColor: progressColor,
          constraints: constraints,
        ),
        child,
      ],
    );
  }
}
