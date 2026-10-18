import 'package:audioplayers/audioplayers.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:vibration/vibration.dart';
import 'package:wakelock_plus/wakelock_plus.dart';

/// The platform plugins a `PlatformEffect` reaches: the border a test
/// replaces with a fake.
abstract interface class EffectPlugins {
  Future<void> playStepSound();

  Future<void> vibrate(int ms);

  /// Alternating pause and buzz lengths in ms, starting with a pause.
  Future<void> vibratePattern(List<int> patternMs);

  Future<void> keepScreenOn({required bool on});

  Future<void> showBrewNotification({required String title, String? next});

  Future<void> hideBrewNotification();

  Future<void> copyToClipboard(String text);
}

/// [EffectPlugins] on `audioplayers`, `vibration` and `wakelock_plus`,
/// Flutter's `Clipboard`, and the app's own Android brew-notification
/// channel (`MainActivity.kt`), which other platforms skip. The step sound
/// is loaded once, when this is created.
class PluginEffects implements EffectPlugins {
  PluginEffects() : _player = AudioPlayer() {
    _loaded = _load();
  }

  static const _stepSound = 'sounds/step-change.mp3';
  static const _brewNotification = MethodChannel(
    'coffee_timer/brew_notification',
  );
  static final bool _hasBrewNotification =
      !kIsWeb && defaultTargetPlatform == TargetPlatform.android;

  final AudioPlayer _player;
  late final Future<void> _loaded;

  Future<void> _load() async {
    await _player.setReleaseMode(ReleaseMode.stop);
    await _player.setSource(AssetSource(_stepSound));
  }

  @override
  Future<void> playStepSound() async {
    await _loaded;
    await _player.resume();
  }

  @override
  Future<void> vibrate(int ms) => Vibration.vibrate(duration: ms);

  @override
  Future<void> vibratePattern(List<int> patternMs) =>
      Vibration.vibrate(pattern: patternMs);

  @override
  Future<void> keepScreenOn({required bool on}) =>
      WakelockPlus.toggle(enable: on);

  @override
  Future<void> showBrewNotification({
    required String title,
    String? next,
  }) async {
    if (!_hasBrewNotification) return;
    await _brewNotification.invokeMethod<void>('show', {
      'title': title,
      'next': next,
    });
  }

  @override
  Future<void> hideBrewNotification() async {
    if (!_hasBrewNotification) return;
    await _brewNotification.invokeMethod<void>('hide');
  }

  @override
  Future<void> copyToClipboard(String text) =>
      Clipboard.setData(ClipboardData(text: text));
}
