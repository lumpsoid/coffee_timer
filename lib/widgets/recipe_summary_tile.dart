import 'package:flutter/material.dart';

/// One row in the recipe list: name/description plus the brew figures. Pure
/// cosmetics: the figures arrive as labels computed in Rust (`20 g`,
/// `1:35`); only the captions are UI text.
class RecipeSummaryTile extends StatelessWidget {
  const RecipeSummaryTile({
    required this.name,
    required this.description,
    required this.coffeeLabel,
    required this.waterLabel,
    required this.timeLabel,
    this.onTap,
    this.onLongPress,
    super.key,
  });

  final String name;
  final String description;
  final String coffeeLabel;
  final String waterLabel;
  final String timeLabel;
  final VoidCallback? onTap;
  final VoidCallback? onLongPress;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onTap: onTap,
      onLongPress: onLongPress,
      child: Container(
        margin: const EdgeInsets.symmetric(vertical: 8, horizontal: 16),
        decoration: BoxDecoration(
          color: ColorScheme.of(context).surfaceContainerLow,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              RichText(
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                text: TextSpan(
                  children: [
                    TextSpan(
                      text: name,
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    if (description.isNotEmpty) ...[
                      TextSpan(
                        text: ' — ',
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                      TextSpan(
                        text: description,
                        style: Theme.of(context).textTheme.bodyMedium,
                      ),
                    ],
                  ],
                ),
              ),
              const SizedBox(height: 8),
              const Divider(height: 1),
              const SizedBox(height: 8),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text('Coffee: $coffeeLabel'),
                  Text('Water: $waterLabel'),
                  Text('Time: $timeLabel'),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
