import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:flutter/material.dart';

/// One line of the brew page's step list. The leading mark follows
/// [BrewStep.status]: a check when done, a play arrow when current, the
/// step's number while pending.
class StepListTile extends StatelessWidget {
  const StepListTile({
    required this.step,
    required this.onLongPress,
    super.key,
  });

  final BrewStep step;
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
    final leading = switch (step.status) {
      BrewStepStatus.done => const Icon(
        Icons.check_circle,
        key: ValueKey('check'),
      ),
      BrewStepStatus.current => const Icon(
        Icons.play_arrow,
        key: ValueKey('play'),
      ),
      BrewStepStatus.pending => _buildLeading(
        Text(step.numberLabel, key: ValueKey('pending_${step.numberLabel}')),
      ),
    };

    return ListTile(
      leading: AnimatedSwitcher(
        duration: const Duration(milliseconds: 200),
        transitionBuilder: (Widget child, Animation<double> animation) {
          return FadeTransition(
            opacity: animation,
            child: child,
          );
        },
        child: leading,
      ),
      title: Text(step.name),
      trailing: Text(step.durationLabel),
      onLongPress: onLongPress,
    );
  }
}
