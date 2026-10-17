import 'package:coffee_timer/holders/recipe_list_holder.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:flutter_test/flutter_test.dart';

import '../fixtures.dart';

void main() {
  late RecipeListHolder holder;
  late int notifications;

  List<String> ids() => [for (final r in holder.recipes!) r.id];

  setUp(() {
    holder = RecipeListHolder();
    notifications = 0;
    holder.addListener(() => notifications++);
  });

  test('holds nothing until the first snapshot', () {
    expect(holder.recipes, isNull);
  });

  test('a snapshot replaces the list', () {
    holder
      ..apply(RecipeListUpdateSnapshot(recipes: [summary('a')]))
      ..apply(
        RecipeListUpdateSnapshot(recipes: [summary('b'), summary('c')]),
      );

    expect(ids(), ['b', 'c']);
    expect(notifications, 2);
  });

  test('an upsert replaces the row with its id, in place', () {
    holder
      ..apply(
        RecipeListUpdateSnapshot(recipes: [summary('a'), summary('b')]),
      )
      ..apply(RecipeListUpdateUpserted(recipe: summary('a', name: 'Chemex')));

    expect(ids(), ['a', 'b']);
    expect(holder.byId('a')?.name, 'Chemex');
    expect(holder.byId('b')?.name, 'V60');
  });

  test('an upsert of a new id appends it', () {
    holder
      ..apply(RecipeListUpdateSnapshot(recipes: [summary('a')]))
      ..apply(RecipeListUpdateUpserted(recipe: summary('b')));

    expect(ids(), ['a', 'b']);
  });

  test('a removal drops the row with its id', () {
    holder
      ..apply(
        RecipeListUpdateSnapshot(recipes: [summary('a'), summary('b')]),
      )
      ..apply(const RecipeListUpdateRemoved(id: 'a'));

    expect(ids(), ['b']);
    expect(holder.byId('a'), isNull);
  });

  test('snapshot, upsert (replace and append), then remove', () {
    holder
      ..apply(
        RecipeListUpdateSnapshot(recipes: [summary('a'), summary('b')]),
      )
      ..apply(RecipeListUpdateUpserted(recipe: summary('b', name: 'Aero')))
      ..apply(RecipeListUpdateUpserted(recipe: summary('c')))
      ..apply(const RecipeListUpdateRemoved(id: 'a'));

    expect(ids(), ['b', 'c']);
    expect(holder.byId('b')?.name, 'Aero');
    expect(notifications, 4);
  });

  test('a failure leaves the list alone and is announced each time', () async {
    final failures = <String>[];
    holder
      ..apply(RecipeListUpdateSnapshot(recipes: [summary('a')]))
      ..failures.listen(failures.add)
      ..apply(const RecipeListUpdateFailed(message: 'Failed to delete'))
      ..apply(const RecipeListUpdateFailed(message: 'Failed to delete'));

    expect(ids(), ['a']);
    expect(failures, ['Failed to delete', 'Failed to delete']);
    expect(notifications, 1);
  });
}
