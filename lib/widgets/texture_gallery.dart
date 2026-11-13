import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:coffee_timer/widgets/label_paper.dart';
import 'package:flutter/material.dart';

/// The label texture gallery: one painted tile per texture Rust offers;
/// tapping one reports its texture (null is Auto).
///
/// Rust decides which tile is chosen ([EditorTexture.chosen]). A tap only
/// reports; the gallery shows the new choice when Rust's next draft arrives.
class TextureGallery extends StatelessWidget {
  const TextureGallery({
    required this.textures,
    required this.onPick,
    super.key,
  });

  final List<EditorTexture> textures;
  final ValueChanged<LabelTexture?> onPick;

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: [
        for (final texture in textures)
          _Tile(
            key: ValueKey(texture.label),
            texture: texture,
            onPick: onPick,
          ),
      ],
    );
  }
}

/// One texture's tile: its paper swatch, a check and border when chosen, and
/// its name under it.
class _Tile extends StatelessWidget {
  const _Tile({required this.texture, required this.onPick, super.key});

  final EditorTexture texture;
  final ValueChanged<LabelTexture?> onPick;

  static const _swatchWidth = 84.0;
  static const _swatchHeight = 56.0;
  static const _radius = 6.0;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final primary = theme.colorScheme.primary;
    // Fixed crema inks: the gallery compares patterns, not the recipe's tone.
    final colors = LabelColors.of(
      const RecipeLabel(
        tone: LabelTone.crema,
        texture: LabelTexture.plain,
        shade: LabelShade.base,
        stain: StainCorner.bottomRight,
      ),
      theme.brightness,
    );
    return Semantics(
      button: true,
      selected: texture.chosen,
      label: texture.label,
      onTap: () => onPick(texture.texture),
      excludeSemantics: true,
      child: InkWell(
        borderRadius: BorderRadius.circular(_radius),
        onTap: () => onPick(texture.texture),
        child: Column(
          children: [
            Stack(
              children: [
                ClipRRect(
                  borderRadius: BorderRadius.circular(_radius),
                  child: LabelPaper(
                    colors: colors,
                    texture: texture.paper,
                    // No ring: on a swatch this small it hides the pattern.
                    stain: null,
                    child: const SizedBox(
                      width: _swatchWidth,
                      height: _swatchHeight,
                    ),
                  ),
                ),
                if (texture.chosen) ...[
                  Positioned.fill(
                    child: DecoratedBox(
                      decoration: BoxDecoration(
                        borderRadius: BorderRadius.circular(_radius),
                        border: Border.all(color: primary, width: 2),
                      ),
                    ),
                  ),
                  Positioned(
                    top: 4,
                    right: 4,
                    child: Icon(Icons.check_circle, size: 18, color: primary),
                  ),
                ],
              ],
            ),
            const SizedBox(height: 4),
            Text(
              texture.label,
              style: theme.textTheme.labelMedium,
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }
}
