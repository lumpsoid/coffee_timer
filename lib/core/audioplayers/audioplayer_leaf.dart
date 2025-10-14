import 'dart:async';

import 'package:audioplayers/audioplayers.dart';
import 'package:coffee_timer/core/audio_asset.dart';
import 'package:coffee_timer/core/events/events.dart';

class AudioPlayerLeaf {
  AudioPlayerLeaf();

  AudioPlayer? _audioPlayer;
  String _loadedAsset = '';

  Future<void> handle(Event event) async {
    switch (event) {
      case AudioInitialize _:
        await _init();

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
          await _init();
          _loadedAsset = AudioAsset.timer;
          await _audioPlayer!.setSource(AssetSource(_loadedAsset));
        }
        await _audioPlayer?.resume();

      default:
        return;
    }
  }

  Future<void> _init() async {
    if (_audioPlayer != null) return;

    _audioPlayer = AudioPlayer();
    await _audioPlayer!.setReleaseMode(ReleaseMode.stop);
  }
}
