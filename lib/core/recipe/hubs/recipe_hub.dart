import 'package:coffee_timer/core/recipe/recipe.dart';

abstract class RecipeHub {
  static final RecipeHub instance = _RecipeHubImpl();

  RecipeEventBus get bus;
  RecipeActor get actor;
  RecipeStorageSqlite get storage;
}

class _RecipeHubImpl implements RecipeHub {
  _RecipeHubImpl();

  final RecipeEventBus _bus = RecipeEventBus();
  @override
  RecipeEventBus get bus => _bus;

  late final RecipeActor _actor = RecipeActor(bus: _bus);
  @override
  RecipeActor get actor => _actor;

  late final RecipeStorageSqlite _storage = RecipeStorageSqlite(bus: _bus);
  @override
  RecipeStorageSqlite get storage => _storage;
}
