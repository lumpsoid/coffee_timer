import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

/// The coffee for this brew: its grams over a gram tape under a fixed
/// needle. Swipe the tape sideways to change it; the tape follows the
/// finger, so dragging left brings more grams under the needle. A slow drag
/// moves one gram at a time, a fast swipe many ([gramsPerPixel]).
///
/// While dragging, the tape runs on the finger and sends each whole gram it
/// reaches through [onGrams]; Rust's reply sets [label] and, once the finger
/// lifts, the tape's place.
class CoffeeTape extends StatefulWidget {
  const CoffeeTape({
    required this.coffee,
    required this.label,
    required this.labelStyle,
    required this.colors,
    required this.onGrams,
    super.key,
  });

  final BrewCoffee coffee;

  /// The coffee shown, `12g`.
  final String label;
  final TextStyle labelStyle;
  final LabelColors colors;
  final ValueChanged<int> onGrams;

  /// Tape length of one gram.
  static const double pixelsPerGram = 12;

  /// Drag distance per gram at a slow drag.
  static const double slowPixelsPerGram = 24;

  /// Finger speeds, in px/ms, where the speed-up starts and where it tops
  /// out at [topGain] times the slow rate.
  static const double speedUpFrom = 0.35;
  static const double speedUpTo = 2;
  static const double topGain = 8;

  /// Grams moved per pixel dragged at finger `speed` (px/ms): the slow rate
  /// up to [speedUpFrom], [topGain] times it from [speedUpTo], eased
  /// between.
  static double gramsPerPixel(double speed) {
    final t = ((speed - speedUpFrom) / (speedUpTo - speedUpFrom)).clamp(
      0.0,
      1.0,
    );
    final eased = t * t * (3 - 2 * t);
    return (1 + (topGain - 1) * eased) / slowPixelsPerGram;
  }

  @override
  State<CoffeeTape> createState() => _CoffeeTapeState();
}

class _CoffeeTapeState extends State<CoffeeTape> {
  /// The grams under the needle while a finger drags the tape; between
  /// whole grams as the tape moves.
  double? _dragged;

  /// The last whole gram sent.
  int _sent = 0;

  /// Finger speed in px/ms, smoothed over moves.
  double _speed = 0;
  Duration? _lastMove;

  void _start(DragStartDetails details) {
    setState(() => _dragged = widget.coffee.grams.toDouble());
    _sent = widget.coffee.grams;
    _speed = 0;
    // The first update carries the whole touch slop at once, right after
    // the start: timing it would read as a fast swipe. Speed is measured
    // from the second update on.
    _lastMove = null;
  }

  void _update(DragUpdateDetails details) {
    final dragged = _dragged;
    if (dragged == null) return;
    final dx = details.delta.dx;
    final at = details.sourceTimeStamp;
    final last = _lastMove;
    if (at != null && last != null) {
      final ms = (at - last).inMicroseconds / 1000;
      _speed = _speed * 0.6 + dx.abs() / (ms < 1 ? 1 : ms) * 0.4;
    }
    _lastMove = at;
    final min = widget.coffee.minGrams.toDouble();
    final max = widget.coffee.maxGrams.toDouble();
    // The tape follows the finger: moving it left brings more grams under
    // the needle.
    final next = (dragged - dx * CoffeeTape.gramsPerPixel(_speed)).clamp(
      min,
      max,
    );
    setState(() => _dragged = next);
    final grams = next.round();
    if (grams != _sent) {
      _sent = grams;
      widget.onGrams(grams);
    }
  }

  void _end() => setState(() => _dragged = null);

  @override
  Widget build(BuildContext context) {
    final adjusted = widget.coffee.setting == BrewCoffeeSetting.adjusted;
    final colors = widget.colors;
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      dragStartBehavior: DragStartBehavior.down,
      onHorizontalDragStart: _start,
      onHorizontalDragUpdate: _update,
      onHorizontalDragEnd: (_) => _end(),
      onHorizontalDragCancel: _end,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            widget.label,
            style: widget.labelStyle.copyWith(
              color: adjusted ? colors.accent : colors.ink,
            ),
          ),
          const SizedBox(height: 4),
          SizedBox(
            height: 52,
            width: double.infinity,
            child: CustomPaint(
              painter: _TapePainter(
                grams: _dragged ?? widget.coffee.grams.toDouble(),
                min: widget.coffee.minGrams,
                max: widget.coffee.maxGrams,
                ink: colors.ink,
                needle: colors.accent,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// A tick per gram, a long one with its number every 5 g, fading towards
/// the edges; the needle stands at the middle over [grams].
class _TapePainter extends CustomPainter {
  _TapePainter({
    required this.grams,
    required this.min,
    required this.max,
    required this.ink,
    required this.needle,
  });

  final double grams;
  final int min;
  final int max;
  final Color ink;
  final Color needle;

  @override
  void paint(Canvas canvas, Size size) {
    final middle = size.width / 2;
    final bottom = size.height;
    final reach = (middle / CoffeeTape.pixelsPerGram).ceil() + 1;
    final tick = Paint()..strokeWidth = 1.5;
    for (
      var g = (grams - reach).floor().clamp(min, max);
      g <= (grams + reach).ceil().clamp(min, max);
      g++
    ) {
      final x = middle + (g - grams) * CoffeeTape.pixelsPerGram;
      final fade = (1 - (x - middle).abs() / middle).clamp(0.0, 1.0);
      if (fade == 0) continue;
      final major = g % 5 == 0;
      tick.color = ink.withValues(alpha: (major ? 0.9 : 0.45) * fade);
      canvas.drawLine(
        Offset(x, bottom),
        Offset(x, bottom - (major ? 20 : 11)),
        tick,
      );
      if (major) {
        final number = TextPainter(
          text: TextSpan(
            text: '$g',
            style: TextStyle(
              fontFamily: 'Oswald',
              fontSize: 11,
              color: ink.withValues(alpha: 0.65 * fade),
            ),
          ),
          textDirection: TextDirection.ltr,
        )..layout();
        number.paint(
          canvas,
          Offset(x - number.width / 2, bottom - 22 - number.height),
        );
      }
    }
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(middle - 1.5, bottom - 30, 3, 30),
        const Radius.circular(1.5),
      ),
      Paint()..color = needle,
    );
  }

  @override
  bool shouldRepaint(_TapePainter old) =>
      old.grams != grams ||
      old.min != min ||
      old.max != max ||
      old.ink != ink ||
      old.needle != needle;
}
