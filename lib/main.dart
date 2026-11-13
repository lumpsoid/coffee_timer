import 'dart:async';

import 'package:coffee_timer/app.dart';
import 'package:coffee_timer/effects/effect_plugins.dart';
import 'package:coffee_timer/effects/platform_effect_performer.dart';
import 'package:coffee_timer/holders/boot_holder.dart';
import 'package:coffee_timer/holders/brew_holder.dart';
import 'package:coffee_timer/holders/editor_holder.dart';
import 'package:coffee_timer/holders/recipe_list_holder.dart';
import 'package:coffee_timer/holders/settings_holder.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/coffee_ring.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:rinf/rinf.dart';

/// Loads the coffee-ring shader, starts Rust, subscribes every holder (each
/// then sends its ready signal, which Rust queues until it has booted), then
/// boots Rust at the app's documents directory. The GUI shows a spinner
/// until the first recipe-list snapshot and the settings arrive.
Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  LicenseRegistry.addLicense(_fontLicenses);
  // Before any label paints: the paper is snapshotted once, ring included.
  await CoffeeRing.load();
  await initializeRust(assignRustSignal);

  PlatformEffectPerformer(PluginEffects()).listen(
    PlatformEffect.rustSignalStream.map((pack) => pack.message),
  );
  final recipes = RecipeListHolder()..connect();
  final brew = BrewHolder()..connect();
  final editor = EditorHolder()..connect();
  final settings = SettingsHolder()..connect();
  final boot = BootHolder();
  unawaited(boot.connect());

  runApp(
    CoffeeTimerApp(
      boot: boot,
      recipes: recipes,
      brew: brew,
      editor: editor,
      settings: settings,
    ),
  );
}

/// The bundled fonts' OFL texts, for the platform licence page.
Stream<LicenseEntry> _fontLicenses() async* {
  for (final family in ['Oswald', 'Fraunces']) {
    yield LicenseEntryWithLineBreaks(
      [family],
      await rootBundle.loadString('assets/fonts/$family-OFL.txt'),
    );
  }
}
