import 'dart:ui' show AppExitResponse;

import 'package:coffee_timer/holders/boot_holder.dart';
import 'package:coffee_timer/holders/brew_holder.dart';
import 'package:coffee_timer/holders/editor_holder.dart';
import 'package:coffee_timer/holders/recipe_list_holder.dart';
import 'package:coffee_timer/holders/settings_holder.dart';
import 'package:coffee_timer/pages/recipe_list_page.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:flutter/material.dart';
import 'package:rinf/rinf.dart';

/// The app: themed from the settings holder, showing the recipe list once
/// Rust has sent it (a spinner until then, or the boot failure with a
/// retry). Shuts Rust down on exit so it can close the database.
class CoffeeTimerApp extends StatefulWidget {
  const CoffeeTimerApp({
    required this.boot,
    required this.recipes,
    required this.brew,
    required this.editor,
    required this.settings,
    super.key,
  });

  final BootHolder boot;
  final RecipeListHolder recipes;
  final BrewHolder brew;
  final EditorHolder editor;
  final SettingsHolder settings;

  @override
  State<CoffeeTimerApp> createState() => _CoffeeTimerAppState();
}

class _CoffeeTimerAppState extends State<CoffeeTimerApp> {
  late final AppLifecycleListener _lifecycle;

  @override
  void initState() {
    super.initState();
    _lifecycle = AppLifecycleListener(
      onExitRequested: () async {
        finalizeRust();
        return AppExitResponse.exit;
      },
    );
  }

  @override
  void dispose() {
    _lifecycle.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: widget.settings,
      builder: (context, _) {
        final settings = widget.settings.state;
        return MaterialApp(
          title: 'Recipe App',
          themeMode: settings == null ? null : _mode(settings.theme),
          theme: settings == null ? null : _light(Color(settings.accent)),
          darkTheme: settings == null ? null : _dark(Color(settings.accent)),
          home: _Home(
            boot: widget.boot,
            recipes: widget.recipes,
            brew: widget.brew,
            editor: widget.editor,
            settings: widget.settings,
          ),
        );
      },
    );
  }

  static ThemeMode _mode(ThemeChoice theme) => switch (theme) {
    ThemeChoice.light => ThemeMode.light,
    ThemeChoice.dark => ThemeMode.dark,
    ThemeChoice.system => ThemeMode.system,
  };

  static ThemeData _light(Color seed) => ThemeData.light().copyWith(
    colorScheme: ColorScheme.fromSeed(seedColor: seed),
  );

  static ThemeData _dark(Color seed) => ThemeData.dark().copyWith(
    colorScheme: ColorScheme.fromSeed(
      seedColor: seed,
      brightness: Brightness.dark,
    ),
  );
}

/// The boot failure, a spinner, or the recipe list once both it and the
/// settings have arrived.
class _Home extends StatelessWidget {
  const _Home({
    required this.boot,
    required this.recipes,
    required this.brew,
    required this.editor,
    required this.settings,
  });

  final BootHolder boot;
  final RecipeListHolder recipes;
  final BrewHolder brew;
  final EditorHolder editor;
  final SettingsHolder settings;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: Listenable.merge([boot, recipes, settings]),
      builder: (context, _) {
        final failure = boot.failure;
        if (failure != null) {
          return _BootFailure(message: failure, onRetry: boot.boot);
        }
        if (recipes.recipes == null || settings.state == null) {
          return const Scaffold(
            body: Center(child: CircularProgressIndicator()),
          );
        }
        return RecipeListPage(
          recipes: recipes,
          brew: brew,
          editor: editor,
          settings: settings,
        );
      },
    );
  }
}

class _BootFailure extends StatelessWidget {
  const _BootFailure({required this.message, required this.onRetry});

  final String message;
  final Future<void> Function() onRetry;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(message, textAlign: TextAlign.center),
              const SizedBox(height: 16),
              ElevatedButton(onPressed: onRetry, child: const Text('Retry')),
            ],
          ),
        ),
      ),
    );
  }
}
