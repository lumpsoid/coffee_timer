import 'package:flutter/material.dart';

class StepListTile extends StatelessWidget {
  const StepListTile({
    required this.index,
    required this.stepName,
    required this.durationSeconds,
    required this.isTicking,
    required this.currentStepIndex,
    required this.onLongPress,
    super.key,
  });

  final int index;
  final String stepName;
  final int durationSeconds;
  final bool isTicking;
  final int currentStepIndex;
  final VoidCallback onLongPress;

  Widget _buildLeading(Widget child) {
    return SizedBox(
      width: 24,
      height: 24,
      child: Align(child: child),
    );
  }

  @override
  Widget build(BuildContext context) {
    Widget newLeading;
    if (isTicking) {
      if (index < currentStepIndex && currentStepIndex != -1) {
        // Already played steps
        newLeading = const Icon(Icons.check_circle, key: ValueKey('check'));
      } else if (index == currentStepIndex) {
        // Current
        newLeading = const Icon(Icons.play_arrow, key: ValueKey('play'));
      } else {
        // Future steps
        newLeading = _buildLeading(
          Text('${index + 1}', key: ValueKey('future_${index + 1}')),
        );
      }
    } else {
      // Not started
      newLeading = _buildLeading(
        Text('${index + 1}', key: ValueKey('not_started_${index + 1}')),
      );
    }

    return ListTile(
      leading: AnimatedSwitcher(
        duration: const Duration(milliseconds: 200),
        transitionBuilder: (Widget child, Animation<double> animation) {
          return FadeTransition(
            opacity: animation,
            child: child,
          );
        },
        child: newLeading,
      ),
      title: Text(stepName),
      trailing: Text('${durationSeconds}s'),
      onLongPress: onLongPress,
    );
  }
}
