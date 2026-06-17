//! Shared application state managed by Tauri and accessed from commands.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gos_bridge::HostBridge;
use gos_core::GosSettings;
use gos_platform::PlatformOps;
use gos_privacy::PrivacyEngine;

pub struct AppState {
    /// The async privacy engine (Tor, DoH, leak tests).
    pub engine: Arc<PrivacyEngine>,
    /// OS-specific operations for the current target.
    pub platform: Box<dyn PlatformOps>,
    /// Host cross-over bridge (files / apps / devices), guarded by toggles.
    pub bridge: Mutex<HostBridge>,
    /// Persisted user settings.
    pub settings: Mutex<GosSettings>,
    pub settings_path: PathBuf,
    /// This install's public node id.
    pub node_id: String,
}
