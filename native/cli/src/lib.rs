//! Command-line head on the core and store: a second shell beside the hub.
//! It builds the controllers, turns arguments into actions, passes every
//! `CatalogChange` on, renders state, and prints effects. It decides nothing.

pub mod args;
pub mod cli_error;
pub mod commands;
pub mod render;

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use coffee_timer_core::brew::BrewController;
use coffee_timer_core::catalog::RecipeCatalog;
use coffee_timer_core::editor::EditorController;
use coffee_timer_core::model::Pin;
use coffee_timer_core::ports::{IdGen, StoreError};
use coffee_timer_core::settings::SettingsController;
use coffee_timer_store::SqliteStore;

use crate::args::Command;
use crate::cli_error::CliError;
use crate::commands::{BrewPause, BrewRun};

/// Every controller, built and loaded from one store.
pub struct Controllers {
    pub catalog: RecipeCatalog,
    pub settings: SettingsController,
    pub brew: BrewController,
    pub editor: EditorController,
}

impl Controllers {
    /// Builds each controller on `store` and `ids`, then loads the catalog
    /// and settings.
    pub fn boot(store: &SqliteStore, ids: Box<dyn IdGen>) -> Result<Self, StoreError> {
        let mut catalog = RecipeCatalog::new(Box::new(store.clone()));
        catalog.load()?;
        let mut settings = SettingsController::new(Box::new(store.clone()));
        settings.load()?;
        Ok(Self {
            catalog,
            settings,
            brew: BrewController::new(),
            editor: EditorController::new(ids),
        })
    }
}

/// Runs one command. `brew` waits a real second between ticks unless
/// `--fast` is given.
pub fn run(command: &Command, c: &mut Controllers, out: &mut impl Write) -> Result<(), CliError> {
    match command {
        Command::List => commands::list(&c.catalog, out),
        Command::Show {
            id,
            ratio_steps,
            coffee,
            save,
        } => commands::show(
            id,
            *coffee,
            *ratio_steps,
            *save,
            &mut c.brew,
            &mut c.catalog,
            out,
        ),
        Command::New(recipe) => {
            commands::new_recipe(recipe, &mut c.editor, &mut c.catalog, &mut c.brew, out)
        }
        Command::Delete { id } => {
            commands::delete(id, &mut c.editor, &mut c.catalog, &mut c.brew, out)
        }
        Command::Duplicate { id, name } => commands::duplicate(
            id,
            name.as_deref(),
            &mut c.editor,
            &mut c.catalog,
            &mut c.brew,
            out,
        ),
        Command::Pin { id } => commands::set_pin(id, Pin::Pinned, &mut c.catalog, &mut c.brew, out),
        Command::Unpin { id } => {
            commands::set_pin(id, Pin::Unpinned, &mut c.catalog, &mut c.brew, out)
        }
        Command::Move { id, to } => commands::move_to(id, *to, &mut c.catalog, &mut c.brew, out),
        Command::Export { ids, out: file } => {
            commands::export(ids, file.as_deref(), &c.catalog, out)
        }
        Command::Import { file } => commands::import(
            &read_input(file)?,
            &mut c.editor,
            &mut c.catalog,
            &mut c.brew,
            out,
        ),
        Command::Brew {
            id,
            from,
            fast,
            pause_at,
            pause_for,
        } => {
            let mut skip = || {};
            let mut sleep = || thread::sleep(Duration::from_secs(1));
            let wait: &mut dyn FnMut() = if *fast { &mut skip } else { &mut sleep };
            let pause = pause_at.map(|at| BrewPause {
                at,
                seconds: *pause_for,
            });
            let run = BrewRun { from: *from, pause };
            commands::brew(
                id,
                run,
                wait,
                &mut c.brew,
                &c.catalog,
                c.settings.state(),
                out,
            )
        }
        Command::Settings { action } => commands::settings(action.as_ref(), &mut c.settings, out),
    }
}

/// The text of `file`, or of stdin for `-`.
fn read_input(file: &Path) -> io::Result<String> {
    if file == Path::new("-") {
        let mut text = String::new();
        io::stdin().read_to_string(&mut text)?;
        return Ok(text);
    }
    std::fs::read_to_string(file)
}

/// `$XDG_DATA_HOME/coffee_timer/coffee_timer.db`, else
/// `$HOME/.local/share/coffee_timer/coffee_timer.db`, else
/// `./coffee_timer.db`.
pub fn default_db_path() -> PathBuf {
    let data_dir = std::env::var_os("XDG_DATA_HOME")
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|dir| !dir.is_empty())
                .map(|home| PathBuf::from(home).join(".local").join("share"))
        });
    match data_dir {
        Some(dir) => dir.join("coffee_timer").join("coffee_timer.db"),
        None => PathBuf::from("coffee_timer.db"),
    }
}
