import 'package:flutter/foundation.dart';

/// Rust's offer to undo a recipe removal: the snackbar text (`Deleted V60`)
/// and the recipe the Undo brings back.
@immutable
class UndoOffer {
  const UndoOffer({required this.recipeId, required this.notice});

  final String recipeId;
  final String notice;

  @override
  bool operator ==(Object other) =>
      other is UndoOffer &&
      other.recipeId == recipeId &&
      other.notice == notice;

  @override
  int get hashCode => Object.hash(recipeId, notice);
}
