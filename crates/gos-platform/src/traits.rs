//! The cross-platform operations contract.

use gos_core::types::{GosError, Result};
use std::net::SocketAddr;

/// Privileged / OS-specific network operations GOS needs.
///
/// Several of these require admin rights and will eventually run through a small
/// privileged helper. Implementations should fail with a clear message rather
/// than doing anything that could lock the user out of their network.
pub trait PlatformOps: Send + Sync {
    fn name(&self) -> &'static str;

    /// The interface carrying the default route (e.g. `en0`, `eth0`).
    fn primary_interface(&self) -> Result<String>;

    /// Point the OS (or its primary service) at a local SOCKS proxy, or clear it.
    fn set_socks_proxy(&self, addr: Option<SocketAddr>) -> Result<()>;

    /// Block all egress except the allowed addresses (kill-switch on).
    fn enable_kill_switch(&self, allow: &[SocketAddr]) -> Result<()>;

    /// Remove kill-switch rules (kill-switch off).
    fn disable_kill_switch(&self) -> Result<()>;

    /// Read the current MAC of an interface.
    fn current_mac(&self, iface: &str) -> Result<String>;

    /// Assign a (typically random) MAC to an interface; returns the new MAC.
    fn randomize_mac(&self, iface: &str) -> Result<String>;
}

/// Fallback for unsupported targets — every op is a clear error.
pub struct Unsupported;

impl PlatformOps for Unsupported {
    fn name(&self) -> &'static str {
        "unsupported"
    }
    fn primary_interface(&self) -> Result<String> {
        Err(GosError::Platform("unsupported platform".into()))
    }
    fn set_socks_proxy(&self, _addr: Option<SocketAddr>) -> Result<()> {
        Err(GosError::Platform("unsupported platform".into()))
    }
    fn enable_kill_switch(&self, _allow: &[SocketAddr]) -> Result<()> {
        Err(GosError::Platform("unsupported platform".into()))
    }
    fn disable_kill_switch(&self) -> Result<()> {
        Err(GosError::Platform("unsupported platform".into()))
    }
    fn current_mac(&self, _iface: &str) -> Result<String> {
        Err(GosError::Platform("unsupported platform".into()))
    }
    fn randomize_mac(&self, _iface: &str) -> Result<String> {
        Err(GosError::Platform("unsupported platform".into()))
    }
}
