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
/// No [stain] leaves the ring out, for swatches that compare textures;
/// otherwise a mug ring sized to the paper, drawn by a shader ([CoffeeRing]),
/// sits half off the edge at that corner.
class LabelPaperPainter extends CustomPainter {
  const LabelPaperPainter({
    required this.colors,
    required this.texture,
    required this.stain,
  });

  final LabelColors colors;
  final LabelTexture texture;
  final StainCorner? stain;

  @override
  void paint(Canvas canvas, Size size) {
    canvas
      ..drawRect(Offset.zero & size, Paint()..color = colors.paper)
      ..save()
      ..clipRect(Offset.zero & size);
    _paintTexture(canvas, size);
    if (stain case final corner?) {
      _paintStain(canvas, size, corner);
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
      case LabelTexture.graph:
        // Fine grid of 6 px cells, with heavier lines every 5 cells. Each
        // family is one path, so the crossings don't take the alpha twice.
        final fine = Path();
        for (var x = 0.5; x < size.width; x += 6) {
          fine
            ..moveTo(x, 0)
            ..lineTo(x, size.height);
        }
        for (var y = 0.5; y < size.height; y += 6) {
          fine
            ..moveTo(0, y)
            ..lineTo(size.width, y);
        }
        stroke.color = ink(0.07);
        canvas.drawPath(fine, stroke);
        final major = Path();
        for (var x = 0.5; x < size.width; x += 30) {
          major
            ..moveTo(x, 0)
            ..lineTo(x, size.height);
        }
        for (var y = 0.5; y < size.height; y += 30) {
          major
            ..moveTo(0, y)
            ..lineTo(size.width, y);
        }
        stroke.color = ink(0.15);
        canvas.drawPath(major, stroke);
      case LabelTexture.ledger:
        // Ruled lines every 10 px, with an accent-coloured margin.
        final rules = Path();
        for (var y = 10.5; y < size.height; y += 10) {
          rules
            ..moveTo(0, y)
            ..lineTo(size.width, y);
        }
        stroke.color = ink(0.12);
        canvas.drawPath(rules, stroke);
        final margin = Path()
          ..moveTo(16.5, 0)
          ..lineTo(16.5, size.height);
        stroke.color = colors.accent.withValues(alpha: 0.25);
        canvas.drawPath(margin, stroke);
      case LabelTexture.dotGrid:
        // Dots on an 8 px grid, all at one alpha.
        final dotPaint = Paint()..color = ink(0.2);
        for (var y = 4.0; y < size.height; y += 8) {
          for (var x = 4.0; x < size.width; x += 8) {
            canvas.drawCircle(Offset(x, y), 0.9, dotPaint);
          }
        }
      case LabelTexture.crosshatch:
        // Two families of lines at 45 degrees, through the centre, in one
        // path as on the design page.
        final halfDiagonal = Offset(size.width, size.height).distance / 2;
        final dx = math.cos(math.pi / 4);
        final hatch = Path();
        for (final sign in [1.0, -1.0]) {
          final dy = sign * math.sin(math.pi / 4);
          for (var off = -halfDiagonal; off <= halfDiagonal; off += 8) {
            final cx = size.width / 2 - dy * off;
            final cy = size.height / 2 + dx * off;
            hatch
              ..moveTo(cx - dx * halfDiagonal, cy - dy * halfDiagonal)
              ..lineTo(cx + dx * halfDiagonal, cy + dy * halfDiagonal);
          }
        }
        stroke.color = ink(0.09);
        canvas.drawPath(hatch, stroke);
      case LabelTexture.herringbone:
        // Staggered 45-degree planks in columns that alternate slope, in one
        // path as on the design page.
        const run = 20 * math.sqrt1_2;
        final planks = Path();
        for (var col = 0; col * run < size.width; col++) {
          final x = col * run;
          final rising = col.isEven;
          final offset = rising ? 0.0 : 2.5;
          for (var y = -run + offset; y < size.height; y += 5) {
            planks
              ..moveTo(x, rising ? y + run : y)
              ..lineTo(x + run, rising ? y : y + run);
          }
        }
        stroke.color = ink(0.13);
        canvas.drawPath(planks, stroke);
      case LabelTexture.ripple:
        // Wavy lines 6 px apart, each a little further out of phase than the
        // one above, in one path.
        final waves = Path();
        var row = 0;
        for (var y0 = 3.0; y0 < size.height + 2; y0 += 6, row++) {
          final phase = row * 0.12;
          double wave(double x) =>
              y0 + 2 * math.sin(2 * math.pi * x / 40 + phase);
          waves.moveTo(0, wave(0));
          for (var x = 2.0; x <= size.width; x += 2) {
            waves.lineTo(x, wave(x));
          }
        }
        stroke.color = ink(0.12);
        canvas.drawPath(waves, stroke);
      case LabelTexture.woodgrain:
        // Grain lines 4 px apart, wobbling and bent around two seeded knots,
        // each ringed by three ovals.
        final random = math.Random(5);
        final knots = [
          for (var i = 0; i < 2; i++)
            Offset(
              random.nextDouble() * size.width,
              random.nextDouble() * size.height,
            ),
        ];
        double bend(double x, double y) {
          var dy = 0.0;
          for (final knot in knots) {
            final dx = (x - knot.dx) / 12;
            final dv = (y - knot.dy) / (12 * 0.6);
            dy += 4 * math.exp(-(dx * dx + dv * dv)) * (y >= knot.dy ? 1 : -1);
          }
          return dy;
        }

        const tau = math.pi * 2;
        final grain = Path();
        for (var y0 = 2.0; y0 < size.height; y0 += 4) {
          // Three sines with seeded phases make each line's wobble.
          final ph = [
            random.nextDouble() * tau,
            random.nextDouble() * tau,
            random.nextDouble() * tau,
          ];
          double at(double x) {
            final wob =
                3 *
                (0.6 * math.sin(tau * x / 120 + ph[0]) +
                    0.3 * math.sin(tau * x / (120 * 0.37) + ph[1]) +
                    0.2 * math.sin(tau * x / (120 * 2.3) + ph[2]));
            final y = y0 + wob;
            return y + bend(x, y);
          }

          grain.moveTo(0, at(0));
          for (var x = 2.0; x <= size.width; x += 2) {
            grain.lineTo(x, at(x));
          }
        }
        stroke.color = ink(0.1);
        canvas.drawPath(grain, stroke);
        for (final knot in knots) {
          for (final f in [0.4, 0.7, 1.0]) {
            canvas.drawOval(
              Rect.fromCenter(
                center: knot,
                width: 2 * 12 * f,
                height: 2 * 12 * f * 0.6,
              ),
              stroke,
            );
          }
        }
      case LabelTexture.laid:
        // Laid lines 2.5 px apart, with chain lines 18 px apart in a stronger
        // alpha, each family in one path.
        final laidLines = Path();
        for (var y = 0.5; y < size.height; y += 2.5) {
          laidLines
            ..moveTo(0, y)
            ..lineTo(size.width, y);
        }
        stroke.color = ink(0.07);
        canvas.drawPath(laidLines, stroke);
        final chain = Path();
        for (var x = 0.5; x < size.width; x += 18) {
          chain
            ..moveTo(x, 0)
            ..lineTo(x, size.height);
        }
        stroke.color = ink(0.12);
        canvas.drawPath(chain, stroke);
      // Bare paper: nothing to paint over the fill.
      case LabelTexture.plain:
    }
  }

  /// A mug ring sized to the paper, half off the edge at [corner]; the
  /// shader draws it (see [CoffeeRing]).
  void _paintStain(Canvas canvas, Size size, StainCorner corner) {
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
      oldDelegate.stain != stain;
}
