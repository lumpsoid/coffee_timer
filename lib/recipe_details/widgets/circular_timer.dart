import 'package:coffee_timer/recipe_details/widgets/widgets.dart';
import 'package:flutter/material.dart';

class CircularTimer extends StatelessWidget {
  const CircularTimer({
    required this.duration,
    required this.child,
    this.strokeWidth = 10,
    this.backgroundColor = const Color(0xFF424242),
    this.progressColor = Colors.greenAccent,
    this.constraints,
    super.key,
  });

  final ValueNotifier<Duration?> duration;
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
        ValueListenableBuilder(
          valueListenable: duration,
          builder: (context, d, _) {
            return DurationalCircularProgressIndicator(
              duration: d,
              strokeWidth: strokeWidth,
              backgroundColor: backgroundColor,
              progressColor: progressColor,
              constraints: constraints,
            );
          },
        ),
        child,
      ],
    );
  }
}
