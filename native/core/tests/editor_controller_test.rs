//! `EditorController`: the recipe editor's draft, live labels, field errors,
//! strict save and delete. Legacy reference: `lib/ui/recipe_edit/`
//! (controller and page) and `test/ui/recipe_edit_controller_test.dart`,
//! `test/ui/recipe_edit_flow_test.dart`.

use coffee_timer_core::catalog::{CatalogChange, RecipeCatalog};
use coffee_timer_core::editor::{
    DeleteError, Draft, DraftMode, EditError, EditorController, EditorState, EditorStepRow,
    SaveError, StepPreset,
};
use coffee_timer_core::fakes::{FakeRecipeStore, SequentialIds};
use coffee_timer_core::model::{
    CoffeeDose, ModelError, Recipe, RecipeId, RecipeName, RecipeStep, StepDuration, StepId,
    WaterAmount,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const ENTER_SECONDS: &str = "Enter seconds";
const OUT_OF_RANGE: &str = "0–5999 seconds";

fn rid(raw: &str) -> Result<RecipeId, ModelError> {
    RecipeId::try_parse(raw)
}

fn sid(raw: &str) -> Result<StepId, ModelError> {
    StepId::try_parse(raw)
}

/// Legacy test fixture: `recipe-1` "V60", "clean", one step `step-1` Bloom 30s.
fn v60() -> Result<Recipe, ModelError> {
    Ok(Recipe::new(
        rid("recipe-1")?,
        RecipeName::try_parse("V60")?,
        "clean".to_owned(),
        CoffeeDose::try_parse(15)?,
        WaterAmount::try_parse(250)?,
        vec![RecipeStep::new(
            sid("step-1")?,
            "Bloom".to_owned(),
            StepDuration::try_parse(30)?,
            None,
        )],
    ))
}

fn espresso() -> Result<Recipe, ModelError> {
    Ok(Recipe::blank(
        rid("espresso")?,
        RecipeName::try_parse("Espresso")?,
        CoffeeDose::try_parse(18)?,
        WaterAmount::try_parse(36)?,
    ))
}

fn catalog_of(store: FakeRecipeStore) -> Result<RecipeCatalog, Box<dyn std::error::Error>> {
    let mut catalog = RecipeCatalog::new(Box::new(store));
    catalog.load()?;
    Ok(catalog)
}

fn catalog(recipes: Vec<Recipe>) -> Result<RecipeCatalog, Box<dyn std::error::Error>> {
    catalog_of(FakeRecipeStore::with_recipes(recipes))
}

fn editor() -> EditorController {
    EditorController::new(Box::new(SequentialIds::default()))
}

/// An editor with a new draft open (recipe `r1`, row `s1`).
fn new_draft() -> EditorController {
    let mut editor = editor();
    editor.new_draft();
    editor
}

/// An editor with `id` from `catalog` open.
fn editing(
    id: &str,
    catalog: &RecipeCatalog,
) -> Result<EditorController, Box<dyn std::error::Error>> {
    let mut editor = editor();
    editor.edit(&rid(id)?, catalog)?;
    Ok(editor)
}

fn draft(editor: &EditorController) -> Result<&Draft, &'static str> {
    editor.state().draft().ok_or("no draft open")
}

fn rows(editor: &EditorController) -> Result<Vec<EditorStepRow>, &'static str> {
    Ok(draft(editor)?.step_rows())
}

fn row(editor: &EditorController, id: &str) -> Result<EditorStepRow, Box<dyn std::error::Error>> {
    let wanted = sid(id)?;
    Ok(rows(editor)?
        .into_iter()
        .find(|r| r.id == wanted)
        .ok_or("no such row")?)
}

fn row_ids(editor: &EditorController) -> Result<Vec<String>, &'static str> {
    Ok(rows(editor)?
        .iter()
        .map(|r| r.id.as_str().to_owned())
        .collect())
}

/// `(id, name, duration seconds)` of each saved step.
fn saved_steps(recipe: &Recipe) -> Vec<(&str, &str, u32)> {
    recipe
        .steps()
        .iter()
        .map(|s| (s.id().as_str(), s.name(), s.duration().seconds()))
        .collect()
}

fn is_closed(editor: &EditorController) -> bool {
    editor.state() == &EditorState::Closed
}

// The plan's example

#[test]
fn plan_example() -> TestResult {
    let mut catalog = RecipeCatalog::new(Box::new(FakeRecipeStore::default()));
    let mut editor = EditorController::new(Box::new(SequentialIds::default()));
    editor.new_draft();
    editor.set_name("V60")?;
    let change = editor.save(&mut catalog)?;
    assert_eq!(catalog.state().len(), 1);
    assert_eq!(change, CatalogChange::Upserted(RecipeId::try_parse("r1")?));
    Ok(())
}

// new_draft / edit

#[test]
fn starts_closed() {
    assert!(is_closed(&editor()));
}

#[test]
fn new_draft_starts_with_one_blank_step_and_no_name() -> TestResult {
    // Legacy: "create starts with one blank step and no name".
    let editor = new_draft();
    let draft = draft(&editor)?;
    assert_eq!(draft.id(), &rid("r1")?);
    assert_eq!(draft.mode(), DraftMode::New);
    assert_eq!(draft.title(), "New Recipe");
    assert!(!draft.can_delete());
    assert_eq!(draft.name(), "");
    assert_eq!(draft.name_error(), None, "errors hidden before a save");
    assert_eq!(draft.description(), "");
    assert_eq!(draft.total_label(), "0:00");
    assert_eq!(
        rows(&editor)?,
        [EditorStepRow {
            id: sid("s1")?,
            name_label: "Step 1 name".to_owned(),
            name: String::new(),
            duration_text: "0".to_owned(),
            duration_label: "00:00".to_owned(),
            duration_error: None,
            water_text: String::new(),
            water_error: None,
        }]
    );
    Ok(())
}

#[test]
fn edit_prefills_from_the_stored_recipe() -> TestResult {
    // Legacy: "edit prefills from the initial recipe".
    let catalog = catalog(vec![espresso()?, v60()?])?;
    let editor = editing("recipe-1", &catalog)?;
    let draft = draft(&editor)?;
    assert_eq!(draft.id(), &rid("recipe-1")?);
    assert_eq!(draft.mode(), DraftMode::Existing);
    assert_eq!(draft.title(), "Edit Recipe");
    assert!(draft.can_delete());
    assert_eq!(draft.name(), "V60");
    assert_eq!(draft.description(), "clean");
    assert_eq!(draft.total_label(), "0:30");
    assert_eq!(
        rows(&editor)?,
        [EditorStepRow {
            id: sid("step-1")?,
            name_label: "Step 1 name".to_owned(),
            name: "Bloom".to_owned(),
            duration_text: "30".to_owned(),
            duration_label: "00:30".to_owned(),
            duration_error: None,
            water_text: String::new(),
            water_error: None,
        }]
    );
    Ok(())
}

#[test]
fn edit_of_an_unknown_id_is_an_error_and_changes_nothing() -> TestResult {
    let catalog = catalog(vec![v60()?])?;
    let mut editor = editor();
    assert_eq!(
        editor.edit(&rid("missing")?, &catalog),
        Err(EditError::UnknownRecipe(rid("missing")?))
    );
    assert!(is_closed(&editor));

    editor.new_draft();
    editor.set_name("Kept")?;
    assert!(editor.edit(&rid("missing")?, &catalog).is_err());
    assert_eq!(draft(&editor)?.name(), "Kept", "open draft untouched");
    Ok(())
}

#[test]
fn edit_of_a_recipe_without_steps_shows_one_blank_row() -> TestResult {
    let catalog = catalog(vec![espresso()?])?;
    let editor = editing("espresso", &catalog)?;
    assert_eq!(row_ids(&editor)?, ["s1"]);
    assert_eq!(row(&editor, "s1")?.duration_text, "0");
    Ok(())
}

#[test]
fn new_draft_replaces_an_open_draft() -> TestResult {
    let mut editor = new_draft();
    editor.set_name("Old")?;
    editor.new_draft();
    let draft = draft(&editor)?;
    assert_eq!(draft.id(), &rid("r2")?);
    assert_eq!(draft.name(), "");
    Ok(())
}

#[test]
fn actions_without_a_draft_are_errors() -> TestResult {
    let mut catalog = catalog(vec![v60()?])?;
    let mut editor = editor();
    let s1 = sid("s1")?;
    assert_eq!(editor.set_name("x"), Err(EditError::NoDraft));
    assert_eq!(editor.set_description("x"), Err(EditError::NoDraft));
    assert_eq!(editor.set_coffee("18"), Err(EditError::NoDraft));
    assert_eq!(editor.set_water("36"), Err(EditError::NoDraft));
    assert_eq!(editor.set_step_name(&s1, "x"), Err(EditError::NoDraft));
    assert_eq!(editor.set_step_duration(&s1, "1"), Err(EditError::NoDraft));
    assert_eq!(editor.add_step(StepPreset::Wait), Err(EditError::NoDraft));
    assert_eq!(editor.remove_step(&s1), Err(EditError::NoDraft));
    assert_eq!(editor.save(&mut catalog), Err(SaveError::NoDraft));
    assert_eq!(editor.delete(&mut catalog), Err(DeleteError::NoDraft));
    assert_eq!(catalog.state(), [v60()?]);
    Ok(())
}

#[test]
fn close_drops_the_draft_without_saving() -> TestResult {
    let mut catalog = catalog(vec![])?;
    let mut editor = new_draft();
    editor.set_name("Unsaved")?;
    editor.close();
    assert!(is_closed(&editor));
    assert_eq!(editor.save(&mut catalog), Err(SaveError::NoDraft));
    assert!(catalog.state().is_empty());
    Ok(())
}

// Name

#[test]
fn blank_name_blocks_save_and_shows_the_field_error() -> TestResult {
    // Legacy: "blank name is rejected and nothing is written" and the flow
    // test "a blank name blocks save and shows the field error".
    let mut catalog = catalog(vec![])?;
    let mut editor = new_draft();
    editor.set_name("   ")?;
    editor.set_step_duration(&sid("s1")?, "30")?;
    assert_eq!(draft(&editor)?.name_error(), None, "hidden before save");

    assert_eq!(editor.save(&mut catalog), Err(SaveError::Invalid));

    assert!(catalog.state().is_empty());
    let draft = draft(&editor)?;
    assert_eq!(draft.title(), "New Recipe", "still on the editor");
    assert_eq!(draft.name(), "   ", "keeps the text as typed");
    assert_eq!(draft.name_error().as_deref(), Some("Enter a recipe name"));
    Ok(())
}

#[test]
fn errors_follow_the_text_after_a_save_attempt() -> TestResult {
    let mut catalog = catalog(vec![])?;
    let mut editor = new_draft();
    let s1 = sid("s1")?;
    editor.set_step_duration(&s1, "abc")?;
    assert_eq!(
        row(&editor, "s1")?.duration_error,
        None,
        "hidden before save"
    );

    assert_eq!(editor.save(&mut catalog), Err(SaveError::Invalid));
    assert_eq!(
        draft(&editor)?.name_error().as_deref(),
        Some("Enter a recipe name")
    );
    assert_eq!(
        row(&editor, "s1")?.duration_error.as_deref(),
        Some(ENTER_SECONDS)
    );

    editor.set_name("Fixed")?;
    editor.set_step_duration(&s1, "7000")?;
    assert_eq!(draft(&editor)?.name_error(), None);
    assert_eq!(
        row(&editor, "s1")?.duration_error.as_deref(),
        Some(OUT_OF_RANGE)
    );

    editor.set_step_duration(&s1, "70")?;
    assert_eq!(row(&editor, "s1")?.duration_error, None);
    editor.save(&mut catalog)?;
    assert_eq!(catalog.state().len(), 1);
    Ok(())
}

#[test]
fn name_and_description_are_trimmed_on_save() -> TestResult {
    // Legacy: "assembles a recipe" (`'  strong  '` → `'strong'`).
    let mut catalog = catalog(vec![])?;
    let mut editor = new_draft();
    editor.set_name("  Aeropress ")?;
    editor.set_description("  strong  ")?;
    editor.save(&mut catalog)?;
    let saved = catalog.get(&rid("r1")?).ok_or("not saved")?;
    assert_eq!(saved.name().as_str(), "Aeropress");
    assert_eq!(saved.description(), "strong");
    Ok(())
}

// Coffee and water

/// `(coffee, water)` grams of the saved recipe `id`.
fn saved_dose(catalog: &RecipeCatalog, id: &str) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    let saved = catalog.get(&rid(id)?).ok_or("not saved")?;
    Ok((saved.coffee().grams(), saved.water().grams()))
}

#[test]
fn a_new_draft_starts_with_a_one_cup_dose() -> TestResult {
    let mut catalog = catalog(vec![])?;
    let mut editor = new_draft();
    let draft = draft(&editor)?;
    assert_eq!((draft.coffee(), draft.water()), ("15", "250"));
    editor.set_name("V60")?;
    editor.save(&mut catalog)?;
    assert_eq!(saved_dose(&catalog, "r1")?, (15, 250));
    Ok(())
}

#[test]
fn edit_prefills_the_stored_dose_and_saves_the_typed_one() -> TestResult {
    let mut catalog = catalog(vec![espresso()?])?;
    let mut editor = editing("espresso", &catalog)?;
    let draft = draft(&editor)?;
    assert_eq!((draft.coffee(), draft.water()), ("18", "36"));
    editor.set_name("Lungo")?;
    editor.set_coffee(" 18 ")?;
    editor.set_water("54")?;
    editor.save(&mut catalog)?;
    assert_eq!(saved_dose(&catalog, "espresso")?, (18, 54));
    Ok(())
}

#[test]
fn dose_errors_show_from_the_first_save_and_follow_the_text() -> TestResult {
    let mut catalog = catalog(vec![])?;
    let mut editor = new_draft();
    editor.set_name("V60")?;
    editor.set_coffee("lots")?;
    editor.set_water("0")?;
    assert_eq!(draft(&editor)?.coffee_error(), None, "hidden before save");
    assert_eq!(draft(&editor)?.water_error(), None, "hidden before save");

    assert_eq!(editor.save(&mut catalog), Err(SaveError::Invalid));
    assert!(catalog.state().is_empty());
    assert_eq!(draft(&editor)?.coffee(), "lots", "keeps the text as typed");
    assert_eq!(
        draft(&editor)?.coffee_error().as_deref(),
        Some("Enter grams")
    );
    assert_eq!(draft(&editor)?.water_error().as_deref(), Some("1–2000 g"));

    editor.set_coffee("201")?;
    editor.set_water("2000")?;
    assert_eq!(draft(&editor)?.coffee_error().as_deref(), Some("1–200 g"));
    assert_eq!(draft(&editor)?.water_error(), None);

    editor.set_coffee("200")?;
    editor.save(&mut catalog)?;
    assert_eq!(saved_dose(&catalog, "r1")?, (200, 2000));
    Ok(())
}

// Step pours

/// The grams each step of the saved recipe `id` pours.
fn saved_pours(
    catalog: &RecipeCatalog,
    id: &str,
) -> Result<Vec<Option<u32>>, Box<dyn std::error::Error>> {
    let saved = catalog.get(&rid(id)?).ok_or("not saved")?;
    Ok(saved
        .steps()
        .iter()
        .map(|s| s.water().map(WaterAmount::grams))
        .collect())
}

#[test]
fn a_new_row_pours_nothing_until_grams_are_typed() -> TestResult {
    let mut catalog = catalog(vec![])?;
    let mut editor = new_draft();
    editor.set_name("V60")?;
    let water = editor.add_step(StepPreset::AddWater)?;
    assert_eq!(row(&editor, "s2")?.name, "Add water");
    assert_eq!(row(&editor, "s2")?.water_text, "", "left for the grams");
    editor.set_step_water(&water, " 60 ")?;
    editor.save(&mut catalog)?;
    assert_eq!(saved_pours(&catalog, "r1")?, [None, Some(60)]);
    Ok(())
}

#[test]
fn edit_prefills_the_stored_pours_and_blank_clears_one() -> TestResult {
    let poured = Recipe::new(
        rid("recipe-1")?,
        RecipeName::try_parse("V60")?,
        String::new(),
        CoffeeDose::try_parse(15)?,
        WaterAmount::try_parse(250)?,
        vec![RecipeStep::new(
            sid("step-1")?,
            "Bloom".to_owned(),
            StepDuration::try_parse(30)?,
            Some(WaterAmount::try_parse(50)?),
        )],
    );
    let mut catalog = catalog(vec![poured])?;
    let mut editor = editing("recipe-1", &catalog)?;
    assert_eq!(row(&editor, "step-1")?.water_text, "50");
    editor.set_step_water(&sid("step-1")?, "  ")?;
    editor.save(&mut catalog)?;
    assert_eq!(saved_pours(&catalog, "recipe-1")?, [None]);
    Ok(())
}

#[test]
fn pour_errors_show_from_the_first_save_and_follow_the_text() -> TestResult {
    let mut catalog = catalog(vec![])?;
    let mut editor = new_draft();
    editor.set_name("V60")?;
    editor.set_step_water(&sid("s1")?, "0")?;
    assert_eq!(row(&editor, "s1")?.water_error, None, "hidden before save");

    assert_eq!(editor.save(&mut catalog), Err(SaveError::Invalid));
    assert!(catalog.state().is_empty());
    assert_eq!(row(&editor, "s1")?.water_error.as_deref(), Some("1–2000 g"));
    editor.set_step_water(&sid("s1")?, "lots")?;
    assert_eq!(
        row(&editor, "s1")?.water_error.as_deref(),
        Some("Enter grams")
    );
    editor.set_step_water(&sid("s1")?, "2000")?;
    assert_eq!(row(&editor, "s1")?.water_error, None);
    editor.save(&mut catalog)?;
    assert_eq!(saved_pours(&catalog, "r1")?, [Some(2000)]);
    Ok(())
}

#[test]
fn set_step_water_of_an_unknown_row_is_an_error() -> TestResult {
    let mut editor = new_draft();
    assert_eq!(
        editor.set_step_water(&sid("nope")?, "50"),
        Err(EditError::UnknownStep(sid("nope")?))
    );
    Ok(())
}

#[test]
fn the_pour_hint_shows_while_step_pours_miss_the_recipes_water() -> TestResult {
    let mut editor = new_draft();
    assert_eq!(draft(&editor)?.pour_hint(), None, "no step pours");
    editor.set_step_water(&sid("s1")?, "50")?;
    assert_eq!(
        draft(&editor)?.pour_hint().as_deref(),
        Some("Step pours add up to 50 g of 250 g")
    );
    let pour = editor.add_step(StepPreset::AddWater)?;
    editor.set_step_water(&pour, "lots")?;
    assert_eq!(
        draft(&editor)?.pour_hint().as_deref(),
        Some("Step pours add up to 50 g of 250 g"),
        "an invalid pour is left out"
    );
    editor.set_step_water(&pour, "200")?;
    assert_eq!(draft(&editor)?.pour_hint(), None, "they add up");
    editor.set_water("300")?;
    assert_eq!(
        draft(&editor)?.pour_hint().as_deref(),
        Some("Step pours add up to 250 g of 300 g")
    );
    editor.set_water("")?;
    assert_eq!(draft(&editor)?.pour_hint(), None, "no water to compare");
    Ok(())
}

// Duration bounds

#[test]
fn duration_text_is_validated_strictly_on_save() -> TestResult {
    let cases: [(&str, Result<u32, &str>); 10] = [
        ("-1", Err(OUT_OF_RANGE)),
        ("0", Ok(0)),
        ("5999", Ok(5999)),
        ("6000", Err(OUT_OF_RANGE)),
        ("10000", Err(OUT_OF_RANGE)),
        ("abc", Err(ENTER_SECONDS)),
        ("", Err(ENTER_SECONDS)),
        ("   ", Err(ENTER_SECONDS)),
        (" 45 ", Ok(45)),
        ("1.5", Err(ENTER_SECONDS)),
    ];
    for (text, expected) in cases {
        let mut catalog = catalog(vec![])?;
        let mut editor = new_draft();
        editor.set_name("X")?;
        editor.set_step_duration(&sid("s1")?, text)?;
        let result = editor.save(&mut catalog);
        match expected {
            Ok(seconds) => {
                assert!(result.is_ok(), "{text:?}");
                let saved = catalog.get(&rid("r1")?).ok_or("not saved")?;
                assert_eq!(saved_steps(saved), [("s1", "Step 1", seconds)], "{text:?}");
            }
            Err(message) => {
                assert_eq!(result, Err(SaveError::Invalid), "{text:?}");
                assert!(catalog.state().is_empty(), "{text:?}");
                let shown = row(&editor, "s1")?;
                assert_eq!(shown.duration_text, text, "keeps the text as typed");
                assert_eq!(shown.duration_error.as_deref(), Some(message), "{text:?}");
            }
        }
    }
    Ok(())
}

// Live labels

#[test]
fn labels_follow_the_typed_durations() -> TestResult {
    // Legacy: "totalDurationLabel reflects typed, clamped durations".
    let mut editor = new_draft();
    editor.set_step_duration(&sid("s1")?, "65")?;
    assert_eq!(row(&editor, "s1")?.duration_label, "01:05");
    assert_eq!(draft(&editor)?.total_label(), "1:05");

    let added = editor.add_step(StepPreset::Blank)?;
    assert_eq!(added, sid("s2")?);
    editor.set_step_duration(&added, "600")?;
    assert_eq!(row(&editor, "s2")?.duration_label, "10:00");
    assert_eq!(draft(&editor)?.total_label(), "11:05");
    Ok(())
}

#[test]
fn labels_read_invalid_text_leniently() -> TestResult {
    // Legacy: `StepDuration.clamped(int.tryParse(text.trim()) ?? 0)`.
    let mut editor = new_draft();
    let s1 = sid("s1")?;
    for (text, label, total) in [
        ("abc", "00:00", "0:00"),
        ("", "00:00", "0:00"),
        ("-5", "00:00", "0:00"),
        ("10000", "99:59", "99:59"),
        (" 90 ", "01:30", "1:30"),
    ] {
        editor.set_step_duration(&s1, text)?;
        assert_eq!(row(&editor, "s1")?.duration_label, label, "{text:?}");
        assert_eq!(draft(&editor)?.total_label(), total, "{text:?}");
    }
    Ok(())
}

#[test]
fn total_label_is_capped_like_a_saved_recipe() -> TestResult {
    let mut editor = new_draft();
    editor.set_step_duration(&sid("s1")?, "5999")?;
    editor.add_step(StepPreset::Blank)?;
    editor.set_step_duration(&sid("s2")?, "1")?;
    assert_eq!(draft(&editor)?.total_label(), "99:99");
    Ok(())
}

#[test]
fn step_name_labels_follow_row_positions() -> TestResult {
    let mut editor = new_draft();
    editor.add_step(StepPreset::Blank)?;
    editor.add_step(StepPreset::Blank)?;
    editor.remove_step(&sid("s1")?)?;
    let labels: Vec<_> = rows(&editor)?
        .into_iter()
        .map(|r| (r.id.as_str().to_owned(), r.name_label))
        .collect();
    assert_eq!(
        labels,
        [
            ("s2".to_owned(), "Step 1 name".to_owned()),
            ("s3".to_owned(), "Step 2 name".to_owned()),
        ]
    );
    Ok(())
}

// Rows

#[test]
fn removing_the_last_row_leaves_one_blank_row() -> TestResult {
    // Legacy: "removeStepAt keeps at least one row".
    let mut editor = new_draft();
    editor.set_step_name(&sid("s1")?, "Bloom")?;
    editor.set_step_duration(&sid("s1")?, "30")?;
    editor.remove_step(&sid("s1")?)?;
    assert_eq!(
        rows(&editor)?,
        [EditorStepRow {
            id: sid("s2")?,
            name_label: "Step 1 name".to_owned(),
            name: String::new(),
            duration_text: "0".to_owned(),
            duration_label: "00:00".to_owned(),
            duration_error: None,
            water_text: String::new(),
            water_error: None,
        }]
    );
    Ok(())
}

#[test]
fn remove_drops_only_the_addressed_row() -> TestResult {
    let mut editor = new_draft();
    editor.add_step(StepPreset::AddWater)?;
    editor.add_step(StepPreset::Wait)?;
    editor.remove_step(&sid("s2")?)?;
    assert_eq!(row_ids(&editor)?, ["s1", "s3"]);
    Ok(())
}

#[test]
fn unknown_row_is_an_error_and_changes_nothing() -> TestResult {
    let mut editor = new_draft();
    let before = editor.state().clone();
    let missing = sid("missing")?;
    let unknown = Err(EditError::UnknownStep(missing.clone()));
    assert_eq!(editor.remove_step(&missing), unknown);
    assert_eq!(editor.set_step_name(&missing, "x"), unknown);
    assert_eq!(editor.set_step_duration(&missing, "1"), unknown);
    assert_eq!(editor.state(), &before);
    Ok(())
}

#[test]
fn edits_address_a_row_not_a_position() -> TestResult {
    let mut editor = new_draft();
    editor.add_step(StepPreset::Blank)?;
    editor.remove_step(&sid("s1")?)?;
    editor.set_step_duration(&sid("s2")?, "12")?;
    assert_eq!(row(&editor, "s2")?.duration_text, "12");
    Ok(())
}

// Moving rows

/// A new draft with rows `s1`, `s2`, `s3`, each named after its id.
fn three_rows() -> Result<EditorController, Box<dyn std::error::Error>> {
    let mut editor = new_draft();
    editor.add_step(StepPreset::Blank)?;
    editor.add_step(StepPreset::Blank)?;
    for id in ["s1", "s2", "s3"] {
        editor.set_step_name(&sid(id)?, id)?;
    }
    Ok(editor)
}

#[test]
fn move_up_puts_the_row_at_the_position() -> TestResult {
    let mut editor = three_rows()?;
    editor.move_step(&sid("s3")?, 0)?;
    assert_eq!(row_ids(&editor)?, ["s3", "s1", "s2"]);
    Ok(())
}

#[test]
fn move_down_counts_the_position_after_the_move() -> TestResult {
    let mut editor = three_rows()?;
    editor.move_step(&sid("s1")?, 1)?;
    assert_eq!(row_ids(&editor)?, ["s2", "s1", "s3"]);
    editor.move_step(&sid("s2")?, 2)?;
    assert_eq!(row_ids(&editor)?, ["s1", "s3", "s2"]);
    Ok(())
}

#[test]
fn move_to_its_own_position_changes_nothing() -> TestResult {
    let mut editor = three_rows()?;
    let before = editor.state().clone();
    editor.move_step(&sid("s2")?, 1)?;
    assert_eq!(editor.state(), &before);
    Ok(())
}

#[test]
fn move_past_the_last_row_is_an_error_and_changes_nothing() -> TestResult {
    let mut editor = three_rows()?;
    let before = editor.state().clone();
    assert_eq!(
        editor.move_step(&sid("s1")?, 3),
        Err(EditError::StepPositionOutOfRange { to: 3, rows: 3 })
    );
    assert_eq!(editor.state(), &before);
    Ok(())
}

#[test]
fn move_of_an_unknown_row_or_without_a_draft_is_an_error() -> TestResult {
    let mut editor = three_rows()?;
    let before = editor.state().clone();
    let missing = sid("missing")?;
    assert_eq!(
        editor.move_step(&missing, 0),
        Err(EditError::UnknownStep(missing.clone()))
    );
    assert_eq!(editor.state(), &before);
    assert_eq!(
        self::editor().move_step(&missing, 0),
        Err(EditError::NoDraft)
    );
    Ok(())
}

#[test]
fn a_moved_row_keeps_its_text_errors_and_pour_and_labels_follow_positions() -> TestResult {
    let mut catalog = catalog(vec![])?;
    let mut editor = three_rows()?;
    editor.set_step_duration(&sid("s3")?, "abc")?;
    editor.set_step_water(&sid("s3")?, "60")?;
    editor.set_step_water(&sid("s1")?, "0")?;
    assert_eq!(editor.save(&mut catalog), Err(SaveError::Invalid));

    editor.move_step(&sid("s3")?, 0)?;

    let moved = row(&editor, "s3")?;
    assert_eq!(moved.name_label, "Step 1 name");
    assert_eq!(moved.name, "s3");
    assert_eq!(moved.duration_text, "abc");
    assert_eq!(moved.duration_error.as_deref(), Some(ENTER_SECONDS));
    assert_eq!(moved.water_text, "60");
    assert_eq!(moved.water_error, None);
    let pushed = row(&editor, "s1")?;
    assert_eq!(pushed.name_label, "Step 2 name");
    assert_eq!(pushed.water_text, "0");
    assert_eq!(pushed.water_error.as_deref(), Some("1–2000 g"));
    assert_eq!(row(&editor, "s2")?.duration_error, None);
    Ok(())
}

#[test]
fn save_keeps_the_moved_order() -> TestResult {
    let mut catalog = catalog(vec![v60()?])?;
    let mut editor = editing("recipe-1", &catalog)?;
    editor.add_step(StepPreset::AddWater)?;
    editor.move_step(&sid("s1")?, 0)?;
    editor.save(&mut catalog)?;
    let saved = catalog.get(&rid("recipe-1")?).ok_or("missing")?;
    assert_eq!(
        saved_steps(saved),
        [("s1", "Add water", 0), ("step-1", "Bloom", 30)]
    );
    Ok(())
}

// Presets

#[test]
fn presets_in_legacy_button_order() {
    let labels: Vec<_> = StepPreset::ALL.iter().map(|p| p.button_label()).collect();
    assert_eq!(labels, ["Add coffee", "Add water", "Wait", "Add Step"]);
}

#[test]
fn each_preset_adds_a_named_zero_second_row() -> TestResult {
    let mut editor = new_draft();
    for preset in StepPreset::ALL {
        editor.add_step(preset)?;
    }
    let shown: Vec<_> = rows(&editor)?
        .into_iter()
        .map(|r| (r.id.as_str().to_owned(), r.name, r.duration_text))
        .collect();
    let expected: Vec<_> = [
        ("s1", ""),
        ("s2", "Add coffee"),
        ("s3", "Add water"),
        ("s4", "Wait"),
        ("s5", ""),
    ]
    .into_iter()
    .map(|(id, name)| (id.to_owned(), name.to_owned(), "0".to_owned()))
    .collect();
    assert_eq!(shown, expected);
    Ok(())
}

#[test]
fn preset_rows_save_with_their_names_and_blank_rows_as_step_n() -> TestResult {
    let mut catalog = catalog(vec![])?;
    let mut editor = new_draft();
    editor.set_name("Pour over")?;
    for preset in StepPreset::ALL {
        editor.add_step(preset)?;
    }
    editor.set_step_name(&sid("s1")?, "  Bloom  ")?;
    editor.save(&mut catalog)?;
    let saved = catalog.get(&rid("r1")?).ok_or("not saved")?;
    assert_eq!(
        saved_steps(saved),
        [
            ("s1", "Bloom", 0),
            ("s2", "Add coffee", 0),
            ("s3", "Add water", 0),
            ("s4", "Wait", 0),
            ("s5", "Step 5", 0),
        ]
    );
    Ok(())
}

// Save

#[test]
fn save_of_a_new_recipe_appends_it_and_closes() -> TestResult {
    // Legacy: "assembles a recipe: minted ids, default step name".
    let mut catalog = catalog(vec![v60()?])?;
    let mut editor = new_draft();
    editor.set_name("Chemex")?;
    editor.set_step_name(&sid("s1")?, "Bloom")?;
    editor.set_step_duration(&sid("s1")?, "90")?;
    editor.add_step(StepPreset::AddWater)?;
    editor.set_step_duration(&sid("s2")?, "120")?;

    let change = editor.save(&mut catalog)?;

    assert_eq!(change, CatalogChange::Upserted(rid("r1")?));
    assert!(is_closed(&editor));
    let ids: Vec<_> = catalog.state().iter().map(|r| r.id().as_str()).collect();
    assert_eq!(ids, ["recipe-1", "r1"]);
    let saved = catalog.get(&rid("r1")?).ok_or("not saved")?;
    assert_eq!(saved.name().as_str(), "Chemex");
    assert_eq!(
        saved_steps(saved),
        [("s1", "Bloom", 90), ("s2", "Add water", 120)]
    );
    Ok(())
}

#[test]
fn save_of_an_edited_recipe_keeps_its_id_step_ids_and_position() -> TestResult {
    // Legacy: "save preserves the recipe id and existing step ids" and "a
    // newly added step gets a minted id, existing one is kept".
    let mut catalog = catalog(vec![v60()?, espresso()?])?;
    let mut editor = editing("recipe-1", &catalog)?;
    editor.set_name("V60 v2")?;
    editor.set_step_duration(&sid("step-1")?, "45")?;
    editor.add_step(StepPreset::Blank)?;
    editor.set_step_name(&sid("s1")?, "Pour")?;

    let change = editor.save(&mut catalog)?;

    assert_eq!(change, CatalogChange::Upserted(rid("recipe-1")?));
    assert!(is_closed(&editor));
    let ids: Vec<_> = catalog.state().iter().map(|r| r.id().as_str()).collect();
    assert_eq!(ids, ["recipe-1", "espresso"], "replaced in place");
    let saved = catalog.get(&rid("recipe-1")?).ok_or("missing")?;
    assert_eq!(saved.name().as_str(), "V60 v2");
    assert_eq!(saved.description(), "clean");
    assert_eq!(
        saved_steps(saved),
        [("step-1", "Bloom", 45), ("s1", "Pour", 0)]
    );
    Ok(())
}

#[test]
fn save_keeps_row_order_after_removals_and_additions() -> TestResult {
    let mut catalog = catalog(vec![v60()?])?;
    let mut editor = editing("recipe-1", &catalog)?;
    editor.add_step(StepPreset::AddCoffee)?;
    editor.add_step(StepPreset::Wait)?;
    editor.remove_step(&sid("step-1")?)?;
    editor.save(&mut catalog)?;
    let saved = catalog.get(&rid("recipe-1")?).ok_or("missing")?;
    assert_eq!(
        saved_steps(saved),
        [("s1", "Add coffee", 0), ("s2", "Wait", 0)]
    );
    Ok(())
}

#[test]
fn save_with_an_invalid_field_leaves_the_catalog_untouched() -> TestResult {
    let mut catalog = catalog(vec![v60()?])?;
    let mut editor = editing("recipe-1", &catalog)?;
    editor.set_name("Renamed")?;
    editor.set_step_duration(&sid("step-1")?, "6000")?;

    assert_eq!(editor.save(&mut catalog), Err(SaveError::Invalid));

    assert_eq!(catalog.state(), [v60()?]);
    let shown = row(&editor, "step-1")?;
    assert_eq!(shown.duration_text, "6000");
    assert_eq!(shown.duration_error.as_deref(), Some(OUT_OF_RANGE));
    assert_eq!(draft(&editor)?.name(), "Renamed", "draft kept");
    Ok(())
}

#[test]
fn store_failure_on_save_surfaces_and_leaves_everything_unchanged() -> TestResult {
    // Legacy: "a write failure surfaces as an error, not an exception".
    let mut catalog = catalog_of(FakeRecipeStore::with_recipes(vec![v60()?]).failing_writes())?;
    let mut editor = editing("recipe-1", &catalog)?;
    editor.set_name("Renamed")?;

    let error = editor.save(&mut catalog).err().ok_or("save succeeded")?;

    assert!(matches!(error, SaveError::Store(_)));
    assert!(error.to_string().starts_with("Failed to save recipe: "));
    assert_eq!(catalog.state(), [v60()?]);
    assert_eq!(draft(&editor)?.name(), "Renamed", "draft kept for a retry");
    Ok(())
}

#[test]
fn a_retried_save_of_a_new_recipe_keeps_its_id() -> TestResult {
    let mut catalog = catalog_of(FakeRecipeStore::default().failing_writes())?;
    let mut editor = new_draft();
    editor.set_name("X")?;
    assert!(editor.save(&mut catalog).is_err());
    assert!(editor.save(&mut catalog).is_err());
    assert_eq!(draft(&editor)?.id(), &rid("r1")?);
    assert!(catalog.state().is_empty());
    Ok(())
}

// Delete

#[test]
fn delete_removes_the_edited_recipe_and_closes() -> TestResult {
    // Legacy: "edit mode deletes the initial recipe".
    let mut catalog = catalog(vec![espresso()?, v60()?])?;
    let mut editor = editing("recipe-1", &catalog)?;

    let change = editor.delete(&mut catalog)?;

    assert_eq!(change, CatalogChange::Removed(rid("recipe-1")?));
    assert!(is_closed(&editor));
    assert_eq!(catalog.state(), [espresso()?]);
    Ok(())
}

#[test]
fn an_editor_delete_can_be_undone_through_the_catalog() -> TestResult {
    let poured = v60()?.with_step_added(RecipeStep::new(
        sid("step-2")?,
        "Pour".to_owned(),
        StepDuration::try_parse(60)?,
        Some(WaterAmount::try_parse(150)?),
    ));
    let mut catalog = catalog(vec![espresso()?, poured.clone(), v60_named("c", "Aero")?])?;
    let mut editor = editing("recipe-1", &catalog)?;
    editor.delete(&mut catalog)?;

    let change = catalog.restore(&rid("recipe-1")?)?;

    assert_eq!(change, CatalogChange::Upserted(rid("recipe-1")?));
    assert_eq!(
        catalog.state(),
        [espresso()?, poured, v60_named("c", "Aero")?],
        "the whole recipe, back in its place"
    );
    Ok(())
}

fn v60_named(id: &str, name: &str) -> Result<Recipe, ModelError> {
    Ok(Recipe::blank(
        rid(id)?,
        RecipeName::try_parse(name)?,
        CoffeeDose::try_parse(15)?,
        WaterAmount::try_parse(250)?,
    ))
}

#[test]
fn delete_of_a_new_recipe_has_nothing_to_delete() -> TestResult {
    // Legacy: "create mode has nothing to delete".
    let mut catalog = catalog(vec![v60()?])?;
    let mut editor = new_draft();
    assert_eq!(editor.delete(&mut catalog), Err(DeleteError::NotSaved));
    assert_eq!(catalog.state(), [v60()?]);
    assert!(!is_closed(&editor));
    Ok(())
}

#[test]
fn store_failure_on_delete_surfaces_and_leaves_everything_unchanged() -> TestResult {
    let mut catalog = catalog_of(FakeRecipeStore::with_recipes(vec![v60()?]).failing_writes())?;
    let mut editor = editing("recipe-1", &catalog)?;

    let error = editor
        .delete(&mut catalog)
        .err()
        .ok_or("delete succeeded")?;

    assert!(matches!(error, DeleteError::Store(_)));
    assert!(error.to_string().starts_with("Failed to delete recipe: "));
    assert_eq!(catalog.state(), [v60()?]);
    assert_eq!(draft(&editor)?.id(), &rid("recipe-1")?, "draft kept");
    Ok(())
}

// Notice: the store failure a snackbar shows

#[test]
fn a_draft_opens_with_no_notice() -> TestResult {
    let catalog = catalog(vec![v60()?])?;
    assert_eq!(draft(&new_draft())?.notice(), None);
    assert_eq!(draft(&editing("recipe-1", &catalog)?)?.notice(), None);
    Ok(())
}

#[test]
fn store_failure_on_save_is_the_notice() -> TestResult {
    // Legacy snackbar: "Failed to save recipe: $e".
    let mut catalog = catalog_of(FakeRecipeStore::with_recipes(vec![v60()?]).failing_writes())?;
    let mut editor = editing("recipe-1", &catalog)?;

    let error = editor.save(&mut catalog).err().ok_or("save succeeded")?;

    let expected = error.to_string();
    assert_eq!(
        expected,
        "Failed to save recipe: Storage failed: fake write failure"
    );
    assert_eq!(draft(&editor)?.notice(), Some(expected.as_str()));
    Ok(())
}

#[test]
fn store_failure_on_delete_is_the_notice() -> TestResult {
    // Legacy snackbar: "Failed to delete recipe: $e".
    let mut catalog = catalog_of(FakeRecipeStore::with_recipes(vec![v60()?]).failing_writes())?;
    let mut editor = editing("recipe-1", &catalog)?;

    let error = editor
        .delete(&mut catalog)
        .err()
        .ok_or("delete succeeded")?;

    let expected = error.to_string();
    assert!(expected.starts_with("Failed to delete recipe: "));
    assert_eq!(draft(&editor)?.notice(), Some(expected.as_str()));
    Ok(())
}

#[test]
fn invalid_fields_leave_no_notice() -> TestResult {
    // Legacy showed no snackbar for an invalid form: the field errors show.
    let mut catalog = catalog(vec![])?;
    let mut editor = new_draft();

    assert_eq!(editor.save(&mut catalog), Err(SaveError::Invalid));

    assert_eq!(draft(&editor)?.notice(), None);
    assert!(draft(&editor)?.name_error().is_some());
    Ok(())
}

#[test]
fn the_next_action_clears_the_notice() -> TestResult {
    let mut catalog = catalog_of(FakeRecipeStore::with_recipes(vec![v60()?]).failing_writes())?;
    let mut editor = editing("recipe-1", &catalog)?;
    assert!(editor.save(&mut catalog).is_err());

    editor.set_description("x")?;

    assert_eq!(draft(&editor)?.notice(), None);
    Ok(())
}

#[test]
fn a_rejected_action_also_clears_the_notice() -> TestResult {
    let mut catalog = catalog_of(FakeRecipeStore::with_recipes(vec![v60()?]).failing_writes())?;
    let mut editor = editing("recipe-1", &catalog)?;
    assert!(editor.save(&mut catalog).is_err());

    let stale = sid("gone")?;
    assert_eq!(
        editor.set_step_name(&stale, "x"),
        Err(EditError::UnknownStep(stale))
    );

    assert_eq!(draft(&editor)?.notice(), None);
    Ok(())
}

#[test]
fn a_repeated_store_failure_sets_the_notice_again() -> TestResult {
    let mut catalog = catalog_of(FakeRecipeStore::with_recipes(vec![v60()?]).failing_writes())?;
    let mut editor = editing("recipe-1", &catalog)?;
    assert!(editor.save(&mut catalog).is_err());
    assert!(editor.delete(&mut catalog).is_err());

    let notice = draft(&editor)?.notice().ok_or("no notice")?;
    assert!(notice.starts_with("Failed to delete recipe: "));
    Ok(())
}

// Legacy flow test: create, edit, then delete

#[test]
fn create_edit_then_delete_a_recipe() -> TestResult {
    let mut catalog = catalog(vec![])?;
    let mut editor = editor();

    editor.new_draft();
    editor.set_name("Chemex")?;
    editor.set_step_name(&sid("s1")?, "Bloom")?;
    editor.set_step_duration(&sid("s1")?, "90")?;
    let created = editor.save(&mut catalog)?;
    let CatalogChange::Upserted(id) = created else {
        return Err("expected Upserted".into());
    };
    let saved = catalog.get(&id).ok_or("not created")?;
    assert_eq!(saved.name().as_str(), "Chemex");
    assert_eq!(saved_steps(saved), [("s1", "Bloom", 90)]);

    editor.edit(&id, &catalog)?;
    editor.set_name("Chemex v2")?;
    editor.set_step_duration(&sid("s1")?, "120")?;
    assert_eq!(
        editor.save(&mut catalog)?,
        CatalogChange::Upserted(id.clone())
    );
    let edited = catalog.state().first().ok_or("gone")?;
    assert_eq!(catalog.state().len(), 1);
    assert_eq!(edited.id(), &id, "recipe id preserved on edit");
    assert_eq!(edited.name().as_str(), "Chemex v2");
    assert_eq!(saved_steps(edited), [("s1", "Bloom", 120)], "step id kept");

    editor.edit(&id, &catalog)?;
    assert_eq!(editor.delete(&mut catalog)?, CatalogChange::Removed(id));
    assert!(catalog.state().is_empty());
    assert!(is_closed(&editor));
    Ok(())
}

// Duplicate

/// `pour-over` "Pour Over", "bright", 15 g to 250 g: Bloom 30s pouring
/// 50 g, Pour 120s pouring 200 g.
fn pour_over() -> Result<Recipe, ModelError> {
    Ok(Recipe::new(
        rid("pour-over")?,
        RecipeName::try_parse("Pour Over")?,
        "bright".to_owned(),
        CoffeeDose::try_parse(15)?,
        WaterAmount::try_parse(250)?,
        vec![
            RecipeStep::new(
                sid("bloom")?,
                "Bloom".to_owned(),
                StepDuration::try_parse(30)?,
                Some(WaterAmount::try_parse(50)?),
            ),
            RecipeStep::new(
                sid("pour")?,
                "Pour".to_owned(),
                StepDuration::try_parse(120)?,
                Some(WaterAmount::try_parse(200)?),
            ),
        ],
    ))
}

/// An editor with a copy of `id` from `catalog` open.
fn duplicating(
    id: &str,
    catalog: &RecipeCatalog,
) -> Result<EditorController, Box<dyn std::error::Error>> {
    let mut editor = editor();
    editor.duplicate(&rid(id)?, catalog)?;
    Ok(editor)
}

#[test]
fn duplicate_opens_a_new_draft_with_fresh_ids() -> TestResult {
    let catalog = catalog(vec![espresso()?, pour_over()?])?;
    let editor = duplicating("pour-over", &catalog)?;
    let draft = draft(&editor)?;
    assert_eq!(draft.id().as_str(), "r1");
    assert_eq!(draft.mode(), DraftMode::New);
    assert_eq!(draft.title(), "New Recipe");
    assert!(!draft.can_delete());
    assert_eq!(row_ids(&editor)?, ["s1", "s2"]);
    Ok(())
}

#[test]
fn duplicate_copies_every_field_and_names_it_a_copy() -> TestResult {
    let catalog = catalog(vec![pour_over()?])?;
    let editor = duplicating("pour-over", &catalog)?;
    let draft = draft(&editor)?;
    assert_eq!(draft.name(), "Pour Over copy");
    assert_eq!(draft.description(), "bright");
    assert_eq!(draft.coffee(), "15");
    assert_eq!(draft.water(), "250");
    let steps: Vec<_> = rows(&editor)?
        .into_iter()
        .map(|r| (r.name, r.duration_text, r.water_text))
        .collect();
    assert_eq!(
        steps,
        [
            ("Bloom".to_owned(), "30".to_owned(), "50".to_owned()),
            ("Pour".to_owned(), "120".to_owned(), "200".to_owned()),
        ]
    );
    assert_eq!(draft.total_label(), "2:30");
    assert_eq!(draft.notice(), None);
    Ok(())
}

#[test]
fn duplicate_of_a_long_name_keeps_the_copy_within_the_bound() -> TestResult {
    let long = "x".repeat(RecipeName::MAX_GENERATED_CHARS);
    let recipe = pour_over()?.renamed(RecipeName::try_parse(&long)?);
    let catalog = catalog(vec![recipe])?;
    let editor = duplicating("pour-over", &catalog)?;
    assert_eq!(
        draft(&editor)?.name(),
        RecipeName::try_parse(&long)?.copy_of().as_str()
    );
    assert_eq!(
        draft(&editor)?.name().chars().count(),
        RecipeName::MAX_GENERATED_CHARS
    );
    Ok(())
}

#[test]
fn duplicate_of_an_unknown_id_is_an_error_and_changes_nothing() -> TestResult {
    let catalog = catalog(vec![pour_over()?])?;
    let mut editor = editor();
    assert_eq!(
        editor.duplicate(&rid("nope")?, &catalog),
        Err(EditError::UnknownRecipe(rid("nope")?))
    );
    assert!(is_closed(&editor));
    let mut open = editing("pour-over", &catalog)?;
    let before = open.state().clone();
    assert!(open.duplicate(&rid("nope")?, &catalog).is_err());
    assert_eq!(open.state(), &before);
    Ok(())
}

#[test]
fn saving_a_duplicate_appends_it_and_leaves_the_source() -> TestResult {
    let mut catalog = catalog(vec![pour_over()?, espresso()?])?;
    let mut editor = duplicating("pour-over", &catalog)?;
    assert_eq!(
        editor.save(&mut catalog)?,
        CatalogChange::Upserted(rid("r1")?)
    );
    assert!(is_closed(&editor));
    let ids: Vec<_> = catalog.state().iter().map(|r| r.id().as_str()).collect();
    assert_eq!(ids, ["pour-over", "espresso", "r1"]);
    assert_eq!(catalog.get(&rid("pour-over")?), Some(&pour_over()?));
    let copy = catalog.get(&rid("r1")?).ok_or("copy not saved")?;
    assert_eq!(copy.name().as_str(), "Pour Over copy");
    assert_eq!(copy.description(), "bright");
    assert_eq!(copy.coffee().grams(), 15);
    assert_eq!(copy.water().grams(), 250);
    assert_eq!(
        saved_steps(copy),
        [("s1", "Bloom", 30), ("s2", "Pour", 120)]
    );
    let pours: Vec<_> = copy
        .steps()
        .iter()
        .map(|s| s.water().map(WaterAmount::grams))
        .collect();
    assert_eq!(pours, [Some(50), Some(200)]);
    Ok(())
}

#[test]
fn a_renamed_duplicate_saves_under_the_typed_name() -> TestResult {
    let mut catalog = catalog(vec![pour_over()?])?;
    let mut editor = duplicating("pour-over", &catalog)?;
    editor.set_name("Iced pour over")?;
    editor.save(&mut catalog)?;
    let copy = catalog.get(&rid("r1")?).ok_or("copy not saved")?;
    assert_eq!(copy.name().as_str(), "Iced pour over");
    Ok(())
}

#[test]
fn closing_a_duplicate_leaves_the_catalog_unchanged() -> TestResult {
    let catalog = catalog(vec![pour_over()?, espresso()?])?;
    let before = catalog.state().to_vec();
    let mut editor = duplicating("pour-over", &catalog)?;
    editor.close();
    assert!(is_closed(&editor));
    assert_eq!(catalog.state(), before.as_slice());
    Ok(())
}
