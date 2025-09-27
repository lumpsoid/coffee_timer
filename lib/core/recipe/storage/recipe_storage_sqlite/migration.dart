class Migration {
  const Migration({
    required this.version,
    required this.statements,
  });

  final int version;
  final List<String> statements;
}

