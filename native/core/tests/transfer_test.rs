//! The recipe document: encoding, decoding and its validation, and the
//! editor's import into the catalog.

use coffee_timer_core::catalog::{CatalogChange, RecipeCatalog};
use coffee_timer_core::editor::EditorController;
use coffee_timer_core::fakes::{FakeRecipeStore, SequentialIds};
use coffee_timer_core::model::{
    CoffeeDose, GrindClicks, ModelError, Pin, PourSpeed, Recipe, RecipeId, RecipeName, RecipeStep,
    StepDuration, StepId, WaterAmount,
};
use coffee_timer_core::transfer::{DocumentError, ImportError, RecipeDocument};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn pour_over() -> Result<Recipe, ModelError> {
    Ok(Recipe::new(
        RecipeId::try_parse("p")?,
        RecipeName::try_parse("Pour Over")?,
        "Light and clean".to_owned(),
        CoffeeDose::try_parse(15)?,
        WaterAmount::try_parse(250)?,
        vec![
            RecipeStep::new(
                StepId::try_parse("p1")?,
                "Grind".to_owned(),
                StepDuration::try_parse(40)?,
                None,
            ),
            RecipeStep::new(
                StepId::try_parse("p2")?,
                "Bloom".to_owned(),
                StepDuration::try_parse(30)?,
                Some(WaterAmount::try_parse(50)?),
            ),
        ],
    )
    .with_pour_speed(PourSpeed::try_parse_text("4.5")?)
    .with_grind(Some(GrindClicks::try_parse(22)?)))
}

/// A version-1 document holding one recipe, `recipe` (its JSON object).
fn document(recipe: &str) -> String {
    format!(r#"{{"format":"coffee-timer-recipes","version":1,"recipes":[{recipe}]}}"#)
}

const VALID: &str = r#"{"name":"V60","description":"Bright","coffee_grams":15,"water_grams":250,
  "steps":[{"name":"Bloom","seconds":30,"water_grams":50},{"name":"  ","seconds":90}]}"#;

fn decode_error(text: &str) -> Result<String, Box<dyn std::error::Error>> {
    match RecipeDocument::decode(text) {
        Ok(_) => Err(format!("decoded {text}").into()),
        Err(e) => Ok(e.to_string()),
    }
}

// encode

#[test]
fn encode_writes_the_versioned_document_without_ids() -> TestResult {
    let recipe = pour_over()?;
    let export = RecipeDocument::encode(&[&recipe])?;
    assert_eq!(
        export.json(),
        r#"{
  "format": "coffee-timer-recipes",
  "version": 1,
  "recipes": [
    {
      "name": "Pour Over",
      "description": "Light and clean",
      "coffee_grams": 15,
      "water_grams": 250,
      "pour_grams_per_second": 4.5,
      "grind_clicks": 22,
      "steps": [
        {
          "name": "Grind",
          "seconds": 40
        },
        {
          "name": "Bloom",
          "seconds": 30,
          "water_grams": 50
        }
      ]
    }
  ]
}"#
    );
    assert_eq!(export.count_label(), "1 recipe");
    Ok(())
}

#[test]
fn encode_then_decode_is_the_same_recipe_apart_from_ids() -> TestResult {
    let recipe = pour_over()?;
    let export = RecipeDocument::encode(&[&recipe, &recipe])?;
    assert_eq!(export.copied_notice(), "Copied 2 recipes");
    let imported = RecipeDocument::decode(export.json())?;
    assert_eq!(imported.len(), 2);

    let mut ids = SequentialIds::default();
    let back = imported
        .into_iter()
        .next()
        .ok_or("no recipe")?
        .into_recipe(&mut ids);
    assert_eq!(back.id().as_str(), "r1");
    assert_eq!(back.name(), recipe.name());
    assert_eq!(back.description(), recipe.description());
    assert_eq!(back.coffee(), recipe.coffee());
    assert_eq!(back.water(), recipe.water());
    assert_eq!(back.pour_speed(), recipe.pour_speed());
    assert_eq!(back.grind(), recipe.grind());
    let steps: Vec<_> = back
        .steps()
        .iter()
        .map(|s| (s.id().as_str(), s.name(), s.duration(), s.water()))
        .collect();
    let want: Vec<_> = recipe
        .steps()
        .iter()
        .zip(["s1", "s2"])
        .map(|(s, id)| (id, s.name(), s.duration(), s.water()))
        .collect();
    assert_eq!(steps, want);
    Ok(())
}

// decode

#[test]
fn decode_reads_values_and_names_a_blank_step() -> TestResult {
    let recipes = RecipeDocument::decode(&document(VALID))?;
    let [recipe] = recipes.as_slice() else {
        return Err("not one recipe".into());
    };
    assert_eq!(recipe.name().as_str(), "V60");
    assert_eq!(recipe.description(), "Bright");
    assert_eq!(recipe.coffee().grams(), 15);
    assert_eq!(recipe.water().grams(), 250);
    let steps: Vec<_> = recipe
        .steps()
        .iter()
        .map(|s| {
            (
                s.name(),
                s.duration().seconds(),
                s.water().map(WaterAmount::grams),
            )
        })
        .collect();
    assert_eq!(steps, [("Bloom", 30, Some(50)), ("Step 2", 90, None)]);
    Ok(())
}

#[test]
fn decode_ignores_fields_it_does_not_know_and_defaults_the_description() -> TestResult {
    let recipe = r#"{"id":"x","pinned":true,"name":"V60","coffee_grams":15,"water_grams":250,
      "steps":[{"seconds":30,"colour":"red"}]}"#;
    let recipes = RecipeDocument::decode(&document(recipe))?;
    let recipe = recipes.first().ok_or("no recipe")?;
    assert_eq!(recipe.description(), "");
    assert_eq!(
        recipe.pour_speed(),
        PourSpeed::DEFAULT,
        "a document from before pour speeds"
    );
    assert_eq!(recipe.grind(), None, "a document from before grinds");
    assert_eq!(recipe.steps().first().map(|s| s.name()), Some("Step 1"));
    Ok(())
}

#[test]
fn decode_refuses_each_invalid_value_naming_where_it_is() -> TestResult {
    let cases = [
        (
            r#"{"name":" ","coffee_grams":15,"water_grams":250,"steps":[{"seconds":1}]}"#,
            "Recipe 2, name: Enter a recipe name",
        ),
        (
            r#"{"name":"A","coffee_grams":0,"water_grams":250,"steps":[{"seconds":1}]}"#,
            "Recipe 2, coffee_grams: 1–200 g",
        ),
        (
            r#"{"name":"A","coffee_grams":15,"water_grams":2001,"steps":[{"seconds":1}]}"#,
            "Recipe 2, water_grams: 1–2000 g",
        ),
        (
            r#"{"name":"A","coffee_grams":15,"water_grams":250,"steps":[]}"#,
            "Recipe 2: no steps",
        ),
        (
            r#"{"name":"A","coffee_grams":15,"water_grams":250,
               "steps":[{"seconds":1},{"seconds":1},{"seconds":6000}]}"#,
            "Recipe 2, step 3, seconds: 0–5999 seconds",
        ),
        (
            r#"{"name":"A","coffee_grams":15,"water_grams":250,"steps":[{"seconds":-1}]}"#,
            "Recipe 2, step 1, seconds: 0–5999 seconds",
        ),
        (
            r#"{"name":"A","coffee_grams":15,"water_grams":250,
               "steps":[{"seconds":1,"water_grams":0}]}"#,
            "Recipe 2, step 1, water_grams: 1–2000 g",
        ),
        (
            r#"{"name":"A","coffee_grams":15,"water_grams":250,"pour_grams_per_second":0.4,
               "steps":[{"seconds":1}]}"#,
            "Recipe 2, pour_grams_per_second: 0.5–30 g/s",
        ),
        (
            r#"{"name":"A","coffee_grams":15,"water_grams":250,"grind_clicks":100,
               "steps":[{"seconds":1}]}"#,
            "Recipe 2, grind_clicks: 0–99 clicks",
        ),
    ];
    for (second, want) in cases {
        let text = document(&format!("{VALID},{second}"));
        assert_eq!(decode_error(&text)?, want, "{second}");
    }
    Ok(())
}

#[test]
fn decode_refuses_a_value_of_the_wrong_type_as_malformed() -> TestResult {
    let text = document(r#"{"name":"A","coffee_grams":"15","water_grams":250,"steps":[]}"#);
    let error = decode_error(&text)?;
    assert!(
        error.starts_with("Not a recipe document: invalid type"),
        "{error}"
    );
    let text = document(r#"{"name":"A","water_grams":250,"steps":[]}"#);
    let error = decode_error(&text)?;
    assert!(error.contains("missing field `coffee_grams`"), "{error}");
    Ok(())
}

#[test]
fn decode_refuses_malformed_json() -> TestResult {
    let error = decode_error("{\"format\": ")?;
    assert!(error.starts_with("Not a recipe document: "), "{error}");
    assert_eq!(
        decode_error("[1, 2]")?.split(':').next(),
        Some("Not a recipe document")
    );
    Ok(())
}

#[test]
fn decode_refuses_blank_text() -> TestResult {
    assert_eq!(
        RecipeDocument::decode("  \n").err(),
        Some(DocumentError::Empty)
    );
    assert_eq!(decode_error("")?, "The text is empty");
    Ok(())
}

#[test]
fn decode_refuses_another_format() -> TestResult {
    assert_eq!(decode_error("{}")?, "Not a Coffee Timer recipe document");
    assert_eq!(
        decode_error(r#"{"format":"other","version":1,"recipes":[]}"#)?,
        "Not a Coffee Timer recipe document"
    );
    Ok(())
}

#[test]
fn decode_refuses_an_unknown_or_missing_version() -> TestResult {
    assert_eq!(
        decode_error(r#"{"format":"coffee-timer-recipes","version":2,"recipes":"later"}"#)?,
        "Recipe document version 2 is not supported; this app reads version 1"
    );
    assert_eq!(
        decode_error(r#"{"format":"coffee-timer-recipes","recipes":[]}"#)?,
        "The recipe document has no version"
    );
    Ok(())
}

#[test]
fn decode_refuses_a_document_without_recipes() -> TestResult {
    assert_eq!(
        decode_error(r#"{"format":"coffee-timer-recipes","version":1,"recipes":[]}"#)?,
        "The recipe document has no recipes"
    );
    Ok(())
}

// EditorController::import

fn catalog(store: FakeRecipeStore) -> Result<RecipeCatalog, Box<dyn std::error::Error>> {
    let mut catalog = RecipeCatalog::new(Box::new(store));
    catalog.load()?;
    Ok(catalog)
}

#[test]
fn import_appends_new_unpinned_recipes_with_fresh_ids() -> TestResult {
    let p = pour_over()?;
    let mut catalog = catalog(FakeRecipeStore::with_recipes(vec![p.clone()]).pinning(p.id()))?;
    let mut editor = EditorController::new(Box::new(SequentialIds::default()));
    let text = RecipeDocument::encode(&[&p])?
        .json()
        .replace("Pour Over", "Copy");

    let import = editor.import(&format!("{text}\n"), &mut catalog)?;
    let r1 = RecipeId::try_parse("r1")?;
    assert_eq!(import.changes(), [CatalogChange::Upserted(r1.clone())]);
    assert_eq!(import.notice(), "Imported 1 recipe");
    let added = catalog.get(&r1).ok_or("not added")?;
    assert_eq!(added.name().as_str(), "Copy");
    let step_ids: Vec<_> = added.steps().iter().map(|s| s.id().as_str()).collect();
    assert_eq!(step_ids, ["s1", "s2"]);
    assert_eq!(catalog.pin_of(&r1), Pin::Unpinned);
    assert_eq!(catalog.index_of(&r1), Some(1));
    assert!(editor.state().draft().is_none(), "no draft opened");
    Ok(())
}

#[test]
fn import_is_all_or_nothing_and_mints_no_ids_when_refused() -> TestResult {
    let mut catalog = catalog(FakeRecipeStore::default())?;
    let mut editor = EditorController::new(Box::new(SequentialIds::default()));
    let bad = document(&format!(
        r#"{VALID},{{"name":"B","coffee_grams":15,"water_grams":250,"steps":[{{"seconds":9999}}]}}"#
    ));
    let error = editor.import(&bad, &mut catalog).err().ok_or("imported")?;
    assert_eq!(
        error.to_string(),
        "Can't import: Recipe 2, step 1, seconds: 0–5999 seconds"
    );
    assert!(catalog.state().is_empty());

    editor.import(&document(VALID), &mut catalog)?;
    assert_eq!(
        catalog.state().first().map(|r| r.id().as_str()),
        Some("r1"),
        "the refused import used up no id"
    );
    Ok(())
}

#[test]
fn import_with_a_failed_write_adds_none() -> TestResult {
    let mut catalog = catalog(FakeRecipeStore::default().failing_writes())?;
    let mut editor = EditorController::new(Box::new(SequentialIds::default()));
    let error = editor
        .import(&document(&format!("{VALID},{VALID}")), &mut catalog)
        .err()
        .ok_or("imported")?;
    assert!(matches!(error, ImportError::Store(_)));
    assert_eq!(
        error.to_string(),
        "Failed to import recipes: Storage failed: fake write failure"
    );
    assert!(catalog.state().is_empty());
    Ok(())
}
