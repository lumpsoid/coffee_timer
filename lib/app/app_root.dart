import 'dart:async';

import 'package:coffee_timer/app/cells.dart';
import 'package:coffee_timer/app/settings_module.dart';
import 'package:coffee_timer/core/core.dart';
import 'package:coffee_timer/core/di/hub.dart';
import 'package:coffee_timer/infrastructure/settings/sqlite_settings_repository.dart';
import 'package:coffee_timer/recipe_list/recipe_list.dart';
import 'package:coffee_timer/ui/settings/theme_controller.dart';
import 'package:flutter/material.dart';
import 'package:sqlite_async/sqlite_async.dart';

/// The single construction root: hosts [MaterialApp] and wires the app's
/// modules once in [State.initState].
///
/// Settings run on the new module/controller stack (registered into `Cells`);
/// recipes still ride the legacy event bus until later phases strangle it
/// (see `docs/pivot-spec.md`).
class AppRoot extends StatefulWidget {
  const AppRoot({required this.dbPath, super.key});

  final String dbPath;

  @override
  State<AppRoot> createState() => _AppRootState();
}

class _AppRootState extends State<AppRoot> {
  late final ThemeController _theme;

  @override
  void initState() {
    super.initState();
    final db = SqliteDatabase(path: widget.dbPath);
    _wireRecipe(db);
    _wireSettings(db);
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
  }

  void _wireSettings(SqliteDatabase db) {
    final repository = SqliteSettingsRepository(db);
    _theme = ThemeController(repository);
    Cells.theme.value = _theme;
    Cells.settingsModule.value = SettingsModule(
      repository: repository,
      theme: _theme,
    );
    unawaited(repository.initialize().then((_) => _theme.load()));
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: _theme,
      builder: (context, child) {
        return MaterialApp(
          title: 'Recipe App',
          themeMode: _theme.mode,
          theme: _theme.light,
          darkTheme: _theme.dark,
          home: const RecipeListPage(),
        );
      },
    );
  }
}
