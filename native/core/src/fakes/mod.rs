//! Test doubles for the [`crate::ports`] traits. Compiled only for tests and
//! with the `test-fakes` feature, so they never reach the production API.
//! Integration tests in `native/core/tests/` get the feature through the
//! crate's self dev-dependency; another crate's tests can enable it the same
//! way.

mod fake_recipe_store;
mod fake_settings_store;
mod sequential_ids;

pub use fake_recipe_store::FakeRecipeStore;
pub use fake_settings_store::FakeSettingsStore;
pub use sequential_ids::SequentialIds;
