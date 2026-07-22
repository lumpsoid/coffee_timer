import 'package:coffee_timer/app/app.dart';
import 'package:flutter/material.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  final dir = await getApplicationDocumentsDirectory();
  final dbPath = p.join(dir.path, 'recipes.db');

  runApp(AppRoot(dbPath: dbPath));
}
