import 'package:coffee_timer/core/recipe/buses/buses.dart';
import 'package:flutter/widgets.dart';

extension RecipeEventBusX on BuildContext {
  RecipeEventBus get recipeEventBus => RecipeEventBus.instance;
}
