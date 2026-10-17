import 'package:flutter/material.dart';

/// [time] with each character in its own fixed-width cell, so the countdown
/// does not jitter as digits change.
class FixedDigit extends StatelessWidget {
  const FixedDigit({
    required this.time,
    required this.style,
    super.key,
  });

  final String time;
  final TextStyle? style;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: time.split('').map((char) {
        return Container(
          width: 24,
          height: 36,
          alignment: Alignment.center,
          margin: const EdgeInsets.symmetric(horizontal: 1),
          child: Text(
            char,
            style: style,
          ),
        );
      }).toList(),
    );
  }
}
