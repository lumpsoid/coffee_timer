import 'dart:ui' as ui;

import 'package:flutter/painting.dart';

/// The brew page's coffee-ring stain, drawn by `shaders/coffee_ring.frag`.
///
/// The shader is loaded once, at startup ([load]); before that, and in
/// tests that don't load it, [paint] draws nothing.
abstract final class CoffeeRing {
  static const _asset = 'shaders/coffee_ring.frag';

  static ui.FragmentProgram? _program;

  static Future<void> load() async {
    _program = await ui.FragmentProgram.fromAsset(_asset);
  }

  /// A ring of [radius] around [center] in [color]; [color]'s alpha is the
  /// darkest the rim gets. [seed] picks the ring's wobble and patches.
  static void paint(
    Canvas canvas, {
    required Offset center,
    required double radius,
    required Color color,
    required double seed,
  }) {
    final program = _program;
    if (program == null) return;
    final shader = program.fragmentShader()
      ..setFloat(0, center.dx)
      ..setFloat(1, center.dy)
      ..setFloat(2, radius)
      ..setFloat(3, color.r)
      ..setFloat(4, color.g)
      ..setFloat(5, color.b)
      ..setFloat(6, color.a)
      ..setFloat(7, seed);
    // The ring and its offset twin stay within 1.2 radii of the centre.
    canvas.drawRect(
      Rect.fromCircle(center: center, radius: radius * 1.2),
      Paint()..shader = shader,
    );
  }
}
