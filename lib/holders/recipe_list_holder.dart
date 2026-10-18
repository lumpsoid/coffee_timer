import 'dart:async';

import 'package:coffee_timer/holders/undo_offer.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:flutter/foundation.dart';
import 'package:rinf/rinf.dart';

/// The recipe list as Rust last sent it: a snapshot folded with the diffs
/// after it. Lives for the app's lifetime.
class RecipeListHolder extends ChangeNotifier {
  StreamSubscription<RustSignalPack<RecipeListUpdate>>? _subscription;
  final _failures = StreamController<String>.broadcast(sync: true);

  /// One message per failed delete, for a snackbar each; repeats included.
  /// Not list state: the list is unchanged.
  Stream<String> get failures => _failures.stream;

  final _notices = StreamController<String>.broadcast(sync: true);

  /// One message per finished export or import (`Copied 2 recipes`), for
  /// a snackbar each. Not list state: an import's rows arrive as upserts.
  Stream<String> get notices => _notices.stream;

  final _undoOffers = StreamController<UndoOffer>.broadcast(sync: true);

  /// One offer per removal Rust says can be undone, for a snackbar with an
  /// Undo each. Not list state: the row is already gone.
  Stream<UndoOffer> get undoOffers => _undoOffers.stream;

  /// Null until the first snapshot arrives.
  List<RecipeSummary>? get recipes => _recipes;
  List<RecipeSummary>? _recipes;

  /// The listed recipe with [id], if the list holds it.
  RecipeSummary? byId(String id) {
    for (final recipe in _recipes ?? const <RecipeSummary>[]) {
      if (recipe.id == id) return recipe;
    }
    return null;
  }

  /// Subscribes to Rust's updates, then asks for the current snapshot.
  void connect() {
    _subscription = RecipeListUpdate.rustSignalStream.listen(
      (pack) => apply(pack.message),
    );
    const RecipeListReady().sendSignalToRust();
  }

  /// Folds one update in: a snapshot replaces the list, an upsert drops the
  /// row with its id, if any, and inserts it at its index (in place for an
  /// edit, moved for a pin or unpin), a removal drops the
  /// row and passes on its undo offer, if any, on [undoOffers]. A failure
  /// leaves the list alone and goes out on [failures], and a notice on
  /// [notices].
  void apply(RecipeListUpdate update) {
    if (update is RecipeListUpdateFailed) {
      _failures.add(update.message);
      return;
    }
    if (update is RecipeListUpdateNotice) {
      _notices.add(update.message);
      return;
    }
    final current = _recipes ?? const <RecipeSummary>[];
    _recipes = switch (update) {
      RecipeListUpdateSnapshot(:final recipes) => List.of(recipes),
      RecipeListUpdateUpserted(:final recipe, :final index) => _upserted(
        current,
        recipe,
        index,
      ),
      RecipeListUpdateRemoved(:final id) => [
        for (final r in current)
          if (r.id != id) r,
      ],
      _ => current,
    };
    notifyListeners();
    if (update case RecipeListUpdateRemoved(
      :final id,
      undoNotice: final String notice,
    )) {
      _undoOffers.add(UndoOffer(recipeId: id, notice: notice));
    }
  }

  static List<RecipeSummary> _upserted(
    List<RecipeSummary> current,
    RecipeSummary recipe,
    int index,
  ) {
    final rest = [
      for (final r in current)
        if (r.id != recipe.id) r,
    ];
    return rest..insert(index.clamp(0, rest.length), recipe);
  }

  @override
  void dispose() {
    unawaited(_subscription?.cancel());
    unawaited(_failures.close());
    unawaited(_notices.close());
    unawaited(_undoOffers.close());
    super.dispose();
  }
}
