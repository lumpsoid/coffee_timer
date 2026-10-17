import 'package:coffee_timer/src/bindings/bindings.dart';

/// Hand-built signal values, shaped like the ones Rust sends.
RecipeSummary summary(String id, {String name = 'V60'}) => RecipeSummary(
  id: id,
  name: name,
  description: 'clean',
  coffeeLabel: '20 g',
  waterLabel: '100 g',
  timeLabel: '1:35',
  label: label(),
);

RecipeLabel label({LabelTone tone = LabelTone.roast}) => RecipeLabel(
  tone: tone,
  texture: LabelTexture.linen,
  shade: LabelShade.base,
  stain: StainCorner.bottomRight,
);

EditorDraft draft({
  String id = 'r1',
  String name = 'V60',
  bool canDelete = true,
  String? notice,
}) => EditorDraft(
  id: id,
  title: canDelete ? 'Edit Recipe' : 'New Recipe',
  canDelete: canDelete,
  name: name,
  description: '',
  totalLabel: '0:30',
  steps: const [
    EditorStep(
      id: 's1',
      nameLabel: 'Step 1 name',
      name: 'Bloom',
      durationText: '30',
      durationLabel: '00:30',
    ),
  ],
  addStepButtons: const [
    AddStepButton(preset: StepPresetChoice.addCoffee, label: 'Add coffee'),
    AddStepButton(preset: StepPresetChoice.addWater, label: 'Add water'),
    AddStepButton(preset: StepPresetChoice.wait, label: 'Wait'),
    AddStepButton(preset: StepPresetChoice.blank, label: 'Add Step'),
  ],
  notice: notice,
);

BrewRecipe brewRecipe({List<BrewStepStatus>? statuses}) => BrewRecipe(
  id: 'r1',
  name: 'V60',
  description: 'clean',
  coffeeLabel: '20g',
  waterLabel: '100g',
  durationLabel: '0:30',
  steps: [
    for (final (i, status)
        in (statuses ?? [BrewStepStatus.pending, BrewStepStatus.pending])
            .indexed)
      BrewStep(
        numberLabel: '${i + 1}',
        name: 'Step ${i + 1}',
        durationLabel: '15s',
        status: status,
      ),
  ],
);

SettingsUpdate settings({
  ThemeChoice theme = ThemeChoice.system,
  int accent = 0xFF2196F3,
}) => SettingsUpdate(
  theme: theme,
  accent: accent,
  themes: const [ThemeChoice.light, ThemeChoice.dark, ThemeChoice.system],
  accents: const [
    AccentChoice(name: 'Blue', argb: 0xFF2196F3),
    AccentChoice(name: 'Red', argb: 0xFFF44336),
  ],
);
