//! SQLite persistence: implements the core's store traits. No decisions.

mod migrations;
mod recipe_rows;
mod settings_rows;
mod sqlite_store;

pub use sqlite_store::SqliteStore;
