import 'dart:async';
import 'package:coffee_timer/core/events/events.dart';

/// Base class for all leaf components in the system
abstract class Leaf {
  // Private nullable emit function
  void Function(Event)? _emit;

  // Public emit method that checks if _emit is null
  void emit(Event event) {
    _emit?.call(event);
  }

  // Abstract method to handle events
  Future<void> handleEvent(Event event);

  // Optional method for initialization
  void initialize() {}

  // Optional method for cleanup
  void dispose() {}
}

/// A dynamic stream manager that handles all event buses and subscriptions
class Hub {
  factory Hub() => _instance;
  Hub._internal();
  static final Hub _instance = Hub._internal();

  // Map to store all stream controllers by name
  final Map<String, StreamController<Event>> _controllers = {};

  // Map to store all subscriptions by module name
  final Map<String, StreamSubscription<Event>> _subscriptions = {};

  // Map to store all registered leaves
  final Map<String, Leaf> _leaves = {};

  /// Creates or gets a stream controller for the specified bus name
  StreamController<Event> _getController(String busName) {
    if (!_controllers.containsKey(busName)) {
      _controllers[busName] = StreamController<Event>.broadcast();
    }
    return _controllers[busName]!;
  }

  /// Gets a stream for the specified bus name
  Stream<Event> getStream(String busName) {
    return _getController(busName).stream;
  }

  /// Fires an event to the specified bus
  void emit(String busName, Event event) {
    if (_controllers.containsKey(busName)) {
      _controllers[busName]!.add(event);
    } else {
      print('Warning: Bus "$busName" not found. Event not fired.');
    }
  }

  void emitToLeaf(String leafName, Event event) {
    if (_leaves.containsKey(leafName)) {
      unawaited(_leaves[leafName]!.handleEvent(event));
    } else {
      print('Warning: Leaf "$leafName" not found. Event not fired.');
    }
  }

  void registerBus(String name) {
    if (_controllers.containsKey(name)) return;

    _controllers[name] = StreamController<Event>.broadcast();
  }

  Leaf getLeaf(String name) {
    final leaf = _leaves[name];
    if (leaf == null) throw Exception('Leaf is not registered');

    return leaf;
  }

  /// Registers a leaf with the hub and subscribes it to a bus
  void registerLeaf({
    required String busName,
    required String name,
    required Leaf leaf,
  }) {
    // Store the leaf
    _leaves[name] = leaf;

    // Set up the private emit function
    final controller = _getController(busName);
    leaf._emit = controller.add;

    // Subscribe to the bus
    subscribe(
      busName: busName,
      moduleName: name,
      handler: leaf.handleEvent,
    );

    // Initialize the leaf
    leaf.initialize();
  }

  /// Subscribes a module to a bus with an event handler
  void subscribe({
    required String busName,
    required String moduleName,
    required void Function(Event) handler,
  }) {
    if (_subscriptions.containsKey(moduleName)) return;

    _subscriptions[moduleName] = getStream(busName).listen(handler);
  }

  /// Disposes all subscriptions for a specific module
  void disposeModule(String moduleName) {
    if (_subscriptions.containsKey(moduleName)) {
      unawaited(_subscriptions[moduleName]!.cancel());
      _subscriptions.remove(moduleName);
    }
  }

  /// Disposes a specific bus
  void disposeBus(String busName) {
    if (_controllers.containsKey(busName)) {
      unawaited(_controllers[busName]!.close());
      _controllers.remove(busName);
    }
  }

  /// Disposes all buses and subscriptions
  void disposeAll() {
    // Cancel all subscriptions
    for (final moduleSubscriptions in _subscriptions.values) {
      unawaited(moduleSubscriptions.cancel());
    }
    _subscriptions.clear();

    // Close all controllers
    for (final controller in _controllers.values) {
      unawaited(controller.close());
    }
    _controllers.clear();
  }

  /// Gets all active bus names
  List<String> get activeBuses => _controllers.keys.toList();

  /// Gets all active module names
  List<String> get activeModules => _subscriptions.keys.toList();
}
