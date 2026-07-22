import 'package:meta/meta.dart';

/// A non-empty, trimmed recipe name, valid by construction.
///
/// Build one via [tryParse], which trims and rejects a blank result. The
/// [untitled] fallback covers legacy rows that stored an empty name, so a
/// [RecipeName] above the storage boundary is always something displayable.
@immutable
class RecipeName {
  const RecipeName._(this.value);

  /// Fallback for legacy rows with a blank name.
  static const untitled = RecipeName._('Untitled');

  final String value;

  /// Returns a trimmed [RecipeName] for [raw], or null if it is blank.
  static RecipeName? tryParse(String raw) {
    final trimmed = raw.trim();
    return trimmed.isEmpty ? null : RecipeName._(trimmed);
  }

  @override
  bool operator ==(Object other) => other is RecipeName && other.value == value;

  @override
  int get hashCode => value.hashCode;

  @override
  String toString() => 'RecipeName($value)';
}
