import 'package:meta/meta.dart';

/// A non-empty step identifier, valid by construction.
///
/// Distinct from `RecipeId` so the two can never be swapped by accident. Build
/// one via [tryParse]; new ids are minted from `uuid` at the write boundary.
@immutable
class StepId {
  const StepId._(this.value);

  final String value;

  /// Returns a [StepId] for [raw], or null if it is empty.
  static StepId? tryParse(String raw) => raw.isEmpty ? null : StepId._(raw);

  @override
  bool operator ==(Object other) => other is StepId && other.value == value;

  @override
  int get hashCode => value.hashCode;

  @override
  String toString() => 'StepId($value)';
}
