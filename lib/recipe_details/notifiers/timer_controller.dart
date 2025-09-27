import 'dart:async';
import 'package:flutter/material.dart';

class TimerController extends ChangeNotifier {
  Timer? _timer;
  int _totalSeconds = 0;
  int _remainingSeconds = 0;
  bool _started = false;

  bool get started => _started;
  int get remainingSeconds => _remainingSeconds;
  double get progress =>
      _totalSeconds == 0 ? 0 : 1 - (_remainingSeconds / _totalSeconds);

  void initialize(int totalSeconds) {
    _totalSeconds = totalSeconds;
    _remainingSeconds = totalSeconds;
    _started = false;
    notifyListeners();
  }

  void start() {
    if (_started || _totalSeconds == 0) return;

    _started = true;
    notifyListeners();

    _timer = Timer.periodic(const Duration(seconds: 1), (timer) {
      if (_remainingSeconds <= 1) {
        stop();
      } else {
        _remainingSeconds--;
        notifyListeners();
      }
    });
  }

  void stop() {
    _timer?.cancel();
    _started = false;
    _remainingSeconds = _totalSeconds;
    notifyListeners();
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }
}
