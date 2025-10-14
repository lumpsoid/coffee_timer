import 'dart:async';

import 'package:coffee_timer/core/events/events.dart';

abstract class RecipeEventBus {
  static final RecipeEventBus instance = _RecipeEventBusImpl();

  Stream<Event> get stream;
  void dispose();
  void fire(Event event);
}

class _RecipeEventBusImpl implements RecipeEventBus {
  final _controller = StreamController<Event>.broadcast();

  @override
  Stream<Event> get stream => _controller.stream;
  @override
  void dispose() => _controller.close();

  @override
  void fire(Event event) => _controller.add(event);
}
