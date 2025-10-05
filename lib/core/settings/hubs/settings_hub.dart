import 'package:coffee_timer/core/settings/actors/actors.dart';
import 'package:coffee_timer/core/settings/buses/buses.dart';
import 'package:coffee_timer/core/settings/storage/storage.dart';

abstract class SettingsHub {
  static final SettingsHub instance = _SettingsHubImpl();

  SettingsEventBus get bus;
  SettingsActor get actor;
  SettingsStorageSqlite get storage;
}

class _SettingsHubImpl implements SettingsHub {
  _SettingsHubImpl();

  final SettingsEventBus _bus = SettingsEventBus();
  @override
  SettingsEventBus get bus => _bus;

  late final SettingsActor _actor = SettingsActor(bus: _bus);
  @override
  SettingsActor get actor => _actor;

  late final SettingsStorageSqlite _storage = SettingsStorageSqlite(bus: _bus);
  @override
  SettingsStorageSqlite get storage => _storage;
}
