//! SQLite persistence and the UUID id generator: implements the core's
//! infrastructure traits. No decisions.

mod migrations;
mod recipe_rows;
mod settings_rows;
mod sqlite_store;
mod uuid_ids;

pub use sqlite_store::SqliteStore;
pub use uuid_ids::UuidIds;
