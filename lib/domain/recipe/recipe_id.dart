import 'package:meta/meta.dart';

/// A non-empty recipe identifier, valid by construction.
///
/// There is no public constructor: build one via [tryParse], the only entry
/// point, which rejects an empty string. Once you hold a [RecipeId] it is a
/// real id — never re-check it. New ids are minted from `uuid` at the boundary.
@immutable
class RecipeId {
  const RecipeId._(this.value);

  final String value;

  /// Returns a [RecipeId] for [raw], or null if it is empty.
  static RecipeId? tryParse(String raw) => raw.isEmpty ? null : RecipeId._(raw);

  @override
  bool operator ==(Object other) => other is RecipeId && other.value == value;

  @override
  int get hashCode => value.hashCode;

  @override
  String toString() => 'RecipeId($value)';
}
