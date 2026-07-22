import 'dart:async';

import 'package:coffee_timer/app/cells.dart';
import 'package:coffee_timer/app/recipe_module.dart';
import 'package:coffee_timer/app/settings_module.dart';
import 'package:coffee_timer/core/core.dart';
import 'package:coffee_timer/core/di/hub.dart';
import 'package:coffee_timer/infrastructure/database/app_database.dart';
import 'package:coffee_timer/infrastructure/recipe/sqlite_recipe_repository.dart';
import 'package:coffee_timer/infrastructure/recipe/uuid_recipe_ids.dart';
import 'package:coffee_timer/infrastructure/settings/sqlite_settings_repository.dart';
import 'package:coffee_timer/ui/recipe_list/recipe_list_page.dart';
import 'package:coffee_timer/ui/settings/theme_controller.dart';
import 'package:flutter/material.dart';
import 'package:sqlite_async/sqlite_async.dart';

/// The single construction root: hosts [MaterialApp] and wires the app's
/// modules once, after the database has been opened and migrated.
///
/// Settings and the recipe *list* + *editor* run on the new module/controller
/// stack (registered into `Cells`); the recipe detail/timer page still rides
/// the legacy event bus until Phase 4 strangles it (see `docs/pivot-spec.md`).
class AppRoot extends StatefulWidget {
  const AppRoot({required this.dbPath, super.key});

  final String dbPath;

  @override
  State<AppRoot> createState() => _AppRootState();
}

class _AppRootState extends State<AppRoot> {
  ThemeController? _theme;

  @override
  void initState() {
    super.initState();
    unawaited(_bootstrap());
  }

  /// Opens + migrates the database, then wires every module. The UI below only
  /// builds once this completes, so no repository query can outrun a migration.
  Future<void> _bootstrap() async {
    final db = await AppDatabase(path: widget.dbPath).open();
    _wireRecipe(db);
    _wireSettings(db);
    if (mounted) setState(() {});
  }

  void _wireRecipe(SqliteDatabase db) {
    Hub()
      ..registerBus('recipe')
      ..registerLeaf(
        busName: 'recipe',
        name: 'audioPlayer',
        leaf: AudioPlayerLeaf(),
      )
      ..registerLeaf(
        busName: 'recipe',
        name: 'recipeActor',
        leaf: RecipeActor(),
      )
      ..registerLeaf(
        busName: 'recipe',
        name: 'recipeStorage',
        leaf: RecipeStorageSqlite(db: db),
      )
      ..subscribe(
        busName: 'recipe',
        moduleName: 'observer',
        handler: (event) {
          // debug part
          // ignore: avoid_print
          print('RecipeHub: event $event');
        },
      )
      ..subscribe(
        busName: 'recipe',
        moduleName: 'recipeToVibrate',
        handler: VibrateFlutterLeaf.handle,
      )
      ..subscribe(
        busName: 'recipe',
        moduleName: 'recipeToWakeLock',
        handler: WakelockLeaf.handle,
      )
      ..emitToLeaf('audioPlayer', const AudioInitialize());

    // New stack for the recipe *list* and *editor*: reads/writes straight
    // through the narrow ports, no bus. Only the detail/timer page still goes
    // through the Hub above. Schema is already migrated by `AppDatabase`.
    final repository = SqliteRecipeRepository(db);
    Cells.recipeModule.value = RecipeModule(
      read: repository,
      write: repository,
      ids: const UuidRecipeIds(),
    );
  }

  void _wireSettings(SqliteDatabase db) {
    final repository = SqliteSettingsRepository(db);
    final theme = ThemeController(repository);
    Cells.theme.value = theme;
    Cells.settingsModule.value = SettingsModule(
      repository: repository,
      theme: theme,
    );
    _theme = theme;
    unawaited(theme.load());
  }

  @override
  Widget build(BuildContext context) {
    final theme = _theme;
    if (theme == null) {
      return const MaterialApp(
        debugShowCheckedModeBanner: false,
        home: Scaffold(body: Center(child: CircularProgressIndicator())),
      );
    }
    return ListenableBuilder(
      listenable: theme,
      builder: (context, child) {
        return MaterialApp(
          title: 'Recipe App',
          themeMode: theme.mode,
          theme: theme.light,
          darkTheme: theme.dark,
          home: const RecipeListPage(),
        );
      },
    );
  }
}
