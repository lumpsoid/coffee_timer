import 'package:flutter/material.dart';

class CircularTimerWidget extends StatelessWidget {
  const CircularTimerWidget({
    required this.controller,
    required this.child,
    super.key,
    this.constraints,
    this.strokeWidth = 10,
    this.backgroundColor = const Color(0xFF424242),
    this.progressColor = Colors.greenAccent,
  });

  final ValueNotifier<double> controller;
  final Widget child;
  final BoxConstraints? constraints;
  final double strokeWidth;
  final Color backgroundColor;
  final Color progressColor;

  @override
  Widget build(BuildContext context) {
    return Stack(
      alignment: Alignment.center,
      children: [
        ValueListenableBuilder(
          valueListenable: controller,
          builder: (context, progress, _) {
            print('CircularProgressIndicator build');
            return CircularProgressIndicator(
              constraints: constraints,
              value: progress,
              strokeWidth: strokeWidth,
              backgroundColor: backgroundColor,
              valueColor: AlwaysStoppedAnimation<Color>(progressColor),
            );
          },
        ),
        child,
      ],
    );
  }
}
