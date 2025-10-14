import 'package:coffee_timer/core/events/events.dart';

class AudioInitialize extends Event {
  const AudioInitialize();
}

class AudioDispose extends Event {
  const AudioDispose();
}

class AudioLoadAsset extends Event {
  const AudioLoadAsset(this.asset);
  final String asset;
}

class AudioUnloadAssetLatest extends Event {
  const AudioUnloadAssetLatest();
}

class AudioPlayTimerSound extends Event {
  const AudioPlayTimerSound();
}
