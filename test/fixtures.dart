import 'package:coffee_timer/src/bindings/bindings.dart';

/// Hand-built signal values, shaped like the ones Rust sends.
RecipeSummary summary(
  String id, {
  String name = 'V60',
  bool pinned = false,
}) => RecipeSummary(
  id: id,
  name: name,
  description: 'clean',
  coffeeLabel: '20 g',
  waterLabel: '100 g',
  timeLabel: '1:35',
  label: label(),
  pinned: pinned,
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
  coffee: '15',
  water: '250',
  totalLabel: '0:30',
  steps: const [
    EditorStep(
      id: 's1',
      nameLabel: 'Step 1 name',
      name: 'Bloom',
      durationText: '30',
      durationLabel: '00:30',
      waterText: '',
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
  ratioLabel: '1:5',
  ratio: BrewRatioSetting.recipe,
  durationLabel: '0:30',
  label: label(),
  steps: [
    for (final (i, status)
        in (statuses ?? [BrewStepStatus.pending, BrewStepStatus.pending])
            .indexed)
      BrewStep(
        numberLabel: '${i + 1}',
        name: 'Step ${i + 1}',
        durationLabel: '15s',
        status: status,
        trackStart: i / 2,
        trackEnd: (i + 1) / 2,
      ),
  ],
);

SettingsUpdate settings({
  ThemeChoice theme = ThemeChoice.system,
  int accent = 0xFF2196F3,
  bool sound = true,
  bool vibration = true,
  int preAlertSeconds = 3,
}) => SettingsUpdate(
  theme: theme,
  accent: accent,
  themes: const [ThemeChoice.light, ThemeChoice.dark, ThemeChoice.system],
  accents: const [
    AccentChoice(name: 'Blue', argb: 0xFF2196F3),
    AccentChoice(name: 'Red', argb: 0xFFF44336),
  ],
  sound: sound,
  vibration: vibration,
  preAlertSeconds: preAlertSeconds,
  preAlerts: const [
    PreAlertChoice(seconds: 0, label: 'Off'),
    PreAlertChoice(seconds: 3, label: '3 s'),
    PreAlertChoice(seconds: 5, label: '5 s'),
    PreAlertChoice(seconds: 10, label: '10 s'),
  ],
);
