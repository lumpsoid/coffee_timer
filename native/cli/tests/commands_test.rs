//! The CLI commands against an in-memory, seeded SQLite store with
//! sequential ids, asserting what they print.

use clap::Parser;
use coffee_timer_cli::args::Args;
use coffee_timer_cli::{Controllers, run};
use coffee_timer_core::fakes::SequentialIds;
use coffee_timer_store::SqliteStore;

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// A shell on a fresh seeded database: Espresso (Grind 30s, Brew 25s) and
/// Pour Over (Grind 40s, Bloom 30s pouring 50 g, Pour 120s pouring 200 g).
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
espresso-uuid  Espresso  Coffee: 18 g  Water: 36 g  Time: 0:55  Strong coffee
pourover-uuid  Pour Over  Coffee: 15 g  Water: 250 g  Time: 3:10  Light and clean
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
Coffee: 15g  Water: 250g  Ratio: 1:16.7  Time: 3:10
  1. Grind  40s  pending
  2. Bloom  30s  50g  pending
  3. Pour  120s  200g  pending
"
    );
    Ok(())
}

#[test]
fn show_with_ratio_steps_scales_the_step_pours() -> TestResult {
    let more = Session::new()?.run(&["show", "pourover-uuid", "--ratio-steps", "3"])?;
    assert!(more.contains("  2. Bloom  30s  54g  pending\n"), "{more}");
    assert!(more.contains("  3. Pour  120s  216g  pending\n"), "{more}");
    Ok(())
}

#[test]
fn show_with_ratio_steps_moves_the_water_for_that_showing() -> TestResult {
    let mut session = Session::new()?;
    let more = session.run(&["show", "pourover-uuid", "--ratio-steps", "3"])?;
    assert!(more.contains("Coffee: 15g  Water: 270g  Ratio: 1:18 (adjusted)"));
    let less = session.run(&["show", "pourover-uuid", "--ratio-steps", "-2"])?;
    assert!(less.contains("Coffee: 15g  Water: 240g  Ratio: 1:16 (adjusted)"));
    let plain = session.run(&["show", "pourover-uuid"])?;
    assert!(plain.contains("Coffee: 15g  Water: 250g  Ratio: 1:16.7  Time"));
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
        "--coffee",
        "16",
        "--water",
        "256",
        "--step",
        "Bloom=30@40",
        "--step",
        "=5",
    ])?;
    assert_eq!(out, "upserted r1\n");

    let mut reopened = session.reopened()?;
    let list = reopened.run(&["list"])?;
    assert_eq!(
        list,
        format!("{SEED_LIST}r1  V60  Coffee: 16 g  Water: 256 g  Time: 0:35  clean\n")
    );
    assert_eq!(
        reopened.run(&["show", "r1"])?,
        "\
r1  V60
clean
Coffee: 16g  Water: 256g  Ratio: 1:16  Time: 0:35
  1. Bloom  30s  40g  pending
  2. Step 2  5s  pending
"
    );
    Ok(())
}

#[test]
fn new_without_steps_or_dose_saves_the_editors_defaults() -> TestResult {
    let mut session = Session::new()?;
    assert_eq!(session.run(&["new", "Plain"])?, "upserted r1\n");
    let shown = session.run(&["show", "r1"])?;
    assert!(shown.contains("Coffee: 15g  Water: 250g  Ratio: 1:16.7"));
    assert!(shown.ends_with("  1. Step 1  0s  pending\n"));
    Ok(())
}

#[test]
fn new_with_invalid_fields_prints_the_field_errors_and_saves_nothing() -> TestResult {
    let mut session = Session::new()?;
    let error = session
        .run(&[
            "new",
            " ",
            "--coffee",
            "0",
            "--water",
            "lots",
            "--step",
            "Grind=30",
            "--step",
            "Bloom=abc",
            "--step",
            "Pour=6000",
            "--step",
            "Wait=10@0",
        ])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error,
        "\
Fix the highlighted fields
  Recipe name \" \": Enter a recipe name
  Coffee \"0\": 1–200 g
  Water \"lots\": Enter grams
  Step 2 name \"Bloom\", duration \"abc\": Enter seconds
  Step 3 name \"Pour\", duration \"6000\": 0–5999 seconds
  Step 4 name \"Wait\", grams \"0\": 1–2000 g"
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
        error.contains("expected NAME=SECONDS[@GRAMS], got \"Bloom\""),
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
        "pourover-uuid  Pour Over  Coffee: 15 g  Water: 250 g  Time: 3:10  Light and clean\n"
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
fn duplicate_saves_a_copy_at_the_end_and_it_persists() -> TestResult {
    let mut session = Session::new()?;
    assert_eq!(
        session.run(&["duplicate", "pourover-uuid"])?,
        "upserted r1\n"
    );
    let mut reopened = session.reopened()?;
    assert_eq!(
        reopened.run(&["list"])?,
        format!(
            "{SEED_LIST}r1  Pour Over copy  Coffee: 15 g  Water: 250 g  Time: 3:10  Light and clean\n"
        )
    );
    assert_eq!(
        reopened.run(&["show", "r1"])?,
        "\
r1  Pour Over copy
Light and clean
Coffee: 15g  Water: 250g  Ratio: 1:16.7  Time: 3:10
  1. Grind  40s  pending
  2. Bloom  30s  50g  pending
  3. Pour  120s  200g  pending
"
    );
    Ok(())
}

#[test]
fn duplicate_with_a_name_saves_the_copy_under_it() -> TestResult {
    let mut session = Session::new()?;
    assert_eq!(
        session.run(&["duplicate", "espresso-uuid", "--name", " Ristretto "])?,
        "upserted r1\n"
    );
    assert!(
        session
            .run(&["list"])?
            .ends_with("r1  Ristretto  Coffee: 18 g  Water: 36 g  Time: 0:55  Strong coffee\n")
    );
    Ok(())
}

#[test]
fn duplicate_with_a_blank_name_prints_the_field_error_and_saves_nothing() -> TestResult {
    let mut session = Session::new()?;
    let error = session
        .run(&["duplicate", "espresso-uuid", "--name", " "])
        .err()
        .ok_or("expected an error")?;
    assert!(error.contains("Enter a recipe name"), "{error}");
    assert_eq!(session.reopened()?.run(&["list"])?, SEED_LIST);
    Ok(())
}

#[test]
fn duplicate_of_an_unknown_id_is_the_editor_error() -> TestResult {
    let mut session = Session::new()?;
    let error = session
        .run(&["duplicate", "nope"])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error, "No recipe with id nope");
    assert_eq!(session.reopened()?.run(&["list"])?, SEED_LIST);
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
start    00:02 A  Next: Zero · 0:00  |  1:current 2:pending 3:pending
[screen on]
[notify A · 00:02 | Next: Zero]
tick 1   00:01 A  Next: Zero · 0:00  |  1:current 2:pending 3:pending
[notify A · 00:01 | Next: Zero]
tick 2   00:00 Zero  Next: B · 0:01  |  1:done 2:current 3:pending
[vibrate 100ms]
[sound]
[notify Zero · 00:00 | Next: B]
tick 3   00:01 B  Last step  |  1:done 2:done 3:current
[vibrate 100ms]
[sound]
[notify B · 00:01]
tick 4   done 0:03  |  1:done 2:done 3:done
[screen off]
[vibrate pattern 0/400/200/400ms]
[sound]
[notify off]
"
    );
    Ok(())
}

#[test]
fn brew_pauses_at_the_chosen_tick_and_resumes_where_it_was() -> TestResult {
    let mut session = Session::new()?;
    session.run(&["new", "Short", "--step", "A=3", "--step", "B=1"])?;
    assert_eq!(
        session.run(&[
            "brew",
            "r1",
            "--fast",
            "--pause-at",
            "1",
            "--pause-for",
            "5"
        ])?,
        "\
start    00:03 A  Next: B · 0:01  |  1:current 2:pending
[screen on]
[notify A · 00:03 | Next: B]
tick 1   00:02 A  Next: B · 0:01  |  1:current 2:pending
[notify A · 00:02 | Next: B]
pause    paused 00:02 A  Next: B · 0:01  |  1:paused 2:pending
[screen off]
[notify A · 00:02 · Paused | Next: B]
resume   00:02 A  Next: B · 0:01  |  1:current 2:pending
[screen on]
[notify A · 00:02 | Next: B]
tick 2   00:01 A  Next: B · 0:01  |  1:current 2:pending
[notify A · 00:01 | Next: B]
tick 3   00:01 B  Last step  |  1:done 2:current
[vibrate 100ms]
[sound]
[notify B · 00:01]
tick 4   done 0:04  |  1:done 2:done
[screen off]
[vibrate pattern 0/400/200/400ms]
[sound]
[notify off]
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
        Some("start    00:25 Brew  Last step  |  1:done 2:current")
    );
    // Start and each running tick add a notification line, the tick that
    // leaves 3 s adds the pre-alert, and the end adds four effect lines.
    // The run brewed only the step it started from.
    assert_eq!(lines.len(), 3 + 24 * 2 + 1 + 5);
    assert_eq!(
        lines.get(lines.len() - 5).copied(),
        Some("tick 25  done 0:25  |  1:done 2:done")
    );
    Ok(())
}

#[test]
fn brew_prints_the_pour_to_target_of_each_pouring_step() -> TestResult {
    let out = Session::new()?.run(&["brew", "pourover-uuid", "--from", "1", "--fast"])?;
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines.first().copied(),
        Some(
            "start    00:30 Bloom (Pour to 50g)  Next: Pour · 2:00 · 200g  |  1:done 2:current 3:pending"
        )
    );
    assert!(
        lines
            .contains(&"tick 30  02:00 Pour (Pour to 250g)  Last step  |  1:done 2:done 3:current"),
        "{out}"
    );
    // Done: the steps brewed and what the scale reads now.
    assert!(
        lines.contains(&"tick 150 done 2:30 (Poured to 250g)  |  1:done 2:done 3:done"),
        "{out}"
    );
    Ok(())
}

#[test]
fn brew_prints_the_pre_alert_before_a_step_ends() -> TestResult {
    let mut session = Session::new()?;
    session.run(&["new", "Short", "--step", "A=4", "--step", "B=1"])?;
    let out = session.run(&["brew", "r1", "--fast"])?;
    let lines: Vec<&str> = out.lines().collect();
    let tick1 = lines
        .iter()
        .position(|line| line.starts_with("tick 1 "))
        .ok_or("no tick 1")?;
    assert_eq!(
        lines.get(tick1..tick1 + 3),
        Some(
            &[
                "tick 1   00:03 A  Next: B · 0:01  |  1:current 2:pending",
                "[vibrate 40ms]",
                "[notify A · 00:03 | Next: B]",
            ][..]
        ),
        "{out}"
    );
    assert_eq!(
        lines
            .iter()
            .filter(|&&line| line == "[vibrate 40ms]")
            .count(),
        1,
        "once, and not for the 1 s step: {out}"
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
