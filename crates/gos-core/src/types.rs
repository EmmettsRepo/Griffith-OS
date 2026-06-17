//! Shared value types crossing the Rust <-> UI boundary.
//!
//! All types serialize as `camelCase` so the TypeScript UI reads them naturally.

use serde::{Deserialize, Serialize};

/// Top-level connection state of the privacy engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

impl Default for ConnectionState {
    fn default() -> Self {
        ConnectionState::Disconnected
    }
}

/// A live snapshot of the privacy engine, pushed to the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyStatus {
    pub state: ConnectionState,
    pub tor_enabled: bool,
    pub doh_enabled: bool,
    pub vpn_enabled: bool,
    pub kill_switch: bool,
    pub mac_randomized: bool,
    /// IP traffic appears to come from (Tor exit / VPN endpoint), if known.
    pub exit_ip: Option<String>,
    pub exit_country: Option<String>,
    /// Local SOCKS port other apps can route through, once Tor is up.
    pub socks_port: Option<u16>,
    /// This install's public node id (for GOS<->GOS transfer later).
    pub node_id: String,
    /// Human-readable status / last error.
    pub message: Option<String>,
}

impl PrivacyStatus {
    pub fn new(node_id: String) -> Self {
        PrivacyStatus {
            state: ConnectionState::Disconnected,
            tor_enabled: false,
            doh_enabled: true,
            vpn_enabled: false,
            kill_switch: true,
            mac_randomized: false,
            exit_ip: None,
            exit_country: None,
            socks_port: None,
            node_id,
            message: None,
        }
    }
}

/// Host cross-over bridges. Each is independently revocable; all-off = full isolation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossoverToggles {
    pub files: bool,
    pub launch_apps: bool,
    pub clipboard: bool,
    pub devices: bool,
}

impl Default for CrossoverToggles {
    /// Host integration is on by default; the user can isolate at will.
    fn default() -> Self {
        CrossoverToggles {
            files: true,
            launch_apps: true,
            clipboard: true,
            devices: true,
        }
    }
}

impl CrossoverToggles {
    pub fn all_off() -> Self {
        CrossoverToggles {
            files: false,
            launch_apps: false,
            clipboard: false,
            devices: false,
        }
    }

    /// True when every host bridge is severed ("Isolation Mode").
    pub fn is_isolated(&self) -> bool {
        !self.files && !self.launch_apps && !self.clipboard && !self.devices
    }
}

/// Plain-language verdict for the privacy/leak check. Never claims "untrackable".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LeakVerdict {
    /// Real IP is hidden and DNS is encrypted.
    Protected,
    /// Some protection, but at least one gap (e.g. DoH off, or VPN-only).
    Partial,
    /// Real IP is visible — treat as not protected.
    Exposed,
}

/// Result of a one-tap leak test comparing the real vs. exit IP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeakReport {
    pub real_ip: Option<String>,
    pub exit_ip: Option<String>,
    pub ip_masked: bool,
    pub doh_active: bool,
    pub verdict: LeakVerdict,
    pub details: Vec<String>,
    pub checked_at_ms: i64,
}

/// Crate-wide error type, convertible to a `String` for Tauri command results.
#[derive(Debug, thiserror::Error)]
pub enum GosError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("privacy engine: {0}")]
    Privacy(String),
    #[error("platform op failed: {0}")]
    Platform(String),
    #[error("host bridge: {0}")]
    Bridge(String),
    #[error("{0}")]
    Other(String),
}

impl From<GosError> for String {
    fn from(e: GosError) -> String {
        e.to_string()
    }
}

pub type Result<T> = std::result::Result<T, GosError>;
