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
fn show_with_save_persists_the_adjusted_water_and_pours() -> TestResult {
    let mut session = Session::new()?;
    let out = session.run(&["show", "pourover-uuid", "--ratio-steps", "3", "--save"])?;
    assert_eq!(
        out,
        "\
upserted pourover-uuid
pourover-uuid  Pour Over
Light and clean
Coffee: 15g  Water: 270g  Ratio: 1:18  Time: 3:10
  1. Grind  40s  pending
  2. Bloom  30s  54g  pending
  3. Pour  120s  216g  pending
"
    );
    let reopened = session.reopened()?.run(&["show", "pourover-uuid"])?;
    assert!(
        reopened.contains("Coffee: 15g  Water: 270g  Ratio: 1:18  Time"),
        "{reopened}"
    );
    assert!(
        reopened.contains("  2. Bloom  30s  54g  pending\n"),
        "{reopened}"
    );
    assert!(
        reopened.contains("  3. Pour  120s  216g  pending\n"),
        "{reopened}"
    );
    Ok(())
}

#[test]
fn show_with_save_past_what_a_recipe_holds_is_an_error() -> TestResult {
    let mut session = Session::new()?;
    // The seeds fit even at 1:30, so: 100 g of coffee to 1950 g (1:19.5),
    // two steps up is 1:20.5, 2050 g.
    session.run(&["new", "Batch", "--coffee", "100", "--water", "1950"])?;
    let error = session
        .run(&["show", "r1", "--ratio-steps", "2", "--save"])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error,
        "Can't save to the recipe: 2050 g of water is outside 1–2000 g"
    );
    let list = session.reopened()?.run(&["list"])?;
    assert!(list.contains("Water: 1950 g"), "{list}");
    Ok(())
}

#[test]
fn show_with_save_needs_a_changed_dose() -> TestResult {
    let error = Session::new()?
        .run(&["show", "pourover-uuid", "--save"])
        .err()
        .ok_or("expected an error")?;
    assert!(error.contains("--ratio-steps"), "{error}");
    assert!(error.contains("--coffee"), "{error}");
    Ok(())
}

#[test]
fn show_with_coffee_scales_the_water_and_pours_for_that_showing() -> TestResult {
    let mut session = Session::new()?;
    let less = session.run(&["show", "pourover-uuid", "--coffee", "9"])?;
    assert!(
        less.contains("Coffee: 9g (adjusted)  Water: 150g  Ratio: 1:16.7  Time"),
        "{less}"
    );
    assert!(less.contains("  2. Bloom  30s  30g  pending\n"), "{less}");
    assert!(less.contains("  3. Pour  120s  120g  pending\n"), "{less}");
    let both = session.run(&[
        "show",
        "pourover-uuid",
        "--coffee",
        "10",
        "--ratio-steps",
        "3",
    ])?;
    assert!(
        both.contains("Coffee: 10g (adjusted)  Water: 180g  Ratio: 1:18 (adjusted)"),
        "{both}"
    );
    let plain = session.run(&["show", "pourover-uuid"])?;
    assert!(plain.contains("Coffee: 15g  Water: 250g  Ratio: 1:16.7  Time"));
    Ok(())
}

#[test]
fn show_with_coffee_and_save_persists_the_coffee_water_and_pours() -> TestResult {
    let mut session = Session::new()?;
    session.run(&["show", "pourover-uuid", "--coffee", "9", "--save"])?;
    let reopened = session.reopened()?.run(&["show", "pourover-uuid"])?;
    assert!(
        reopened.contains("Coffee: 9g  Water: 150g  Ratio: 1:16.7  Time"),
        "{reopened}"
    );
    assert!(
        reopened.contains("  3. Pour  120s  120g  pending\n"),
        "{reopened}"
    );
    Ok(())
}

#[test]
fn show_with_coffee_out_of_range_is_an_error() -> TestResult {
    let error = Session::new()?
        .run(&["show", "pourover-uuid", "--coffee", "201"])
        .err()
        .ok_or("expected an error")?;
    assert!(error.contains("1–200"), "{error}");
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
fn new_times_a_pour_without_seconds_at_the_pour_speed() -> TestResult {
    let mut session = Session::new()?;
    let out = session.run(&[
        "new",
        "Timed",
        "--pour-speed",
        "4",
        "--step",
        "Bloom=@50",
        "--step",
        "Pour=20@200",
    ])?;
    assert_eq!(out, "upserted r1\n");
    let shown = session.run(&["show", "r1"])?;
    assert!(shown.contains("  1. Bloom  13s  50g  pending\n"), "{shown}");
    assert!(
        shown.contains("  2. Pour  20s  200g  pending\n"),
        "typed seconds win: {shown}"
    );
    Ok(())
}

#[test]
fn new_with_an_invalid_pour_speed_prints_its_error() -> TestResult {
    let mut session = Session::new()?;
    let error = session
        .run(&["new", "Fast", "--pour-speed", "31"])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error,
        "Fix the highlighted fields\n  Pour speed \"31\": 0.5–30 g/s"
    );
    Ok(())
}

#[test]
fn new_with_a_grind_lists_and_shows_it() -> TestResult {
    let mut session = Session::new()?;
    let out = session.run(&["new", "V60", "--grind", "22"])?;
    assert_eq!(out, "upserted r1\n");
    let list = session.reopened()?.run(&["list"])?;
    assert!(
        list.ends_with("r1  V60  Coffee: 15 g  Water: 250 g  Time: 0:00  Clicks: 22\n"),
        "{list}"
    );
    let shown = session.run(&["show", "r1"])?;
    assert!(
        shown.contains("Ratio: 1:16.7  Time: 0:00  Clicks: 22\n"),
        "{shown}"
    );
    Ok(())
}

#[test]
fn new_with_an_invalid_grind_prints_its_error() -> TestResult {
    let mut session = Session::new()?;
    let error = session
        .run(&["new", "V60", "--grind", "100"])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error,
        "Fix the highlighted fields\n  Grind \"100\": 0–99 clicks"
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

const PINNED_POUR_OVER_LIST: &str = "\
pourover-uuid  Pour Over [pinned]  Coffee: 15 g  Water: 250 g  Time: 3:10  Light and clean
espresso-uuid  Espresso  Coffee: 18 g  Water: 36 g  Time: 0:55  Strong coffee
";

#[test]
fn pin_lists_the_recipe_first_with_a_marker_and_it_persists() -> TestResult {
    let mut session = Session::new()?;
    assert_eq!(
        session.run(&["pin", "pourover-uuid"])?,
        "upserted pourover-uuid\n"
    );
    assert_eq!(session.run(&["list"])?, PINNED_POUR_OVER_LIST);
    assert_eq!(session.reopened()?.run(&["list"])?, PINNED_POUR_OVER_LIST);
    Ok(())
}

#[test]
fn unpin_puts_the_recipe_back_in_its_place() -> TestResult {
    let mut session = Session::new()?;
    session.run(&["pin", "pourover-uuid"])?;
    assert_eq!(
        session.run(&["unpin", "pourover-uuid"])?,
        "upserted pourover-uuid\n"
    );
    assert_eq!(session.reopened()?.run(&["list"])?, SEED_LIST);
    Ok(())
}

#[test]
fn pin_of_an_unknown_id_is_the_core_error() -> TestResult {
    let error = Session::new()?
        .run(&["pin", "nope"])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error, "No recipe with id nope");
    Ok(())
}

#[test]
fn move_puts_the_recipe_at_its_new_place_and_it_persists() -> TestResult {
    let mut session = Session::new()?;
    assert_eq!(
        session.run(&["move", "pourover-uuid", "0"])?,
        "upserted pourover-uuid\n"
    );
    assert_eq!(
        session.reopened()?.run(&["list"])?,
        "\
pourover-uuid  Pour Over  Coffee: 15 g  Water: 250 g  Time: 3:10  Light and clean
espresso-uuid  Espresso  Coffee: 18 g  Water: 36 g  Time: 0:55  Strong coffee
"
    );
    Ok(())
}

#[test]
fn move_keeps_an_unpinned_recipe_below_the_pinned_ones() -> TestResult {
    let mut session = Session::new()?;
    session.run(&["pin", "pourover-uuid"])?;
    session.run(&["move", "espresso-uuid", "0"])?;
    assert_eq!(session.run(&["list"])?, PINNED_POUR_OVER_LIST);
    Ok(())
}

#[test]
fn move_of_an_unknown_id_is_the_core_error() -> TestResult {
    let error = Session::new()?
        .run(&["move", "nope", "0"])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error, "No recipe with id nope");
    Ok(())
}

#[test]
fn a_duplicate_of_a_pinned_recipe_is_listed_unpinned_at_the_end() -> TestResult {
    let mut session = Session::new()?;
    session.run(&["pin", "pourover-uuid"])?;
    session.run(&["duplicate", "pourover-uuid"])?;
    assert_eq!(
        session.reopened()?.run(&["list"])?,
        format!(
            "{PINNED_POUR_OVER_LIST}r1  Pour Over copy  Coffee: 15 g  Water: 250 g  Time: 3:10  Light and clean\n"
        )
    );
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
[sound breath-and-bowl]
[notify Zero · 00:00 | Next: B]
tick 3   00:01 B  Last step  |  1:done 2:done 3:current
[vibrate 100ms]
[sound breath-and-bowl]
[notify B · 00:01]
tick 4   done 0:03  |  1:done 2:done 3:done
[screen off]
[vibrate pattern 0/340/100/130/90/130/90/340/100/340/540/340/100/500ms at 0/140/0/70/0/70/0/100/0/100/0/140/0/180]
[sound singing-bowl]
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
[sound breath-and-bowl]
[notify B · 00:01]
tick 4   done 0:04  |  1:done 2:done
[screen off]
[vibrate pattern 0/340/100/130/90/130/90/340/100/340/540/340/100/500ms at 0/140/0/70/0/70/0/100/0/100/0/140/0/180]
[sound singing-bowl]
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
                "[vibrate pattern 0/50/100/50ms]",
                "[notify A · 00:03 | Next: B]",
            ][..]
        ),
        "{out}"
    );
    assert_eq!(
        lines
            .iter()
            .filter(|&&line| line == "[vibrate pattern 0/50/100/50ms]")
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

/// The cue lines `settings` prints at the defaults.
const DEFAULT_CUES: &str = "Sound: on\nVibration: on\nFinish knock: on\nPre-alert: 3 s\n\
Step sound: breath-and-bowl\nFinish sound: singing-bowl\n";

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
Pre-alert options: 0, 3, 5, 10 (seconds; 0 is off)
Step sound options:
  temple-block     Temple block         1.0 s
  wooden-clappers  Wooden clappers      1.0 s
  tingsha          Tingsha              1.0 s
  water-drop       Water drop           1.0 s
  kalimba          Kalimba              1.0 s
  breath-and-bowl  Breath and bowl      1.0 s
  bamboo-tock      Bamboo tock          1.0 s
Finish sound options:
  singing-bowl     Singing bowl         3.0 s
  bowl-twice       Bowl struck twice    4.6 s
  bowl-cadence     Bowl cadence         4.6 s
  kalimba-cadence  Kalimba cadence      2.6 s
  kalimba-rise     Kalimba rise         2.6 s
  tingsha-three    Tingsha three times  3.8 s
  counter-bell     Counter bell         3.4 s
  wind-chimes      Wind chimes          4.0 s
  last-drops       Last drops           2.4 s
  gong             Gong                 4.4 s
  spoon-on-cup     Spoon on cup         3.2 s
  music-box        Music box            3.9 s
";

#[test]
fn settings_shows_the_defaults_and_options() -> TestResult {
    assert_eq!(
        Session::new()?.run(&["settings"])?,
        format!("Theme: system\nAccent: #FF2196F3\n{DEFAULT_CUES}{ACCENT_OPTIONS}")
    );
    Ok(())
}

#[test]
fn settings_set_theme_and_accent_persist() -> TestResult {
    let mut session = Session::new()?;
    session.run(&["settings", "theme", "dark"])?;
    assert_eq!(
        session.run(&["settings", "accent", "teal"])?,
        format!("Theme: dark\nAccent: #FF009688\n{DEFAULT_CUES}{ACCENT_OPTIONS}")
    );
    session.run(&["settings", "accent", "#123456"])?;
    assert_eq!(
        session.reopened()?.run(&["settings"])?,
        format!("Theme: dark\nAccent: #FF123456\n{DEFAULT_CUES}{ACCENT_OPTIONS}")
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

#[test]
fn settings_set_sound_vibration_and_pre_alert_persist() -> TestResult {
    let mut session = Session::new()?;
    session.run(&["settings", "sound", "off"])?;
    session.run(&["settings", "vibration", "off"])?;
    assert_eq!(
        session.run(&["settings", "pre-alert", "10"])?,
        format!(
            "Theme: system\nAccent: #FF2196F3\nSound: off\nVibration: off\nFinish knock: on\nPre-alert: 10 s\n\
             Step sound: breath-and-bowl\nFinish sound: singing-bowl\n{ACCENT_OPTIONS}"
        )
    );
    session.run(&["settings", "pre-alert", "0"])?;
    let reopened = session.reopened()?.run(&["settings"])?;
    assert!(
        reopened.contains("Sound: off\nVibration: off\nFinish knock: on\nPre-alert: Off\n"),
        "{reopened}"
    );
    session.run(&["settings", "sound", "on"])?;
    assert!(
        session
            .run(&["settings"])?
            .contains("Sound: on\nVibration: off\n")
    );
    Ok(())
}

#[test]
fn settings_reject_an_unknown_toggle_or_pre_alert() -> TestResult {
    let mut session = Session::new()?;
    assert_eq!(
        session.run(&["settings", "sound", "loud"]).err().as_deref(),
        Some("\"loud\" is not on or off")
    );
    assert_eq!(
        session
            .run(&["settings", "vibration", "yes"])
            .err()
            .as_deref(),
        Some("\"yes\" is not on or off")
    );
    assert_eq!(
        session
            .run(&["settings", "pre-alert", "4"])
            .err()
            .as_deref(),
        Some("Pre-alert \"4\" is not 0, 3, 5 or 10 seconds")
    );
    assert!(session.run(&["settings"])?.contains(DEFAULT_CUES));
    Ok(())
}

#[test]
fn settings_set_step_and_finish_sounds_persist() -> TestResult {
    let mut session = Session::new()?;
    session.run(&["settings", "step-sound", "kalimba"])?;
    assert!(
        session
            .run(&["settings", "finish-sound", "gong"])?
            .contains("Pre-alert: 3 s\nStep sound: kalimba\nFinish sound: gong\n")
    );
    let reopened = session.reopened()?.run(&["settings"])?;
    assert!(
        reopened.contains("Step sound: kalimba\nFinish sound: gong\n"),
        "{reopened}"
    );
    Ok(())
}

#[test]
fn settings_reject_an_unknown_sound_or_one_of_the_other_kind() -> TestResult {
    let mut session = Session::new()?;
    for (action, raw, error) in [
        ("step-sound", "bell", "No step sound named \"bell\""),
        ("step-sound", "gong", "No step sound named \"gong\""),
        (
            "finish-sound",
            "bell",
            "No end-of-brew sound named \"bell\"",
        ),
        (
            "finish-sound",
            "kalimba",
            "No end-of-brew sound named \"kalimba\"",
        ),
    ] {
        assert_eq!(
            session.run(&["settings", action, raw]).err().as_deref(),
            Some(error),
            "{action} {raw}"
        );
    }
    assert!(session.run(&["settings"])?.contains(DEFAULT_CUES));
    Ok(())
}

#[test]
fn brew_plays_the_chosen_sounds() -> TestResult {
    let out = brewed_short(&[
        &["settings", "step-sound", "tingsha"],
        &["settings", "finish-sound", "kalimba-rise"],
    ])?;
    assert!(
        out.contains("[vibrate 100ms]\n[sound tingsha]\n[notify B · 00:01]\n"),
        "{out}"
    );
    assert!(
        out.ends_with("[sound kalimba-rise]\n[notify off]\n"),
        "{out}"
    );
    Ok(())
}

/// What `brew r1 --fast` prints for a recipe `A=4, B=1` under the settings
/// `setup` sets.
fn brewed_short(setup: &[&[&str]]) -> Result<String, Box<dyn std::error::Error>> {
    let mut session = Session::new()?;
    session.run(&["new", "Short", "--step", "A=4", "--step", "B=1"])?;
    for argv in setup {
        session.run(argv)?;
    }
    Ok(session.run(&["brew", "r1", "--fast"])?)
}

#[test]
fn brew_with_sound_off_prints_no_sound() -> TestResult {
    let out = brewed_short(&[&["settings", "sound", "off"]])?;
    assert!(!out.contains("[sound "), "{out}");
    assert!(out.contains("[vibrate pattern 0/50/100/50ms]\n"), "{out}");
    assert!(
        out.contains("[vibrate 100ms]\n[notify B · 00:01]\n"),
        "{out}"
    );
    assert!(
        out.ends_with("[screen off]\n[vibrate pattern 0/340/100/130/90/130/90/340/100/340/540/340/100/500ms at 0/140/0/70/0/70/0/100/0/100/0/140/0/180]\n[notify off]\n"),
        "{out}"
    );
    Ok(())
}

#[test]
fn brew_with_the_finish_knock_off_ends_on_two_long_buzzes() -> TestResult {
    let mut session = Session::new()?;
    assert!(
        session
            .run(&["settings", "finish-knock", "off"])?
            .contains("Vibration: on\nFinish knock: off\n")
    );
    let out = brewed_short(&[&["settings", "finish-knock", "off"]])?;
    assert!(
        out.ends_with(
            "[screen off]\n[vibrate pattern 0/400/200/400ms]\n[sound singing-bowl]\n[notify off]\n"
        ),
        "{out}"
    );
    Ok(())
}

#[test]
fn brew_with_vibration_off_prints_no_buzz() -> TestResult {
    let out = brewed_short(&[&["settings", "vibration", "off"]])?;
    assert!(!out.contains("[vibrate"), "{out}");
    assert!(
        out.contains("[sound breath-and-bowl]\n[notify B · 00:01]\n"),
        "{out}"
    );
    assert!(
        out.ends_with("[screen off]\n[sound singing-bowl]\n[notify off]\n"),
        "{out}"
    );
    Ok(())
}

#[test]
fn brew_with_both_off_prints_only_screen_and_notification_effects() -> TestResult {
    let out = brewed_short(&[
        &["settings", "sound", "off"],
        &["settings", "vibration", "off"],
    ])?;
    assert!(
        !out.contains("[vibrate") && !out.contains("[sound "),
        "{out}"
    );
    assert!(out.ends_with("[screen off]\n[notify off]\n"), "{out}");
    Ok(())
}

#[test]
fn brew_follows_the_pre_alert_setting() -> TestResult {
    assert!(
        !brewed_short(&[&["settings", "pre-alert", "0"]])?
            .contains("[vibrate pattern 0/50/100/50ms]")
    );
    assert!(
        !brewed_short(&[&["settings", "pre-alert", "5"]])?
            .contains("[vibrate pattern 0/50/100/50ms]"),
        "the 4 s step is no longer than the lead"
    );
    let out = brewed_short(&[&["settings", "pre-alert", "3"]])?;
    assert!(
        out.contains("tick 1   00:03 A  Next: B · 0:01  |  1:current 2:pending\n[vibrate pattern 0/50/100/50ms]\n"),
        "{out}"
    );
    Ok(())
}

#[test]
fn export_prints_the_recipe_document_without_ids() -> TestResult {
    let printed = Session::new()?.run(&["export", "espresso-uuid"])?;
    assert!(
        printed.starts_with("{\n  \"format\": \"coffee-timer-recipes\",\n  \"version\": 1,"),
        "{printed}"
    );
    assert!(printed.contains("\"name\": \"Espresso\""), "{printed}");
    assert!(!printed.contains("Pour Over"), "{printed}");
    assert!(!printed.contains("uuid"), "no ids: {printed}");
    Ok(())
}

#[test]
fn export_of_an_unknown_id_is_the_core_error() -> TestResult {
    let error = Session::new()?
        .run(&["export", "espresso-uuid", "nope"])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error, "No recipe with id nope");
    Ok(())
}

#[test]
fn export_then_import_into_a_fresh_database_gives_the_same_recipes_unpinned() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("recipes.json");
    let file_arg = file.to_str().ok_or("path is not UTF-8")?;

    let mut source = Session::new()?;
    source.run(&["pin", "pourover-uuid"])?;
    assert_eq!(
        source.run(&["export", "--out", file_arg])?,
        format!("Exported 2 recipes to {file_arg}\n")
    );

    let mut fresh = Session::new()?;
    fresh.run(&["delete", "espresso-uuid"])?;
    fresh.run(&["delete", "pourover-uuid"])?;
    assert_eq!(
        fresh.run(&["import", file_arg])?,
        "upserted r1\nupserted r2\nImported 2 recipes\n"
    );
    let mut reopened = fresh.reopened()?;
    assert_eq!(
        reopened.run(&["list"])?,
        "\
r1  Pour Over  Coffee: 15 g  Water: 250 g  Time: 3:10  Light and clean
r2  Espresso  Coffee: 18 g  Water: 36 g  Time: 0:55  Strong coffee
"
    );
    assert_eq!(
        reopened.run(&["export"])?,
        std::fs::read_to_string(&file)?,
        "the same recipes, steps and pours"
    );
    Ok(())
}

#[test]
fn import_of_an_invalid_document_adds_nothing() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("bad.json");
    std::fs::write(
        &file,
        r#"{"format":"coffee-timer-recipes","version":1,"recipes":[
          {"name":"V60","coffee_grams":15,"water_grams":250,"steps":[{"seconds":30}]},
          {"name":"Big","coffee_grams":500,"water_grams":250,"steps":[{"seconds":30}]}]}"#,
    )?;
    let mut session = Session::new()?;
    let error = session
        .run(&["import", file.to_str().ok_or("path is not UTF-8")?])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(error, "Can't import: Recipe 2, coffee_grams: 1–200 g");
    assert_eq!(session.reopened()?.run(&["list"])?, SEED_LIST);
    Ok(())
}

#[test]
fn import_of_a_newer_version_is_refused() -> TestResult {
    let dir = tempfile::tempdir()?;
    let file = dir.path().join("v2.json");
    std::fs::write(&file, r#"{"format":"coffee-timer-recipes","version":2}"#)?;
    let error = Session::new()?
        .run(&["import", file.to_str().ok_or("path is not UTF-8")?])
        .err()
        .ok_or("expected an error")?;
    assert_eq!(
        error,
        "Can't import: Recipe document version 2 is not supported; this app reads version 1"
    );
    Ok(())
}
