//! Coffee Timer business logic and state: feature controllers over state
//! objects. No rinf, no async runtime, no I/O.

pub mod catalog;
#[cfg(any(test, feature = "test-fakes"))]
pub mod fakes;
pub mod model;
pub mod ports;
pub mod settings;
