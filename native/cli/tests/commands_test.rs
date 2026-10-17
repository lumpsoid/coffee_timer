//! The CLI commands against an in-memory, seeded SQLite store with
//! sequential ids, asserting what they print.

use clap::Parser;
use coffee_timer_cli::args::Args;
use coffee_timer_cli::{Controllers, run};
use coffee_timer_core::fakes::SequentialIds;
use coffee_timer_store::SqliteStore;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// A shell on a fresh seeded database: Espresso (Grind 30s, Brew 25s) and
/// Pour Over (Grind 40s, Bloom 30s, Pour 120s).
struct Session {
    store: SqliteStore,
    controllers: Controllers,
}

impl Session {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let store = SqliteStore::open_in_memory()?;
        let controllers = Controllers::boot(&store, Box::new(SequentialIds::default()))?;
        Ok(Self { store, controllers })
    }

    /// Runs `argv` (without the program name) and returns what it printed,
    /// or the error text main would print to stderr.
    fn run(&mut self, argv: &[&str]) -> Result<String, String> {
        let args =
            Args::try_parse_from(std::iter::once("coffee_timer_cli").chain(argv.iter().copied()))
                .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        run(&args.command, &mut self.controllers, &mut out).map_err(|e| e.to_string())?;
        String::from_utf8(out).map_err(|e| e.to_string())
    }

    /// The same database, read back by a fresh set of controllers.
    fn reopened(&self) -> Result<Self, Box<dyn std::error::Error>> {
        let controllers = Controllers::boot(&self.store, Box::new(SequentialIds::default()))?;
        Ok(Self {
            store: self.store.clone(),
            controllers,
        })
    }
}

const SEED_LIST: &str = "\
espresso-uuid  Espresso  Coffee: 20 g  Water: 100 g  Time: 0:55  Strong coffee
pourover-uuid  Pour Over  Coffee: 30 g  Water: 150 g  Time: 3:10  Light and clean
";

#[test]
fn list_shows_the_seed_in_catalog_order() -> TestResult {
    assert_eq!(Session::new()?.run(&["list"])?, SEED_LIST);
    Ok(())
}

#[test]
fn show_renders_the_brew_page() -> TestResult {
    assert_eq!(
        Session::new()?.run(&["show", "pourover-uuid"])?,
        "\
pourover-uuid  Pour Over
Light and clean
Coffee: 30g  Water: 150g  Time: 3:10
  1. Grind  40s  pending
  2. Bloom  30s  pending
  3. Pour  120s  pending
"
    );
    Ok(())
}

#[test]
fn show_of_an_unknown_id_is_the_empty_page() -> TestResult {
    assert_eq!(Session::new()?.run(&["show", "nope"])?, "Empty recipe\n");
    Ok(())
}

#[test]
fn new_saves_a_recipe_and_it_persists() -> TestResult {
    let mut session = Session::new()?;
    let out = session.run(&[
        "new",
        "  V60 ",
        "--description",
        "clean ",
        "--step",
        "Bloom=30",
        "--step",
        "=5",
    ])?;
    assert_eq!(out, "upserted r1\n");

    let mut reopened = session.reopened()?;
    let list = reopened.run(&["list"])?;
    assert_eq!(
        list,
        format!("{SEED_LIST}r1  V60  Coffee: 20 g  Water: 100 g  Time: 0:35  clean\n")
    );
    assert_eq!(
        reopened.run(&["show", "r1"])?,
        "\
r1  V60
clean
Coffee: 20g  Water: 100g  Time: 0:35
  1. Bloom  30s  pending
  2. Step 2  5s  pending
"
    );
    Ok(())
}

#[test]
fn new_without_steps_saves_one_blank_step() -> TestResult {
    let mut session = Session::new()?;
    assert_eq!(session.run(&["new", "Plain"])?, "upserted r1\n");
    assert!(
        session
            .run(&["show", "r1"])?
            .ends_with("  1. Step 1  0s  pending\n")
    );
    Ok(())
}

#[test]
fn new_with_invalid_fields_prints_the_field_errors_and_saves_nothing() -> TestResult {
    let mut session = Session::new()?;
    let error = session
        .run(&[
            "new",
            " ",
            "--step",
            "Grind=30",
            "--step",
            "Bloom=abc",
            "--step",
            "Pour=6000",
        ])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error,
        "\
Fix the highlighted fields
  Recipe name \" \": Enter a recipe name
  Step 2 name \"Bloom\", duration \"abc\": Enter seconds
  Step 3 name \"Pour\", duration \"6000\": 0–5999 seconds"
    );
    assert_eq!(session.reopened()?.run(&["list"])?, SEED_LIST);
    Ok(())
}

#[test]
fn a_step_without_equals_is_an_argument_error() -> TestResult {
    let error = Session::new()?
        .run(&["new", "V60", "--step", "Bloom"])
        .err()
        .ok_or("expected an error")?;
    assert!(
        error.contains("expected NAME=SECONDS, got \"Bloom\""),
        "{error}"
    );
    Ok(())
}

#[test]
fn delete_removes_the_recipe_and_it_persists() -> TestResult {
    let mut session = Session::new()?;
    assert_eq!(
        session.run(&["delete", "espresso-uuid"])?,
        "removed espresso-uuid\n"
    );
    assert_eq!(
        session.reopened()?.run(&["list"])?,
        "pourover-uuid  Pour Over  Coffee: 30 g  Water: 150 g  Time: 3:10  Light and clean\n"
    );
    Ok(())
}

#[test]
fn delete_of_an_unknown_id_is_the_editor_error() -> TestResult {
    let error = Session::new()?
        .run(&["delete", "nope"])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error, "No recipe with id nope");
    Ok(())
}

#[test]
fn fast_brew_prints_every_tick_and_effect() -> TestResult {
    let mut session = Session::new()?;
    session.run(&[
        "new", "Short", "--step", "A=2", "--step", "Zero=0", "--step", "B=1",
    ])?;
    assert_eq!(
        session.run(&["brew", "r1", "--fast"])?,
        "\
start    00:02 A  |  1:current 2:pending 3:pending
[screen on]
tick 1   00:01 A  |  1:current 2:pending 3:pending
tick 2   00:00 Zero  |  1:done 2:current 3:pending
[vibrate 100ms]
[sound]
tick 3   00:01 B  |  1:done 2:done 3:current
[vibrate 100ms]
[sound]
tick 4   idle 0:03  |  1:pending 2:pending 3:pending
[screen off]
[vibrate 100ms]
"
    );
    Ok(())
}

#[test]
fn brew_from_a_later_step_starts_there() -> TestResult {
    let out = Session::new()?.run(&["brew", "espresso-uuid", "--from", "1", "--fast"])?;
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines.first().copied(),
        Some("start    00:25 Brew  |  1:done 2:current")
    );
    assert_eq!(lines.len(), 2 + 25 + 2);
    assert_eq!(
        lines.get(lines.len() - 3).copied(),
        Some("tick 25  idle 0:55  |  1:pending 2:pending")
    );
    Ok(())
}

#[test]
fn brew_from_out_of_range_is_the_core_error() -> TestResult {
    let error = Session::new()?
        .run(&["brew", "espresso-uuid", "--from", "2", "--fast"])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error, "Step 2 is out of range for 2 steps");
    Ok(())
}

#[test]
fn brew_of_an_unknown_id_is_the_core_error() -> TestResult {
    let error = Session::new()?
        .run(&["brew", "nope", "--fast"])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error, "No recipe is open");
    Ok(())
}

const ACCENT_OPTIONS: &str = "\
Themes: light, dark, system
Accent options:
  Blue    #FF2196F3
  Red     #FFF44336
  Green   #FF4CAF50
  Purple  #FF9C27B0
  Orange  #FFFF9800
  Pink    #FFE91E63
  Indigo  #FF3F51B5
  Teal    #FF009688
  Cyan    #FF00BCD4
  Lime    #FFCDDC39
";

#[test]
fn settings_shows_the_defaults_and_options() -> TestResult {
    assert_eq!(
        Session::new()?.run(&["settings"])?,
        format!("Theme: system\nAccent: #FF2196F3\n{ACCENT_OPTIONS}")
    );
    Ok(())
}

#[test]
fn settings_set_theme_and_accent_persist() -> TestResult {
    let mut session = Session::new()?;
    session.run(&["settings", "theme", "dark"])?;
    assert_eq!(
        session.run(&["settings", "accent", "teal"])?,
        format!("Theme: dark\nAccent: #FF009688\n{ACCENT_OPTIONS}")
    );
    session.run(&["settings", "accent", "#123456"])?;
    assert_eq!(
        session.reopened()?.run(&["settings"])?,
        format!("Theme: dark\nAccent: #FF123456\n{ACCENT_OPTIONS}")
    );
    Ok(())
}

#[test]
fn settings_reject_an_unknown_theme_or_accent() -> TestResult {
    let mut session = Session::new()?;
    assert_eq!(
        session
            .run(&["settings", "theme", "sepia"])
            .err()
            .as_deref(),
        Some("Theme \"sepia\" is not light, dark or system")
    );
    assert_eq!(
        session
            .run(&["settings", "accent", "Mauve"])
            .err()
            .as_deref(),
        Some("Accent \"Mauve\" is not #AARRGGBB or #RRGGBB")
    );
    assert!(
        session
            .run(&["settings"])?
            .starts_with("Theme: system\nAccent: #FF2196F3\n")
    );
    Ok(())
}
