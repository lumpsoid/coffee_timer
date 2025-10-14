import 'dart:async';

import 'package:coffee_timer/core/events/events.dart';

abstract class SettingsEventBus {
  static final SettingsEventBus instance = _SettingsEventBusImpl();

  Stream<Event> get stream;
  void fire(Event event);
  void init();
  void dispose();
}

class _SettingsEventBusImpl implements SettingsEventBus {
  StreamController<Event>? _controller;

  @override
  void init() {
    _controller = StreamController<Event>.broadcast();
  }

  @override
  void dispose() {
    unawaited(_controller?.close());
    _controller = null;
  }

  @override
  Stream<Event> get stream => _controller!.stream;

  @override
  void fire(Event event) => _controller?.add(event);
}
