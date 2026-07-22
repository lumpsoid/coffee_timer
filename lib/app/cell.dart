/// A tiny typed holder for a single long-lived instance.
///
/// The construction root (`AppRoot`) writes each cell exactly once; feature
/// pages read [value] to reach shared modules and controllers. Reading before
/// the cell is set is a programming error, surfaced as a [StateError].
class Cell<T> {
  T? _value;

  bool get isSet => _value != null;

  T get value =>
      _value ?? (throw StateError('Cell<$T> read before it was set'));

  set value(T instance) => _value = instance;
}
