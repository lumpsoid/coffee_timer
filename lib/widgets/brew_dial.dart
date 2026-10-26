import 'dart:math' as math;

import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:flutter/material.dart';

final _snapshot = SnapshotController(allowSnapshotting: true);

/// The brew page's timer, drawn like the bezel of a kitchen scale: sixty
/// second ticks, an inner arc sweeping the current step, and an outer track
/// with one arc per step. [child] sits in the middle.
///
/// The sweep repeats once per [sweep]; a null [sweep] holds it still. It
/// starts over whenever [restartKey] changes (a new step, or a fresh start
/// of the same one). The track's arcs and their states come from [steps]
/// as Rust sent them. While a step is `paused` the sweep stops where it is,
/// drawn in faded ink instead of the accent, and goes on from there once
/// the step is current again.
class BrewDial extends StatelessWidget {
  const BrewDial({
    required this.sweep,
    required this.restartKey,
    required this.steps,
    required this.colors,
    required this.child,
    super.key,
  });

  final Duration? sweep;
  final Object restartKey;
  final List<BrewStep> steps;
  final LabelColors colors;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return AspectRatio(
      aspectRatio: 1,
      child: Stack(
        fit: StackFit.expand,
        children: [
          // The groove and its dotted ring never move: drawn once, then
          // reused under every sweep frame. A new look gets a new snapshot.
          SnapshotWidget(
            key: ValueKey(colors),
            controller: _snapshot,
            autoresize: true,
            child: CustomPaint(painter: _GroovePainter(colors: colors)),
          ),
          _SweepingDial(
            key: ValueKey(restartKey),
            sweep: sweep,
            steps: steps,
            colors: colors,
          ),
          Center(child: child),
        ],
      ),
    );
  }
}

class _SweepingDial extends StatefulWidget {
  const _SweepingDial({
    required this.sweep,
    required this.steps,
    required this.colors,
    super.key,
  });

  final Duration? sweep;
  final List<BrewStep> steps;
  final LabelColors colors;

  @override
  State<_SweepingDial> createState() => _SweepingDialState();
}

class _SweepingDialState extends State<_SweepingDial>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    vsync: this,
    duration: widget.sweep,
  );

  static bool _held(List<BrewStep> steps) =>
      steps.any((s) => s.status == BrewStepStatus.paused);

  /// Sweeps on from where the ring stands, unless there is nothing to sweep
  /// or the brew is paused.
  void _sweepOn() {
    if (widget.sweep != null && !_held(widget.steps)) _controller.repeat();
  }

  @override
  void initState() {
    super.initState();
    _sweepOn();
  }

  @override
  void didUpdateWidget(_SweepingDial oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.sweep != widget.sweep) {
      _controller
        ..duration = widget.sweep
        ..reset();
      _sweepOn();
    } else if (_held(oldWidget.steps) != _held(widget.steps)) {
      _controller.stop();
      _sweepOn();
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, _) => CustomPaint(
        painter: BrewDialPainter(
          progress: _controller.value,
          steps: widget.steps,
          colors: widget.colors,
        ),
      ),
    );
  }
}

/// The dial's fixed inner groove and the dotted ring inside it.
class _GroovePainter extends CustomPainter {
  const _GroovePainter({required this.colors});

  final LabelColors colors;

  @override
  void paint(Canvas canvas, Size size) {
    final center = size.center(Offset.zero);
    final unit = size.shortestSide / 300;
    final ink = colors.ink;
    canvas.drawCircle(
      center,
      102 * unit,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 7 * unit
        ..color = ink.withValues(alpha: 0.12),
    );
    final dots = Paint()..color = ink.withValues(alpha: 0.3);
    for (var i = 0; i < 120; i++) {
      canvas.drawCircle(
        BrewDialPainter._at(center, 92 * unit, i / 120),
        0.7 * unit,
        dots,
      );
    }
  }

  @override
  bool shouldRepaint(_GroovePainter oldDelegate) =>
      oldDelegate.colors != colors;
}

/// Paints the dial's moving parts over its groove. [progress] is how far
/// the sweep has come through the current (or paused) step, `0..1`; with
/// neither in [steps] nothing sweeps. A paused step's sweep is faded ink
/// rather than the accent.
class BrewDialPainter extends CustomPainter {
  const BrewDialPainter({
    required this.progress,
    required this.steps,
    required this.colors,
  });

  final double progress;
  final List<BrewStep> steps;
  final LabelColors colors;

  static const _ticks = 60;

  /// Gap left between neighbouring arcs on the step track, in turns.
  static const _trackGap = 0.012;

  static bool _isOn(BrewStep step) =>
      step.status == BrewStepStatus.current ||
      step.status == BrewStepStatus.paused;

  bool get _running => steps.any(_isOn);

  bool get _paused => steps.any((s) => s.status == BrewStepStatus.paused);

  @override
  void paint(Canvas canvas, Size size) {
    final center = size.center(Offset.zero);
    // Measures are in a 300-unit dial, scaled to fit.
    final unit = size.shortestSide / 300;
    final ink = colors.ink;
    final swept = _paused ? ink.withValues(alpha: 0.45) : colors.accent;
    final stroke = Paint()
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.round;

    // Bezel ticks, lit as the sweep passes them.
    for (var i = 0; i < _ticks; i++) {
      final long = i % 5 == 0;
      final lit = _running && i / _ticks < progress;
      canvas.drawLine(
        _at(center, 124 * unit, i / _ticks),
        _at(center, (long ? 112 : 117) * unit, i / _ticks),
        stroke
          ..strokeWidth = (long ? 2 : 1.2) * unit
          ..color = lit ? swept : ink.withValues(alpha: long ? 0.55 : 0.28),
      );
    }

    // The step track.
    final track = Rect.fromCircle(center: center, radius: 140 * unit);
    stroke.strokeWidth = 5 * unit;
    for (final step in steps) {
      final start = step.trackStart + _trackGap / 2;
      final length = step.trackEnd - step.trackStart - _trackGap;
      if (length <= 0) continue;
      _arc(
        canvas,
        track,
        start,
        length,
        stroke
          ..color = ink.withValues(
            alpha: step.status == BrewStepStatus.done ? 0.8 : 0.18,
          ),
      );
      if (_isOn(step) && progress > 0) {
        _arc(
          canvas,
          track,
          start,
          length * progress,
          stroke..color = swept,
        );
      }
    }

    // The sweep through the current step.
    if (_running && progress > 0) {
      _arc(
        canvas,
        Rect.fromCircle(center: center, radius: 102 * unit),
        0,
        progress,
        stroke
          ..strokeWidth = 7 * unit
          ..color = swept,
      );
    }
  }

  /// The point [turns] of the way round a circle of [radius], clockwise
  /// from twelve o'clock.
  static Offset _at(Offset center, double radius, double turns) {
    final angle = turns * 2 * math.pi;
    return center + Offset(math.sin(angle), -math.cos(angle)) * radius;
  }

  static void _arc(
    Canvas canvas,
    Rect rect,
    double startTurns,
    double turns,
    Paint paint,
  ) {
    canvas.drawArc(
      rect,
      (startTurns - 0.25) * 2 * math.pi,
      turns * 2 * math.pi,
      false,
      paint,
    );
  }

  @override
  bool shouldRepaint(BrewDialPainter oldDelegate) =>
      oldDelegate.progress != progress ||
      oldDelegate.steps != steps ||
      oldDelegate.colors != colors;
}
