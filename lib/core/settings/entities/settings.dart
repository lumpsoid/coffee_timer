class Settings {
  const Settings({
    required this.theme,
    required this.accentColor,
  });

  factory Settings.fromMap(Map<String, Object?> map) {
    return Settings(
      theme: map['theme'] as String? ?? 'light',
      accentColor: map['accent_color'] as int? ?? 0xFF2196F3,
    );
  }

  static const empty = Settings(
    theme: 'light',
    accentColor: 0xFF2196F3, // Material Blue 500
  );

  final String theme;
  final int accentColor; // Store as ARGB integer

  Settings copyWith({
    String? theme,
    int? accentColor,
  }) {
    return Settings(
      theme: theme ?? this.theme,
      accentColor: accentColor ?? this.accentColor,
    );
  }

  Map<String, Object?> toMap() {
    return {
      'theme': theme,
      'accent_color': '#${accentColor.toRadixString(16).padLeft(8, '0')}',
    };
  }
}
