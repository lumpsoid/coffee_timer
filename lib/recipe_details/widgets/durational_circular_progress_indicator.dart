import 'dart:async';

import 'package:flutter/material.dart';

class DurationalCircularProgressIndicator extends StatefulWidget {
  const DurationalCircularProgressIndicator({
    required this.duration,
    required this.strokeWidth,
    required this.backgroundColor,
    required this.progressColor,
    this.constraints,
    super.key,
  });
  final Duration? duration;
  final double strokeWidth;
  final Color backgroundColor;
  final Color progressColor;
  final BoxConstraints? constraints;

  @override
  State<DurationalCircularProgressIndicator> createState() =>
      _DurationalCircularProgressIndicator();
}

class _DurationalCircularProgressIndicator
    extends State<DurationalCircularProgressIndicator>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _initAnimation();
  }

  void _initAnimation() {
    _controller = AnimationController(
      vsync: this,
      duration: widget.duration,
    );
    if (widget.duration != null) {
      unawaited(_controller.repeat());
    }
  }

  @override
  void didUpdateWidget(
    covariant DurationalCircularProgressIndicator oldWidget,
  ) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.duration != widget.duration) {
      _controller.duration = widget.duration;
      _controller.reset();
      if (widget.duration != null) {
        unawaited(_controller.repeat());
      }
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
      builder: (context, child) {
        return CircularProgressIndicator(
          value: _controller.value,
          strokeWidth: widget.strokeWidth,
          backgroundColor: widget.backgroundColor,
          valueColor: AlwaysStoppedAnimation<Color>(widget.progressColor),
          constraints: widget.constraints,
        );
      },
    );
  }
}
