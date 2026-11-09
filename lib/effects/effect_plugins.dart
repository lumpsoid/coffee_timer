import 'package:audioplayers/audioplayers.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:vibration/vibration.dart';
import 'package:wakelock_plus/wakelock_plus.dart';

/// The platform plugins a `PlatformEffect` reaches: the border a test
/// replaces with a fake.
abstract interface class EffectPlugins {
  /// Plays the [asset] (`sounds/kalimba.mp3`) from its start, cutting off
  /// whatever plays.
  Future<void> playSound(String asset);

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
/// channel (`MainActivity.kt`), which other platforms skip. One player
/// plays every sound; a sound is loaded when it is asked for after a
/// different one.
class PluginEffects implements EffectPlugins {
  PluginEffects() : _player = AudioPlayer() {
    _ready = _player.setReleaseMode(ReleaseMode.stop);
  }

  static const _brewNotification = MethodChannel(
    'coffee_timer/brew_notification',
  );
  static final bool _hasBrewNotification =
      !kIsWeb && defaultTargetPlatform == TargetPlatform.android;

  final AudioPlayer _player;
  late final Future<void> _ready;

  /// The asset the player holds; null until the first sound.
  String? _loaded;

  @override
  Future<void> playSound(String asset) async {
    await _ready;
    await _player.stop();
    if (asset != _loaded) {
      await _player.setSource(AssetSource(asset));
      _loaded = asset;
    }
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
