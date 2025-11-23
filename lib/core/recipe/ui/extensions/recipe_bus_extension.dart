import 'package:coffee_timer/core/di.dart';
import 'package:coffee_timer/core/events/events.dart';
import 'package:flutter/widgets.dart';

extension RecipeEventBusX on BuildContext {
  Stream<Event> get recipeStream => kernel.getStream('recipe');
}
