//! The app's version as the user sees it.

/// The crate version and the 7-char commit hash it was built from,
/// `0.1.0 (2391c1d)`; the version alone when built without git. Stamped
/// by `build.rs`.
pub const APP_VERSION: &str = env!("COFFEE_TIMER_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_the_crate_version() {
        assert!(APP_VERSION.starts_with(env!("CARGO_PKG_VERSION")));
    }
}
