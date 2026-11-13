import 'dart:math' as math;
import 'dart:ui' show PointMode;

import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/coffee_ring.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:flutter/material.dart';

/// The paper of a recipe label: fill, texture, a coffee-ring stain, paper
/// grain and a printed inner frame. Pure paint; every choice arrives in the
/// [RecipeLabel] from Rust.
///
/// No [stain] leaves the ring out, for swatches that compare textures. A
/// [boldStain] is a mug ring sized to the paper, drawn by a shader
/// ([CoffeeRing]), for a label as large as the brew page; otherwise it is a
/// faint fixed-size arc in the corner.
class LabelPaperPainter extends CustomPainter {
  const LabelPaperPainter({
    required this.colors,
    required this.texture,
    required this.stain,
    this.boldStain = false,
  });

  final LabelColors colors;
  final LabelTexture texture;
  final StainCorner? stain;
  final bool boldStain;

  @override
  void paint(Canvas canvas, Size size) {
    canvas
      ..drawRect(Offset.zero & size, Paint()..color = colors.paper)
      ..save()
      ..clipRect(Offset.zero & size);
    _paintTexture(canvas, size);
    if (stain case final corner?) {
      if (boldStain) {
        _paintBoldStain(canvas, size, corner);
      } else {
        _paintStain(canvas, size, corner);
      }
    }
    _paintGrain(canvas, size);
    canvas
      ..restore()
      ..drawRRect(
        RRect.fromRectAndRadius(
          (Offset.zero & size).deflate(7),
          const Radius.circular(3),
        ),
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1
          ..color = colors.ink.withValues(alpha: 0.28),
      );
  }

  void _paintTexture(Canvas canvas, Size size) {
    final stroke = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;
    Color ink(double alpha) => colors.stain.withValues(alpha: alpha);

    switch (texture) {
      case LabelTexture.kraft:
        final random = math.Random(7);
        stroke.strokeCap = StrokeCap.round;
        final fibres = (size.width * size.height / 160).round();
        for (var i = 0; i < fibres; i++) {
          final start = Offset(
            random.nextDouble() * size.width,
            random.nextDouble() * size.height,
          );
          final length = 6 + random.nextDouble() * 22;
          final slope = (random.nextDouble() - 0.5) * 0.12;
          stroke.color = ink(0.06 + random.nextDouble() * 0.1);
          canvas.drawLine(
            start,
            start + Offset(length, length * slope),
            stroke,
          );
        }
      case LabelTexture.linen:
        stroke.color = ink(0.1);
        for (var y = 0.5; y < size.height; y += 3) {
          canvas.drawLine(Offset(0, y), Offset(size.width, y), stroke);
        }
        stroke.color = ink(0.08);
        for (var x = 0.5; x < size.width; x += 4) {
          canvas.drawLine(Offset(x, 0), Offset(x, size.height), stroke);
        }
      case LabelTexture.halftone:
        // Dots fading out from the top-right corner.
        final corner = Offset(size.width, 0);
        final reach = size.longestSide * 0.7;
        final dot = Paint();
        for (var y = 3.5; y < size.height; y += 7) {
          for (var x = 3.5; x < size.width; x += 7) {
            final fade = 1 - (Offset(x, y) - corner).distance / reach;
            if (fade <= 0) continue;
            dot.color = ink(0.24 * fade);
            canvas.drawCircle(Offset(x, y), 1.3, dot);
          }
        }
      case LabelTexture.pinstripe:
        stroke.color = ink(0.11);
        for (var x = -size.height; x < size.width; x += 9) {
          canvas.drawLine(
            Offset(x, size.height),
            Offset(x + size.height, 0),
            stroke,
          );
        }
      case LabelTexture.contour:
        // Two sets of origin-map contour lines from opposite corners.
        stroke.color = ink(0.16);
        final low = Offset(size.width * 0.08, size.height * 1.2);
        for (var r = 12.0; r < size.longestSide * 1.2; r += 12) {
          canvas.drawOval(
            Rect.fromCenter(center: low, width: r * 2.8, height: r * 1.8),
            stroke,
          );
        }
        stroke.color = ink(0.1);
        final high = Offset(size.width * 1.05, -size.height * 0.1);
        for (var r = 14.0; r < size.longestSide; r += 14) {
          canvas.drawOval(
            Rect.fromCenter(center: high, width: r * 1.6, height: r * 2.4),
            stroke,
          );
        }
      case LabelTexture.burlap:
        stroke
          ..strokeWidth = 2
          ..color = ink(0.085);
        for (var y = 1.0; y < size.height; y += 5) {
          canvas.drawLine(Offset(0, y), Offset(size.width, y), stroke);
        }
        for (var x = 1.0; x < size.width; x += 5) {
          canvas.drawLine(Offset(x, 0), Offset(x, size.height), stroke);
        }
      // Bare paper: nothing to paint over the fill.
      case LabelTexture.plain:
    }
  }

  /// A coffee-ring arc, half off the edge, open on one side.
  void _paintStain(Canvas canvas, Size size, StainCorner corner) {
    final (center, start) = switch (corner) {
      StainCorner.bottomRight => (
        Offset(size.width - 27, size.height - 13),
        0.35,
      ),
      StainCorner.topLeft => (const Offset(19, 5), 2.8),
    };
    final ring = Rect.fromCircle(center: center, radius: 40);
    const sweep = math.pi * 1.3;
    canvas
      ..drawArc(
        ring,
        start,
        sweep,
        false,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 6
          ..color = colors.stain.withValues(alpha: 0.06),
      )
      ..drawArc(
        ring,
        start,
        sweep,
        false,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 2
          ..strokeCap = StrokeCap.round
          ..color = colors.stain.withValues(alpha: 0.16),
      );
  }

  /// A mug ring sized to the paper, half off the edge at [corner]; the
  /// shader draws it (see [CoffeeRing]).
  void _paintBoldStain(Canvas canvas, Size size, StainCorner corner) {
    final radius = (size.shortestSide * 0.38).clamp(40.0, 180.0);
    final (center, seed) = switch (corner) {
      StainCorner.bottomRight => (
        Offset(size.width - radius * 0.55, size.height - radius * 0.3),
        3.0,
      ),
      StainCorner.topLeft => (Offset(radius * 0.45, radius * 0.15), 17.0),
    };
    CoffeeRing.paint(
      canvas,
      center: center,
      radius: radius,
      color: colors.stain.withValues(alpha: 0.5),
      seed: seed,
    );
  }

  void _paintGrain(Canvas canvas, Size size) {
    final random = math.Random(3);
    final specks = List.generate(
      (size.width * size.height / 14).round(),
      (_) => Offset(
        random.nextDouble() * size.width,
        random.nextDouble() * size.height,
      ),
    );
    canvas.drawPoints(
      PointMode.points,
      specks,
      Paint()
        ..strokeWidth = 1
        ..color = colors.ink.withValues(alpha: 0.05),
    );
  }

  @override
  bool shouldRepaint(LabelPaperPainter oldDelegate) =>
      oldDelegate.colors != colors ||
      oldDelegate.texture != texture ||
      oldDelegate.stain != stain ||
      oldDelegate.boldStain != boldStain;
}
