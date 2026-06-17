//! macOS implementation of `PlatformOps`.
//!
//! Uses `route`, `networksetup`, and `ifconfig`. Firewall kill-switch is routed
//! through the (not-yet-installed) privileged helper rather than touching `pf`
//! directly — manipulating the packet filter wrong can lock the user off the net.

use super::{random_mac, run, PlatformOps};
use gos_core::types::{GosError, Result};
use std::net::SocketAddr;

#[derive(Default)]
pub struct MacOs;

impl MacOs {
    /// Map the primary interface (e.g. `en0`) to its network *service* name
    /// (e.g. `Wi-Fi`), which `networksetup` operates on.
    fn primary_service(&self) -> Result<String> {
        let iface = self.primary_interface()?;
        let listing = run("networksetup", &["-listnetworkserviceorder"])?;
        // Blocks look like:
        //   (1) Wi-Fi
        //   (Hardware Port: Wi-Fi, Device: en0)
        let mut last_service: Option<String> = None;
        for line in listing.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix('(') {
                if let Some(idx) = rest.find(')') {
                    // "(1) Wi-Fi" -> service name after ") "
                    if let Some(name) = trimmed.splitn(2, ") ").nth(1) {
                        last_service = Some(name.trim().to_string());
                    }
                    let _ = idx;
                }
            }
            if trimmed.contains(&format!("Device: {iface}")) {
                if let Some(svc) = last_service.take() {
                    return Ok(svc);
                }
            }
        }
        Err(GosError::Platform(format!(
            "no network service found for interface {iface}"
        )))
    }
}

impl PlatformOps for MacOs {
    fn name(&self) -> &'static str {
        "macos"
    }

    fn primary_interface(&self) -> Result<String> {
        let out = run("route", &["-n", "get", "default"])?;
        for line in out.lines() {
            let line = line.trim();
            if let Some(iface) = line.strip_prefix("interface:") {
                return Ok(iface.trim().to_string());
            }
        }
        Err(GosError::Platform("no default route interface".into()))
    }

    fn set_socks_proxy(&self, addr: Option<SocketAddr>) -> Result<()> {
        let svc = self.primary_service()?;
        match addr {
            Some(a) => {
                run(
                    "networksetup",
                    &[
                        "-setsocksfirewallproxy",
                        &svc,
                        &a.ip().to_string(),
                        &a.port().to_string(),
                    ],
                )?;
                run("networksetup", &["-setsocksfirewallproxystate", &svc, "on"])?;
            }
            None => {
                run("networksetup", &["-setsocksfirewallproxystate", &svc, "off"])?;
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

    fn current_mac(&self, iface: &str) -> Result<String> {
        let out = run("ifconfig", &[iface])?;
        for line in out.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("ether ") {
                return Ok(rest.trim().to_string());
            }
        }
        Err(GosError::Platform(format!("no MAC for interface {iface}")))
    }

    fn randomize_mac(&self, iface: &str) -> Result<String> {
        let mac = random_mac();
        // Requires admin; surfaces a clear error if not privileged.
        run("ifconfig", &[iface, "ether", &mac])?;
        Ok(mac)
    }
}
