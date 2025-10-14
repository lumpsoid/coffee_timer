import 'dart:async';

class Datum {
  Datum(this.value);
  final dynamic value;

  Datum clone() => Datum(value);
}

class MessageEvent {
  MessageEvent({
    required this.port,
    required this.datum,
  });
  final String port;
  final Datum datum;

  MessageEvent copyWith({
    String? port,
    Datum? datum,
  }) {
    return MessageEvent(
      port: port ?? this.port,
      datum: datum ?? this.datum,
    );
  }

  @override
  String toString() => 'MessageEvent(port: $port, datum: ${datum.value})';
}

enum ComponentType {
  container,
  leaf,
}

class Sender {
  Sender({
    required this.component,
    required this.port,
  });
  final Component component;
  final String port;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is Sender &&
          runtimeType == other.runtimeType &&
          identical(component, other.component) &&
          port == other.port;

  @override
  int get hashCode => component.hashCode ^ port.hashCode;
}

class Receiver {
  Receiver({
    required this.component,
    required this.port,
    required this.queue,
  });
  final Component component;
  final String port;
  final List<MessageEvent> queue;
}

class Connector {
  Connector({
    required this.sender,
    required this.receiver,
  });
  final Sender sender;
  final Receiver receiver;
}

abstract class Component {
  Component({
    required this.name,
    required this.type,
    this.owner,
  });
  static ContainerComponent rootContainer(String name) {
    return ContainerComponent(name: name);
  }

  static ContainerComponent container(String name, Component? owner) {
    return ContainerComponent(name: name, owner: owner);
  }

  static LeafComponent<T> leaf<T>({
    required String name,
    Component? owner,
    T? instanceData,
    EventHandler<T>? handler,
  }) {
    return LeafComponent<T>(
      name: name,
      owner: owner,
      instanceData: instanceData,
      handler: handler,
    );
  }

  final String name;
  final ComponentType type;
  Component? owner;

  final List<MessageEvent> _inQueue = [];
  final List<MessageEvent> _outQueue = [];

  String state = 'idle';

  List<MessageEvent> get inQueue => _inQueue;
  List<MessageEvent> get outQueue => _outQueue;

  Component get root {
    var current = this;
    while (current.owner != null) {
      current = current.owner!;
    }
    return current;
  }

  void send(String port, dynamic value) {
    print('$name sending to $port');
    final event = MessageEvent(
      port: port,
      datum: Datum(value),
    );
    _outQueue.add(event);
  }

  void inject(String port, dynamic value) {
    final event = MessageEvent(
      port: port,
      datum: Datum(value),
    );
    _inQueue.add(event);
    _wakeUpRecursion();
  }

  void _wakeUpRecursion() {
    final rootComp = root;
    if (rootComp is ContainerComponent) {
      rootComp.wakeUp();
    }
  }

  void handle(MessageEvent event);

  bool get isReady {
    return _inQueue.isNotEmpty ||
        _outQueue.isNotEmpty ||
        state != 'idle' ||
        _anyChildReady();
  }

  bool _anyChildReady() => false;
}

typedef EventHandler<T> =
    void Function(LeafComponent<T> component, MessageEvent event);

class LeafComponent<T> extends Component {
  LeafComponent({
    required super.name,
    super.owner,
    this.handler,
    this.instanceData,
  }) : super(
         type: ComponentType.leaf,
       );
  final EventHandler<T>? handler;
  T? instanceData;

  @override
  void handle(MessageEvent event) {
    handler?.call(this, event);
  }
}

class ContainerComponent extends Component {
  ContainerComponent({
    required super.name,
    super.owner,
  }) : super(
         type: ComponentType.container,
       );
  final List<Component> _children = [];
  final List<Connector> _connectors = [];
  final Map<String, Component> _childrenByName = {};

  // Wake mechanism using Completer for efficient blocking
  Completer<void>? _wakeCompleter;
  bool _isRunning = false;

  List<Component> get children => List.unmodifiable(_children);
  List<Connector> get connectors => List.unmodifiable(_connectors);

  void wakeUp() {
    if (_wakeCompleter != null && !_wakeCompleter!.isCompleted) {
      _wakeCompleter!.complete();
    }
  }

  void addChild(Component child) {
    child.owner = this;
    _children.add(child);
    _childrenByName[child.name] = child;
  }

  void addChildren(List<Component> children) {
    children.forEach(addChild);
  }

  Component? findChild(String name) => _childrenByName[name];

  void withConnectors(List<Connector> connectors) {
    _connectors.addAll(connectors);
  }

  @override
  void handle(MessageEvent event) {
    route(this, event);
    // Exhaust all work
    while (_anyChildReady()) {
      stepAll();
    }
  }

  void route(Component from, MessageEvent event) {
    final sender = Sender(component: from, port: event.port);

    // Broadcast to all children if port is "."
    if (event.port == '.') {
      for (final child in _children) {
        child.inQueue.add(event);
      }
      return;
    }

    var sent = false;
    for (final connector in _connectors) {
      if (connector.sender == sender) {
        final clone = MessageEvent(
          port: connector.receiver.port,
          datum: event.datum.clone(),
        );
        connector.receiver.queue.add(clone);
        sent = true;
      }
    }

    if (!sent) {
      print('✗ DROPPED: $name: event "${event.port}" from <${from.name}>');
    }
  }

  @override
  bool _anyChildReady() {
    return _children.any((child) => child.isReady);
  }

  void stepAll() {
    state = 'idle';

    // Visit each child once in fixed order
    for (final child in _children) {
      print('${child.name} InputQ=${child.inQueue.length}');

      if (child.inQueue.isNotEmpty) {
        final event = child.inQueue.removeAt(0);
        child.handle(event);
      } else if (child.state != 'idle') {
        // Tick event
        final tick = MessageEvent(port: '.', datum: Datum('!'));
        child.handle(tick);
      }

      // Route anything that appeared in output queue
      while (child.outQueue.isNotEmpty) {
        final event = child.outQueue.removeAt(0);
        route(child, event);
      }

      if (child.state == 'active') {
        state = 'active';
      }
    }
  }

  //-------------------------  RUN METHODS  -------------------------

  /// Single-threaded run-to-completion: exhausts all work and returns
  void runToCompletion() {
    while (_anyChildReady()) {
      stepAll();
    }
  }

  /// Synchronous run: exhausts all current work then returns
  void run() {
    runToCompletion();
  }

  /// Async run with wake mechanism: runs indefinitely until stopped
  Future<void> runAsync() async {
    _isRunning = true;

    while (_isRunning) {
      // Exhaust all work
      runToCompletion();

      // Create a new completer for the next wake-up
      _wakeCompleter = Completer<void>();

      // Wait for wake-up signal
      await _wakeCompleter!.future;
    }
  }

  /// Run with context/cancellation support
  Future<void> runWithContext(Future<void> cancellationFuture) async {
    _isRunning = true;

    while (_isRunning) {
      // Exhaust all work
      runToCompletion();

      // Create a new completer for the next wake-up
      _wakeCompleter = Completer<void>();

      // Wait for either wake-up or cancellation
      final result = await Future.any([
        _wakeCompleter!.future.then((_) => 'wake'),
        cancellationFuture.then((_) => 'cancel'),
      ]);

      if (result == 'cancel') {
        _isRunning = false;
        break;
      }
    }
  }

  /// Stop the async run loop
  void stop() {
    _isRunning = false;
    wakeUp(); // Wake up to exit the loop
  }
}

//-------------------------  CONNECTOR BUILDER  -------------------------

class ConnectorBuilder {
  ConnectorBuilder(this.container);
  final ContainerComponent container;

  Connector down({
    required String senderPort,
    required String receiverName,
    required String receiverPort,
  }) {
    final receiver = container.findChild(receiverName);
    if (receiver == null) {
      throw Exception('Child "$receiverName" not found');
    }

    return Connector(
      sender: Sender(component: container, port: senderPort),
      receiver: Receiver(
        component: receiver,
        port: receiverPort,
        queue: receiver.inQueue,
      ),
    );
  }

  Connector up({
    required String senderName,
    required String senderPort,
    required String receiverPort,
  }) {
    final sender = container.findChild(senderName);
    if (sender == null) {
      throw Exception('Child "$senderName" not found');
    }

    return Connector(
      sender: Sender(component: sender, port: senderPort),
      receiver: Receiver(
        component: container,
        port: receiverPort,
        queue: container.outQueue,
      ),
    );
  }

  Connector across({
    required String senderName,
    required String senderPort,
    required String receiverName,
    required String receiverPort,
  }) {
    final sender = container.findChild(senderName);
    final receiver = container.findChild(receiverName);

    if (sender == null || receiver == null) {
      throw Exception(
        'Sender or receiver not found ($senderName, $receiverName)',
      );
    }

    return Connector(
      sender: Sender(component: sender, port: senderPort),
      receiver: Receiver(
        component: receiver,
        port: receiverPort,
        queue: receiver.inQueue,
      ),
    );
  }

  Connector through({
    required String senderPort,
    required String receiverPort,
  }) {
    return Connector(
      sender: Sender(component: container, port: senderPort),
      receiver: Receiver(
        component: container,
        port: receiverPort,
        queue: container.outQueue,
      ),
    );
  }
}
