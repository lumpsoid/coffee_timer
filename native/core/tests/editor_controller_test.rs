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
    ModelError, Recipe, RecipeId, RecipeName, RecipeStep, StepDuration, StepId,
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
        vec![RecipeStep::new(
            sid("step-1")?,
            "Bloom".to_owned(),
            StepDuration::try_parse(30)?,
        )],
    ))
}

fn espresso() -> Result<Recipe, ModelError> {
    Ok(Recipe::blank(
        rid("espresso")?,
        RecipeName::try_parse("Espresso")?,
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

    editor.add_step(StepPreset::Blank)?;
    editor.set_step_duration(&sid("s2")?, "600")?;
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
