//! Command-line head on the core and store. See the crate library for the
//! commands; this only wires arguments, the database and the exit code.

use std::io;
use std::process::ExitCode;

use clap::Parser;
use coffee_timer_cli::args::Args;
use coffee_timer_cli::cli_error::CliError;
use coffee_timer_cli::{Controllers, default_db_path, run};
use coffee_timer_store::{SqliteStore, UuidIds};

fn main() -> ExitCode {
    let args = Args::parse();
    match open_and_run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn open_and_run(args: &Args) -> Result<(), CliError> {
    let path = args.db.clone().unwrap_or_else(default_db_path);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let store = SqliteStore::open(&path)?;
    let mut controllers = Controllers::boot(&store, Box::new(UuidIds))?;
    run(&args.command, &mut controllers, &mut io::stdout().lock())
}
