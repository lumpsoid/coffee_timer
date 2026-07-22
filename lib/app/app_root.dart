import 'dart:async';

import 'package:coffee_timer/core/core.dart';
import 'package:coffee_timer/core/di/hub.dart';
import 'package:coffee_timer/recipe_list/recipe_list.dart';
import 'package:flutter/material.dart';
import 'package:sqlite_async/sqlite_async.dart';

/// The single construction root: hosts [MaterialApp] and wires the app's
/// modules once in [State.initState].
///
/// Phase 0 keeps the existing event-bus wiring verbatim so behaviour is
/// unchanged; later phases replace it with modules registered into `Cells`
/// (see `docs/pivot-spec.md`).
class AppRoot extends StatefulWidget {
  const AppRoot({required this.dbPath, super.key});

  final String dbPath;

  @override
  State<AppRoot> createState() => _AppRootState();
}

class _AppRootState extends State<AppRoot> {
  @override
  void initState() {
    super.initState();
    _wire();
  }

  void _wire() {
    final db = SqliteDatabase(path: widget.dbPath);

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

    final settingsBus = SettingsEventBus.instance..init();

    SettingsActor(emit: settingsBus.fire);
    final settingsStorage = SettingsStorageSqlite(
      db: db,
      emit: settingsBus.fire,
    );

    settingsBus.stream.listen(settingsStorage.handleEvent);
    settingsBus.stream.listen(ThemeManager.instance.handleEvent);

    unawaited(
      settingsStorage.initialize().then(
        (_) => settingsBus.fire(const SettingsLoad()),
      ),
    );
  }

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
