/// The theme options the user can choose. A closed set — a value outside these
/// three cannot exist, so no downstream code needs a default branch.
enum ThemeSelection {
  light('light'),
  dark('dark'),
  system('system');

  const ThemeSelection(this.storageValue);

  /// The string persisted in storage.
  final String storageValue;

  /// Parses a stored value, falling back to [ThemeSelection.system] for
  /// anything unrecognised or missing.
  static ThemeSelection fromStorage(String? value) {
    for (final selection in ThemeSelection.values) {
      if (selection.storageValue == value) {
        return selection;
      }
    }
    return ThemeSelection.system;
  }
}
