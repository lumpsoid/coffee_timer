import 'dart:async';

import 'package:audioplayers/audioplayers.dart';
import 'package:coffee_timer/core/audio_asset.dart';
import 'package:coffee_timer/core/di/hub.dart';
import 'package:coffee_timer/core/events/events.dart';

class AudioPlayerLeaf extends Leaf {
  AudioPlayerLeaf();

  AudioPlayer? _audioPlayer;
  String _loadedAsset = '';

  @override
  Future<void> handleEvent(Event event) async {
    switch (event) {
      case AudioInitialize _:
        await initialize();

      case AudioDispose _:
        await _audioPlayer?.dispose();

      case AudioLoadAsset():
        if (_loadedAsset == event.asset) return;

        _loadedAsset = event.asset;
        await _audioPlayer!.setSource(AssetSource(_loadedAsset));

      case AudioUnloadAssetLatest _:
        if (_loadedAsset.isEmpty) return;

        _loadedAsset = '';
        await _audioPlayer?.release();

      case AudioPlayTimerSound _:
        if (_loadedAsset != AudioAsset.timer) {
          await initialize();
          _loadedAsset = AudioAsset.timer;
          await _audioPlayer!.setSource(AssetSource(_loadedAsset));
        }
        await _audioPlayer?.resume();

      default:
        return;
    }
  }

  @override
  Future<void> initialize() async {
    if (_audioPlayer != null) return;

    _audioPlayer = AudioPlayer();
    await _audioPlayer!.setReleaseMode(ReleaseMode.stop);
  }
}
