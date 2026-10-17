use rinf::{DartSignal, RustSignal};
use serde::{Deserialize, Serialize};

/// Sent once at app start: the directory the database lives in, resolved by
/// Dart (`path_provider`). The hub opens the store there, loads recipes and
/// settings, then answers each feature's ready signal.
#[derive(Deserialize, DartSignal)]
pub struct Boot {
    pub db_dir: String,
}

/// The store at [`Boot::db_dir`] could not be opened or loaded. Dart shows
/// `message` instead of the spinner; it may send `Boot` again to retry.
#[derive(Serialize, RustSignal)]
pub struct BootFailed {
    pub message: String,
}
