import 'package:coffee_timer/core/recipe/recipe.dart';
import 'package:coffee_timer/core/settings/settings.dart';
import 'package:coffee_timer/core/theme/theme.dart';
import 'package:coffee_timer/recipe_list/recipe_list.dart';
import 'package:flutter/material.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  RecipeHub.instance.actor;
  await RecipeHub.instance.storage.initialize();
  RecipeHub.instance.bus.stream.listen((event) {
    print('RecipeHub: event $event');
  });
  // Initialize settings storage
  SettingsHub.instance.actor;
  await SettingsHub.instance.storage.initialize();

  // Initialize theme manager
  ThemeManager.instance.initialize();
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
