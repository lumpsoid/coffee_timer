use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Coffee Timer on the command line: the same core and database as the app.
#[derive(Debug, Parser)]
#[command(name = "coffee_timer_cli", version)]
pub struct Args {
    /// SQLite database file. Created and seeded if missing.
    /// [default: $XDG_DATA_HOME/coffee_timer/coffee_timer.db, else
    /// ~/.local/share/coffee_timer/coffee_timer.db, else ./coffee_timer.db]
    #[arg(long, global = true, value_name = "PATH")]
    pub db: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List recipes as the app lists them, pinned ones first (marked
    /// `[pinned]`), with their coffee, water and time.
    List,
    /// Show a recipe and its steps as the brew page does.
    Show {
        /// Recipe id, as printed by `list`.
        id: String,
        /// Change the ratio by N steps of 0.5 for this showing, as the brew
        /// page's ratio buttons do: positive adds water, negative takes it
        /// away. The coffee stays.
        #[arg(
            long,
            default_value_t = 0,
            value_name = "N",
            allow_negative_numbers = true
        )]
        ratio_steps: i32,
        /// Save the adjusted ratio to the recipe, as the brew page's "Save
        /// to recipe" does: its water and step pours become the ones shown.
        #[arg(long, requires = "ratio_steps")]
        save: bool,
    },
    /// Create a recipe.
    New(NewRecipe),
    /// Delete a recipe.
    Delete {
        /// Recipe id, as printed by `list`.
        id: String,
    },
    /// Copy a recipe, as the list's Duplicate does, and save the copy at
    /// the end of the list: same description, coffee, water and steps, new
    /// ids, named "<name> copy" unless --name is given.
    Duplicate {
        /// Recipe id, as printed by `list`.
        id: String,
        /// Name the copy this instead of "<name> copy".
        #[arg(long, value_name = "NAME")]
        name: Option<String>,
    },
    /// Pin a recipe to the top of the list, as the list's long-press Pin
    /// does.
    Pin {
        /// Recipe id, as printed by `list`.
        id: String,
    },
    /// Unpin a recipe: it goes back to its place among the unpinned ones.
    Unpin {
        /// Recipe id, as printed by `list`.
        id: String,
    },
    /// Write recipes as a JSON recipe document (no ids, no pins), as the
    /// app's Export copies them.
    Export {
        /// Recipe ids, as printed by `list`, in the order to write them.
        /// None: every recipe, in list order.
        ids: Vec<String>,
        /// Write the document to this file instead of stdout.
        #[arg(long, value_name = "FILE")]
        out: Option<PathBuf>,
    },
    /// Add every recipe in a JSON recipe document as a new, unpinned
    /// recipe at the end of the list, and print the new ids. All or
    /// nothing: an invalid document adds none.
    Import {
        /// The document's file, or `-` for stdin.
        file: PathBuf,
    },
    /// Brew a recipe: one tick per second until the last step ends, printing
    /// the countdown, step statuses and effects.
    Brew {
        /// Recipe id, as printed by `list`.
        id: String,
        /// Step to start from, 0-based (0 brews the whole recipe).
        #[arg(long, default_value_t = 0, value_name = "N")]
        from: usize,
        /// Tick without waiting a second between ticks.
        #[arg(long)]
        fast: bool,
        /// Pause after this many ticks, as tapping the running dial does;
        /// ignored if the brew has ended by then.
        #[arg(long, value_name = "SECONDS")]
        pause_at: Option<u32>,
        /// Stay paused this many seconds, then resume (no wait with
        /// `--fast`).
        #[arg(
            long,
            value_name = "SECONDS",
            default_value_t = 0,
            requires = "pause_at"
        )]
        pause_for: u32,
    },
    /// Show settings, or change the theme, accent, sound, vibration or
    /// pre-alert.
    Settings {
        #[command(subcommand)]
        action: Option<SettingsAction>,
    },
}

#[derive(Debug, Clone, clap::Args)]
pub struct NewRecipe {
    /// Recipe name.
    pub name: String,
    /// Recipe description.
    #[arg(long, default_value = "")]
    pub description: String,
    /// Ground coffee in grams, 1–200. [default: the editor's, 15]
    #[arg(long, value_name = "GRAMS")]
    pub coffee: Option<String>,
    /// Water in grams, 1–2000. [default: the editor's, 250]
    #[arg(long, value_name = "GRAMS")]
    pub water: Option<String>,
    /// How fast the recipe pours, in grams per second, 0.5–30.
    /// [default: 5]
    #[arg(long, value_name = "G/S")]
    pub pour_speed: Option<String>,
    /// Where to set the grinder, in clicks, 0–99. [default: none]
    #[arg(long, value_name = "CLICKS")]
    pub grind: Option<String>,
    /// A step as NAME=SECONDS, or NAME=SECONDS@GRAMS for a step that pours
    /// that much water (e.g. "Grind=30", "Bloom=30@50"); repeat in brew
    /// order. NAME=@GRAMS leaves out the seconds: the step lasts as long as
    /// its pour takes at the pour speed. A blank NAME saves as "Step N".
    /// Without any, the recipe gets one blank 0-second step.
    #[arg(long = "step", value_name = "NAME=SECONDS[@GRAMS]", value_parser = StepArg::parse)]
    pub steps: Vec<StepArg>,
}

/// One `--step NAME=SECONDS[@GRAMS]`, as typed. The editor validates every
/// part.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepArg {
    pub name: String,
    pub seconds: String,
    /// The grams poured, if `@GRAMS` was given.
    pub water: Option<String>,
}

impl StepArg {
    /// Splits at the last `=`, so a name may contain `=`; then the seconds
    /// at an `@`, if any.
    pub fn parse(raw: &str) -> Result<Self, String> {
        let (name, amounts) = raw
            .rsplit_once('=')
            .ok_or_else(|| format!("expected NAME=SECONDS[@GRAMS], got \"{raw}\""))?;
        let (seconds, water) = match amounts.split_once('@') {
            Some((seconds, water)) => (seconds, Some(water.to_owned())),
            None => (amounts, None),
        };
        Ok(Self {
            name: name.to_owned(),
            seconds: seconds.to_owned(),
            water,
        })
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum SettingsAction {
    /// Set the theme: light, dark or system.
    Theme { value: String },
    /// Set the accent: an option name (e.g. Teal) or #AARRGGBB / #RRGGBB.
    Accent { value: String },
    /// Turn the brew's step sound on or off.
    Sound { value: String },
    /// Turn the brew's vibration on or off: step boundaries, the pre-alert
    /// and the finish.
    Vibration { value: String },
    /// Buzz this many seconds before each step ends: 0 (off), 3, 5 or 10.
    PreAlert { value: String },
}
