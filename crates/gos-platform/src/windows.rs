//! Windows implementation of `PlatformOps` (compiled only on Windows).
//!
//! Uses `netsh` and the registry. Kill-switch would use Windows Filtering
//! Platform via the privileged helper. Not exercised on the macOS dev machine.

use super::{random_mac, run, PlatformOps};
use gos_core::types::{GosError, Result};
use std::net::SocketAddr;

#[derive(Default)]
pub struct Windows;

impl PlatformOps for Windows {
    fn name(&self) -> &'static str {
        "windows"
    }

    fn primary_interface(&self) -> Result<String> {
        // Best-effort: first connected interface from `netsh`.
        let out = run("netsh", &["interface", "show", "interface"])?;
        for line in out.lines() {
            if line.contains("Connected") {
                if let Some(name) = line.split_whitespace().last() {
                    return Ok(name.to_string());
                }
            }
        }
        Err(GosError::Platform("no connected interface".into()))
    }

    fn set_socks_proxy(&self, addr: Option<SocketAddr>) -> Result<()> {
        match addr {
            Some(a) => {
                run(
                    "reg",
                    &[
                        "add",
                        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
                        "/v",
                        "ProxyServer",
                        "/t",
                        "REG_SZ",
                        "/d",
                        &format!("socks={a}"),
                        "/f",
                    ],
                )?;
                run(
                    "reg",
                    &[
                        "add",
                        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
                        "/v",
                        "ProxyEnable",
                        "/t",
                        "REG_DWORD",
                        "/d",
                        "1",
                        "/f",
                    ],
                )?;
            }
            None => {
                run(
                    "reg",
                    &[
                        "add",
                        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
                        "/v",
                        "ProxyEnable",
                        "/t",
                        "REG_DWORD",
                        "/d",
                        "0",
                        "/f",
                    ],
                )?;
            }
        }
        Ok(())
    }

    fn enable_kill_switch(&self, _allow: &[SocketAddr]) -> Result<()> {
        Err(GosError::Platform(
            "kill-switch needs the GOS privileged helper (not installed yet)".into(),
        ))
    }

    fn disable_kill_switch(&self) -> Result<()> {
        Err(GosError::Platform(
            "kill-switch needs the GOS privileged helper (not installed yet)".into(),
        ))
    }

    fn current_mac(&self, _iface: &str) -> Result<String> {
        let out = run("getmac", &["/fo", "csv", "/nh"])?;
        if let Some(first) = out.lines().next() {
            if let Some(mac) = first.split(',').next() {
                return Ok(mac.trim_matches('"').to_string());
            }
        }
        Err(GosError::Platform("no MAC found".into()))
    }

    fn randomize_mac(&self, _iface: &str) -> Result<String> {
        // Windows MAC change is per-adapter via the registry + adapter restart;
        // deferred to the privileged helper. Return the candidate we'd apply.
        Err(GosError::Platform(format!(
            "MAC randomization on Windows needs the GOS privileged helper (candidate {})",
            random_mac()
        )))
    }
}
