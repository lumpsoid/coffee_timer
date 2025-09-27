import 'package:coffee_timer/core/recipe/recipe.dart';
import 'package:coffee_timer/recipe_list/recipe_list.dart';
import 'package:flutter/material.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  RecipeHub.instance.actor;
  await RecipeHub.instance.storage.initialize();
  RecipeHub.instance.bus.stream.listen((event) {
    print('RecipeHub: event $event');
  });
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Recipe App',
      theme: ThemeData(primarySwatch: Colors.brown),
      home: const RecipeListPage(),
    );
  }
}
