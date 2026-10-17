import 'package:audioplayers/audioplayers.dart';
import 'package:vibration/vibration.dart';
import 'package:wakelock_plus/wakelock_plus.dart';

/// The platform plugins a `PlatformEffect` reaches: the border a test
/// replaces with a fake.
abstract interface class EffectPlugins {
  Future<void> playStepSound();

  Future<void> vibrate(int ms);

  Future<void> keepScreenOn({required bool on});
}

/// [EffectPlugins] on `audioplayers`, `vibration` and `wakelock_plus`. The
/// step sound is loaded once, when this is created.
class PluginEffects implements EffectPlugins {
  PluginEffects() : _player = AudioPlayer() {
    _loaded = _load();
  }

  static const _stepSound = 'sounds/step-change.mp3';

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
  Future<void> keepScreenOn({required bool on}) =>
      WakelockPlus.toggle(enable: on);
}
