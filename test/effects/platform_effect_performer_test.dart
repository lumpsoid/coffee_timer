import 'dart:async';

import 'package:coffee_timer/effects/effect_plugins.dart';
import 'package:coffee_timer/effects/platform_effect_performer.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:flutter_test/flutter_test.dart';

/// Records each plugin call. The sound takes a moment, so a later effect
/// that overtook it would show up out of order.
class _FakePlugins implements EffectPlugins {
  final calls = <String>[];
  bool failSound = false;

  @override
  Future<void> playSound(String asset) async {
    await Future<void>.delayed(const Duration(milliseconds: 5));
    if (failSound) throw StateError('no audio');
    calls.add('sound $asset');
  }

  @override
  Future<void> vibrate(int ms) async => calls.add('vibrate $ms');

  @override
  Future<void> vibratePattern(List<int> patternMs) async =>
      calls.add('vibrate pattern ${patternMs.join('/')}');

  @override
  Future<void> keepScreenOn({required bool on}) async =>
      calls.add('screen $on');

  @override
  Future<void> showBrewNotification({
    required String title,
    String? next,
  }) async => calls.add('notify $title | $next');

  @override
  Future<void> hideBrewNotification() async => calls.add('notify off');

  @override
  Future<void> copyToClipboard(String text) async => calls.add('copy $text');
}

void main() {
  late _FakePlugins plugins;
  late PlatformEffectPerformer performer;

  setUp(() {
    plugins = _FakePlugins();
    performer = PlatformEffectPerformer(plugins);
  });

  test('each variant is one plugin call', () async {
    await performer.perform(const PlatformEffectPlaySound(sound: 'kalimba'));
    await performer.perform(const PlatformEffectVibrate(ms: 100));
    await performer.perform(
      const PlatformEffectVibratePattern(patternMs: [0, 400, 200, 400]),
    );
    await performer.perform(const PlatformEffectKeepScreenOn(on: true));
    await performer.perform(const PlatformEffectKeepScreenOn(on: false));
    await performer.perform(
      const PlatformEffectShowBrewNotification(
        title: 'Bloom · 00:35',
        next: 'Next: Pour',
      ),
    );
    await performer.perform(
      const PlatformEffectShowBrewNotification(title: 'Pour · 00:10'),
    );
    await performer.perform(const PlatformEffectHideBrewNotification());
    await performer.perform(
      const PlatformEffectCopyToClipboard(text: '{"version":1}'),
    );

    expect(plugins.calls, [
      'sound sounds/kalimba.mp3',
      'vibrate 100',
      'vibrate pattern 0/400/200/400',
      'screen true',
      'screen false',
      'notify Bloom · 00:35 | Next: Pour',
      'notify Pour · 00:10 | null',
      'notify off',
      'copy {"version":1}',
    ]);
  });

  test('effects from the stream run in arrival order', () async {
    final effects = StreamController<PlatformEffect>();
    performer.listen(effects.stream);

    effects
      ..add(const PlatformEffectVibrate(ms: 100))
      ..add(const PlatformEffectPlaySound(sound: 'tingsha'))
      ..add(const PlatformEffectKeepScreenOn(on: false));
    await effects.close();
    await performer.perform(const PlatformEffectVibrate(ms: 1));

    expect(plugins.calls, [
      'vibrate 100',
      'sound sounds/tingsha.mp3',
      'screen false',
      'vibrate 1',
    ]);
  });

  test('a failed call does not hold up the next', () async {
    plugins.failSound = true;
    unawaited(
      performer.perform(const PlatformEffectPlaySound(sound: 'tingsha')),
    );
    await performer.perform(const PlatformEffectVibrate(ms: 100));

    expect(plugins.calls, ['vibrate 100']);
  });
}
