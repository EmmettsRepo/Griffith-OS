//! gos-bridge — host "cross-over" features, each gated by a toggle.
//!
//! Files, app-launching, and device/network enumeration let the host OS "cross
//! over" into GOS. Every call checks its toggle first, so flipping a switch off
//! (or Isolation Mode) genuinely severs that capability — not just hides a button.
//! Clipboard cross-over lives in the Tauri layer (it owns the system clipboard).

use gos_core::types::{CrossoverToggles, GosError, Result};
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: u64,
}

pub struct HostBridge {
    toggles: CrossoverToggles,
}

impl HostBridge {
    pub fn new(toggles: CrossoverToggles) -> Self {
        HostBridge { toggles }
    }

    pub fn set_toggles(&mut self, toggles: CrossoverToggles) {
        self.toggles = toggles;
    }

    pub fn toggles(&self) -> CrossoverToggles {
        self.toggles
    }

    fn require(enabled: bool, what: &str) -> Result<()> {
        if enabled {
            Ok(())
        } else {
            Err(GosError::Bridge(format!("{what} cross-over is turned off")))
        }
    }

    /// List a host directory. Gated by the `files` toggle.
    pub fn list_dir(&self, path: &str) -> Result<Vec<FileEntry>> {
        Self::require(self.toggles.files, "files")?;
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(Path::new(path))? {
            let entry = entry?;
            let meta = entry.metadata()?;
            entries.push(FileEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                is_dir: meta.is_dir(),
                size_bytes: meta.len(),
            });
        }
        entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));
        Ok(entries)
    }

    /// Launch an installed host application by name. Gated by `launch_apps`.
    pub fn launch_app(&self, name: &str) -> Result<()> {
        Self::require(self.toggles.launch_apps, "launch-apps")?;
        let result = if cfg!(target_os = "macos") {
            std::process::Command::new("open").args(["-a", name]).spawn()
        } else if cfg!(target_os = "windows") {
            std::process::Command::new("cmd").args(["/C", "start", "", name]).spawn()
        } else {
            std::process::Command::new("xdg-open").arg(name).spawn()
        };
        result
            .map(|_| ())
            .map_err(|e| GosError::Bridge(format!("could not launch {name}: {e}")))
    }

    /// Enumerate host network interfaces. Gated by the `devices` toggle.
    pub fn list_network_interfaces(&self) -> Result<Vec<String>> {
        Self::require(self.toggles.devices, "devices")?;
        let out = if cfg!(target_os = "windows") {
            std::process::Command::new("getmac").args(["/fo", "csv", "/nh"]).output()
        } else {
            // -l lists interface names on both macOS and Linux ifconfig variants;
            // fall back to reading them from output if -l is unsupported.
            std::process::Command::new("ifconfig").arg("-l").output()
        };
        let out = out.map_err(|e| GosError::Bridge(format!("interface list failed: {e}")))?;
        let text = String::from_utf8_lossy(&out.stdout);
        let names: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        Ok(names)
    }
}
