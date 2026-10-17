//! The rinf bridge: turns Dart signals into controller actions and
//! controller state into Rust signals. Entry point of the Rust logic.

mod brew;
mod editor;
mod recipe_list;
mod run;
mod settings;
mod signals;

use std::path::Path;

use coffee_timer_core::ports::StoreError;
use coffee_timer_store::SqliteStore;
use rinf::{DartSignal, RustSignal, dart_shutdown, write_interface};

use crate::run::Controllers;
use crate::signals::{Boot, BootFailed};

write_interface!();

/// The database file inside `Boot::db_dir`.
const DB_FILE: &str = "coffee_timer.db";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tokio::spawn(boot_then_run());
    // Keep the main function running until Dart shutdown.
    dart_shutdown().await;
}

/// Waits for `Boot`, opens and loads the store at its directory, then runs
/// the hub task. A failed boot sends `BootFailed` and waits for the next
/// `Boot`. Once the hub runs, a later `Boot` (Flutter hot restart) stays
/// unread: the `<Feature>Ready` handshake re-sends every state instead.
async fn boot_then_run() {
    let boot = Boot::get_dart_signal_receiver();
    while let Some(pack) = boot.recv().await {
        match boot_at(Path::new(&pack.message.db_dir)) {
            Ok(controllers) => return run::run(controllers).await,
            Err(error) => BootFailed {
                message: error.to_string(),
            }
            .send_signal_to_dart(),
        }
    }
}

fn boot_at(db_dir: &Path) -> Result<Controllers, StoreError> {
    Controllers::boot(&SqliteStore::open(db_dir.join(DB_FILE))?)
}
