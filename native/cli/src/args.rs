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
    /// List recipes in catalog order with their coffee, water and time.
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
    /// Show settings, or change the theme or accent.
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
    /// A step as NAME=SECONDS, or NAME=SECONDS@GRAMS for a step that pours
    /// that much water (e.g. "Grind=30", "Bloom=30@50"); repeat in brew
    /// order. A blank NAME saves as "Step N". Without any, the recipe gets
    /// one blank 0-second step.
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
}
