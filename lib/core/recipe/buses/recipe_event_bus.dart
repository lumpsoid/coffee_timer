import 'dart:async';

import 'package:coffee_timer/core/recipe/events/events.dart';

class RecipeEventBus {
  factory RecipeEventBus() => _instance;

  RecipeEventBus._internal();

  static final RecipeEventBus _instance = RecipeEventBus._internal();

  final _controller = StreamController<RecipeEvent>.broadcast();

  Stream<RecipeEvent> get stream => _controller.stream;
  void dispose() => _controller.close();

  void fire(RecipeEvent event) => _controller.add(event);
}
