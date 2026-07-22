import 'package:meta/meta.dart';

/// A single step's duration in seconds, always within `0..maxStepSeconds`.
///
/// Two entry points, one home for the bound:
/// - [tryParse] is strict — it rejects out-of-range input (user input, later).
/// - [StepDuration.clamped] is lenient — it clamps into range (storage rows).
@immutable
class StepDuration {
  const StepDuration._(this.seconds);

  /// Returns a [StepDuration] with [raw] clamped into `0..maxStepSeconds`.
  factory StepDuration.clamped(int raw) {
    if (raw < 0) return zero;
    if (raw > maxStepSeconds) return const StepDuration._(maxStepSeconds);
    return StepDuration._(raw);
  }

  /// Inclusive upper bound for a single step. The one home for this rule.
  static const maxStepSeconds = 5999;

  static const zero = StepDuration._(0);

  /// Always within `0..maxStepSeconds`.
  final int seconds;

  /// Returns a [StepDuration] for [raw], or null if it is out of range.
  static StepDuration? tryParse(int raw) =>
      (raw < 0 || raw > maxStepSeconds) ? null : StepDuration._(raw);

  /// `MM:SS`, each part zero-padded to two digits.
  String get mmss =>
      '${(seconds ~/ 60).toString().padLeft(2, '0')}'
      ':${(seconds % 60).toString().padLeft(2, '0')}';

  @override
  bool operator ==(Object other) =>
      other is StepDuration && other.seconds == seconds;

  @override
  int get hashCode => seconds.hashCode;

  @override
  String toString() => 'StepDuration($seconds)';
}
