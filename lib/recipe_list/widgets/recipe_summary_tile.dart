import 'package:flutter/material.dart';

class RecipeSummaryTile extends StatelessWidget {
  const RecipeSummaryTile({
    required this.name,
    required this.description,
    required this.coffeeGrams,
    required this.waterGrams,
    required this.duration,
    this.onTap,
    this.onLongPress,
    super.key,
  });

  final String name;
  final String description;
  final int coffeeGrams;
  final int waterGrams;
  final String duration;
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
                    TextSpan(
                      text: ' — ',
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    TextSpan(
                      text: description,
                      style: Theme.of(context).textTheme.bodyMedium,
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 8),
              const Divider(height: 1),
              const SizedBox(height: 8),
              // Parameters row
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text('Coffee: $coffeeGrams g'),
                  Text('Water: $waterGrams g'),
                  Text('Time: $duration'),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
