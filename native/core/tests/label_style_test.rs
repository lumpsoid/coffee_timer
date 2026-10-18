use std::collections::HashSet;

use coffee_timer_core::model::{
    BrewMetrics, CoffeeDose, LabelShade, LabelStyle, LabelTexture, LabelTone, ModelError, Recipe,
    RecipeId, RecipeName, RecipeStep, StainCorner, StepDuration, StepId, WaterAmount,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn recipe(id: &str, name: &str, step_seconds: &[i64]) -> Result<Recipe, ModelError> {
    let steps = step_seconds
        .iter()
        .enumerate()
        .map(|(i, &s)| {
            Ok(RecipeStep::new(
                StepId::try_parse(&format!("s{i}"))?,
                "step".to_owned(),
                StepDuration::clamped(s),
                None,
            ))
        })
        .collect::<Result<_, ModelError>>()?;
    Ok(Recipe::new(
        RecipeId::try_parse(id)?,
        RecipeName::try_parse(name)?,
        String::new(),
        CoffeeDose::try_parse(20)?,
        WaterAmount::try_parse(100)?,
        steps,
    ))
}

fn style(recipe: &Recipe) -> LabelStyle {
    LabelStyle::of(recipe, &BrewMetrics::default())
}

// LabelTone

#[test]
fn tone_turns_caramel_at_two_minutes_and_crema_at_four() {
    assert_eq!(LabelTone::for_brew_seconds(0), LabelTone::Roast);
    assert_eq!(LabelTone::for_brew_seconds(119), LabelTone::Roast);
    assert_eq!(LabelTone::for_brew_seconds(120), LabelTone::Caramel);
    assert_eq!(LabelTone::for_brew_seconds(239), LabelTone::Caramel);
    assert_eq!(LabelTone::for_brew_seconds(240), LabelTone::Crema);
    assert_eq!(LabelTone::for_brew_seconds(6000), LabelTone::Crema);
}

#[test]
fn tone_follows_the_recipes_total_brew_time() -> TestResult {
    assert_eq!(style(&recipe("r", "R", &[30, 25])?).tone(), LabelTone::Roast);
    assert_eq!(style(&recipe("r", "R", &[40, 30, 120])?).tone(), LabelTone::Caramel);
    assert_eq!(style(&recipe("r", "R", &[200, 40])?).tone(), LabelTone::Crema);
    assert_eq!(style(&recipe("r", "R", &[])?).tone(), LabelTone::Roast);
    Ok(())
}

// LabelStyle

#[test]
fn the_seeded_recipes_keep_their_labels_across_builds() -> TestResult {
    // Pinned: a change here repaints every user's existing recipes.
    let espresso = style(&recipe("espresso-uuid", "Espresso", &[30, 25])?);
    assert_eq!(espresso.tone(), LabelTone::Roast);
    assert_eq!(espresso.texture(), LabelTexture::Linen);
    assert_eq!(espresso.shade(), LabelShade::Lighter);
    assert_eq!(espresso.stain(), StainCorner::TopLeft);

    let pour_over = style(&recipe("pourover-uuid", "Pour Over", &[40, 30, 120])?);
    assert_eq!(pour_over.tone(), LabelTone::Caramel);
    assert_eq!(pour_over.texture(), LabelTexture::Linen);
    assert_eq!(pour_over.shade(), LabelShade::Darker);
    assert_eq!(pour_over.stain(), StainCorner::BottomRight);
    Ok(())
}

#[test]
fn the_look_depends_on_the_id_not_the_name_or_steps() -> TestResult {
    let before = style(&recipe("r1", "V60", &[30])?);
    let after = style(&recipe("r1", "Renamed", &[30, 10])?);
    assert_eq!(before.texture(), after.texture());
    assert_eq!(before.shade(), after.shade());
    assert_eq!(before.stain(), after.stain());
    Ok(())
}

#[test]
fn recipes_of_one_tone_still_get_every_texture_shade_and_stain() -> TestResult {
    let styles = (0..60)
        .map(|n| Ok(style(&recipe(&format!("r{n}"), "Shot", &[30])?)))
        .collect::<Result<Vec<_>, ModelError>>()?;
    assert!(styles.iter().all(|s| s.tone() == LabelTone::Roast));
    let textures: HashSet<_> = styles.iter().map(|s| s.texture()).collect();
    let shades: HashSet<_> = styles.iter().map(|s| s.shade()).collect();
    let stains: HashSet<_> = styles.iter().map(|s| s.stain()).collect();
    assert_eq!(textures.len(), LabelTexture::ALL.len());
    assert_eq!(shades.len(), LabelShade::ALL.len());
    assert_eq!(stains.len(), StainCorner::ALL.len());
    Ok(())
}
