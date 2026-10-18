import 'dart:async';

import 'package:coffee_timer/effects/effect_plugins.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:flutter/foundation.dart';

/// Carries out each [PlatformEffect] Rust sends, one plugin call per
/// effect, in arrival order. Keeps nothing.
class PlatformEffectPerformer {
  PlatformEffectPerformer(this._plugins);

  final EffectPlugins _plugins;
  Future<void> _last = Future.value();

  /// Performs every effect [effects] delivers; production passes
  /// `PlatformEffect.rustSignalStream`'s messages.
  void listen(Stream<PlatformEffect> effects) {
    effects.listen((effect) => unawaited(perform(effect)));
  }

  /// Queues [effect] behind the ones before it. A failed call is logged and
  /// does not hold up the next.
  Future<void> perform(PlatformEffect effect) {
    return _last = _last
        .then((_) => _call(effect))
        .catchError((Object error) => debugPrint('PlatformEffect: $error'));
  }

  Future<void> _call(PlatformEffect effect) => switch (effect) {
    PlatformEffectPlayStepSound() => _plugins.playStepSound(),
    PlatformEffectVibrate(:final ms) => _plugins.vibrate(ms),
    PlatformEffectVibratePattern(:final patternMs) => _plugins.vibratePattern(
      patternMs,
    ),
    PlatformEffectKeepScreenOn(:final on) => _plugins.keepScreenOn(on: on),
    PlatformEffectShowBrewNotification(:final title, :final next) =>
      _plugins.showBrewNotification(title: title, next: next),
    PlatformEffectHideBrewNotification() => _plugins.hideBrewNotification(),
    PlatformEffectCopyToClipboard(:final text) => _plugins.copyToClipboard(
      text,
    ),
    _ => Future.value(),
  };
}
