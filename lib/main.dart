import 'dart:async';

import 'package:coffee_timer/core/core.dart';
import 'package:coffee_timer/core/di/hub.dart';
import 'package:coffee_timer/recipe_list/recipe_list.dart';
import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:sqlite_async/sqlite_async.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  final dir = await getApplicationDocumentsDirectory();
  final dbPath = p.join(dir.path, 'recipes.db');

  final db = SqliteDatabase(path: dbPath);

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

  // Initialize settings
  final settingsBus = SettingsEventBus.instance..init();

  SettingsActor(emit: settingsBus.fire);
  final settingsStorage = SettingsStorageSqlite(db: db, emit: settingsBus.fire);
  await settingsStorage.initialize();

  settingsBus.stream.listen(settingsStorage.handleEvent);
  settingsBus.stream.listen(ThemeManager.instance.handleEvent);

  settingsBus.fire(const SettingsLoad());

  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: ThemeManager.instance,
      builder: (context, child) {
        return MaterialApp(
          title: 'Recipe App',
          themeMode: ThemeManager.instance.themeMode,
          theme: ThemeManager.instance.light,
          darkTheme: ThemeManager.instance.dark,
          home: const RecipeListPage(),
        );
      },
    );
  }
}
