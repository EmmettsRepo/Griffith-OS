//! Linux implementation of `PlatformOps` (compiled only on Linux).
//!
//! Uses `ip`, `nmcli`, and would route the kill-switch through nftables via the
//! privileged helper. Not exercised on the macOS dev machine.

use super::{random_mac, run, PlatformOps};
use gos_core::types::{GosError, Result};
use std::net::SocketAddr;

#[derive(Default)]
pub struct Linux;

impl PlatformOps for Linux {
    fn name(&self) -> &'static str {
        "linux"
    }

    fn primary_interface(&self) -> Result<String> {
        // `ip route show default` -> "default via X dev eth0 ..."
        let out = run("ip", &["route", "show", "default"])?;
        let mut it = out.split_whitespace();
        while let Some(tok) = it.next() {
            if tok == "dev" {
                if let Some(dev) = it.next() {
                    return Ok(dev.to_string());
                }
            }
        }
        Err(GosError::Platform("no default route interface".into()))
    }

    fn set_socks_proxy(&self, addr: Option<SocketAddr>) -> Result<()> {
        match addr {
            Some(a) => {
                run(
                    "gsettings",
                    &["set", "org.gnome.system.proxy", "mode", "'manual'"],
                )?;
                run(
                    "gsettings",
                    &[
                        "set",
                        "org.gnome.system.proxy.socks",
                        "host",
                        &format!("'{}'", a.ip()),
                    ],
                )?;
                run(
                    "gsettings",
                    &[
                        "set",
                        "org.gnome.system.proxy.socks",
                        "port",
                        &a.port().to_string(),
                    ],
                )?;
            }
            None => {
                run(
                    "gsettings",
                    &["set", "org.gnome.system.proxy", "mode", "'none'"],
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

    fn current_mac(&self, iface: &str) -> Result<String> {
        let out = run("cat", &[&format!("/sys/class/net/{iface}/address")])?;
        Ok(out.trim().to_string())
    }

    fn randomize_mac(&self, iface: &str) -> Result<String> {
        let mac = random_mac();
        run("ip", &["link", "set", "dev", iface, "down"])?;
        run("ip", &["link", "set", "dev", iface, "address", &mac])?;
        run("ip", &["link", "set", "dev", iface, "up"])?;
        Ok(mac)
    }
}
