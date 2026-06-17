//! gos-core — shared types, settings, and node identity for Griffith OS.
//!
//! Everything here is dependency-light and platform-agnostic so every other crate
//! (privacy, platform, bridge, the Tauri app) can build on a common vocabulary.

pub mod identity;
pub mod settings;
pub mod types;

pub use identity::NodeIdentity;
pub use settings::GosSettings;
pub use types::*;

/// Milliseconds since the Unix epoch. Used for timestamps in reports.
pub fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
