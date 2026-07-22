import 'package:coffee_timer/app/cell.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('Cell', () {
    test('is not set before a value is written', () {
      expect(Cell<int>().isSet, isFalse);
    });

    test('exposes the value once set', () {
      final cell = Cell<int>()..value = 7;
      expect(cell.isSet, isTrue);
      expect(cell.value, 7);
    });

    test('throws when read before being set', () {
      expect(() => Cell<int>().value, throwsStateError);
    });
  });
}
