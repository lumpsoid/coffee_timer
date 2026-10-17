//! Infrastructure traits: the border between the core and what cannot run in
//! a test (disk, randomness). Controllers take them as `Box<dyn Trait>` in
//! their constructor; `coffee_timer_store` and the hub implement them; the
//! `fakes` module holds the test doubles.
//!
//! Each trait requires `Send`, so the controllers holding them can move into
//! the hub's spawned task.

mod id_gen;
mod recipe_store;
mod settings_store;
mod store_error;

pub use id_gen::IdGen;
pub use recipe_store::RecipeStore;
pub use settings_store::SettingsStore;
pub use store_error::StoreError;
