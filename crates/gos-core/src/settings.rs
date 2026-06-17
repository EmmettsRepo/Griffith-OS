//! Persisted user settings — a small JSON file in the app config dir.
//!
//! Deliberately dependency-light (no DB, no plugin) so v1 stays simple.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::types::{CrossoverToggles, Result};

/// Which DoH provider to resolve through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DohProvider {
    Cloudflare,
    Quad9,
    Google,
}

impl Default for DohProvider {
    fn default() -> Self {
        DohProvider::Cloudflare
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GosSettings {
    pub tor_enabled: bool,
    pub doh_enabled: bool,
    pub vpn_enabled: bool,
    pub kill_switch: bool,
    pub mac_randomized: bool,
    pub doh_provider: DohProvider,
    pub crossover: CrossoverToggles,
    /// Accent color for the UI theme (hex). Lays groundwork for full theming.
    pub theme_accent: String,
}

impl Default for GosSettings {
    fn default() -> Self {
        GosSettings {
            tor_enabled: false,
            doh_enabled: true,
            vpn_enabled: false,
            kill_switch: true,
            mac_randomized: false,
            doh_provider: DohProvider::default(),
            crossover: CrossoverToggles::default(),
            theme_accent: "#4ade80".to_string(),
        }
    }
}

impl GosSettings {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)?;
        // Be forgiving: a malformed/old file falls back to defaults rather than crashing.
        Ok(serde_json::from_str(&raw).unwrap_or_default())
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}
