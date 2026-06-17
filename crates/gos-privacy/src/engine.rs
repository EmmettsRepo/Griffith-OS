//! The privacy engine — the single async object the Tauri app drives.
//!
//! Holds privacy state behind a mutex and exposes high-level operations:
//! start/stop Tor, toggle DoH, run a leak test, request a new identity.
//! Compiles with or without the `tor` feature so the app always runs.

use tokio::sync::Mutex;

use gos_core::now_ms;
use gos_core::settings::DohProvider;
use gos_core::types::{
    ConnectionState, GosError, LeakReport, LeakVerdict, PrivacyStatus, Result,
};

use crate::{doh, net};

#[cfg(feature = "tor")]
use crate::tor::{self, GosTorClient};
#[cfg(feature = "tor")]
use std::sync::Arc;

pub struct PrivacyEngine {
    inner: Mutex<Inner>,
}

struct Inner {
    status: PrivacyStatus,
    doh_provider: DohProvider,
    #[cfg(feature = "tor")]
    tor: Option<Arc<GosTorClient>>,
}

impl PrivacyEngine {
    pub fn new(node_id: String) -> Self {
        PrivacyEngine {
            inner: Mutex::new(Inner {
                status: PrivacyStatus::new(node_id),
                doh_provider: DohProvider::default(),
                #[cfg(feature = "tor")]
                tor: None,
            }),
        }
    }

    pub async fn status(&self) -> PrivacyStatus {
        self.inner.lock().await.status.clone()
    }

    pub async fn set_doh(&self, enabled: bool, provider: DohProvider) -> PrivacyStatus {
        let mut g = self.inner.lock().await;
        g.status.doh_enabled = enabled;
        g.doh_provider = provider;
        g.status.clone()
    }

    /// Bootstrap Tor and capture the exit IP.
    pub async fn start_tor(&self) -> Result<PrivacyStatus> {
        #[cfg(not(feature = "tor"))]
        {
            Err(GosError::Privacy(
                "Tor support was not built into this binary".into(),
            ))
        }
        #[cfg(feature = "tor")]
        {
            {
                let mut g = self.inner.lock().await;
                g.status.state = ConnectionState::Connecting;
                g.status.message = Some("bootstrapping Tor…".into());
            }
            let client = tor::bootstrap().await?;
            let info = tor::exit_ip(client.as_ref()).await.ok();

            let mut g = self.inner.lock().await;
            g.tor = Some(client);
            g.status.state = ConnectionState::Connected;
            g.status.tor_enabled = true;
            if let Some(info) = info {
                g.status.exit_ip = Some(info.ip);
                g.status.exit_country = info.country;
            }
            g.status.message = Some("Tor connected".into());
            Ok(g.status.clone())
        }
    }

    pub async fn stop_tor(&self) -> PrivacyStatus {
        let mut g = self.inner.lock().await;
        #[cfg(feature = "tor")]
        {
            g.tor = None;
        }
        g.status.tor_enabled = false;
        g.status.state = ConnectionState::Disconnected;
        g.status.exit_ip = None;
        g.status.exit_country = None;
        g.status.message = Some("Tor stopped".into());
        g.status.clone()
    }

    /// Request a fresh circuit (new exit IP) while Tor stays up.
    pub async fn new_identity(&self) -> Result<PrivacyStatus> {
        #[cfg(not(feature = "tor"))]
        {
            Err(GosError::Privacy(
                "Tor support was not built into this binary".into(),
            ))
        }
        #[cfg(feature = "tor")]
        {
            let client = { self.inner.lock().await.tor.clone() };
            let client =
                client.ok_or_else(|| GosError::Privacy("Tor is not running".into()))?;
            let info = tor::exit_ip_isolated(client.as_ref()).await?;
            let mut g = self.inner.lock().await;
            g.status.exit_ip = Some(info.ip);
            g.status.exit_country = info.country;
            g.status.message = Some("new identity".into());
            Ok(g.status.clone())
        }
    }

    /// One-tap leak test: compare the real vs. exit IP and confirm DoH works.
    pub async fn leak_test(&self) -> LeakReport {
        let (doh_enabled, provider, exit_ip_known) = {
            let g = self.inner.lock().await;
            (g.status.doh_enabled, g.doh_provider, g.status.exit_ip.clone())
        };

        let mut details = Vec::new();

        // Real IP via a direct connection.
        let real_ip = match net::direct_ip().await {
            Ok(i) => Some(i.ip),
            Err(e) => {
                details.push(format!("could not determine real IP: {e}"));
                None
            }
        };

        // Exit IP: re-measure over Tor if it's running, else use last known.
        #[allow(unused_mut)]
        let mut exit_ip = exit_ip_known;
        #[cfg(feature = "tor")]
        {
            let client = { self.inner.lock().await.tor.clone() };
            if let Some(client) = client {
                match tor::exit_ip(client.as_ref()).await {
                    Ok(i) => exit_ip = Some(i.ip),
                    Err(e) => details.push(format!("Tor exit check failed: {e}")),
                }
            }
        }

        // DoH check.
        let doh_active = if doh_enabled {
            match doh::resolve(provider, "example.com").await {
                Ok(ips) => {
                    details.push(format!("DoH OK via {provider:?}: {} address(es)", ips.len()));
                    true
                }
                Err(e) => {
                    details.push(format!("DoH check failed: {e}"));
                    false
                }
            }
        } else {
            details.push("DoH disabled".into());
            false
        };

        let ip_masked = match (&real_ip, &exit_ip) {
            (Some(r), Some(x)) => r != x,
            (None, Some(_)) => true,
            _ => false,
        };

        if ip_masked {
            details.push("real IP is hidden behind the exit node".into());
        } else {
            details.push("real IP is visible — not masked".into());
        }

        let verdict = match (ip_masked, doh_active) {
            (true, true) => LeakVerdict::Protected,
            (true, false) | (false, true) => LeakVerdict::Partial,
            (false, false) => LeakVerdict::Exposed,
        };

        LeakReport {
            real_ip,
            exit_ip,
            ip_masked,
            doh_active,
            verdict,
            details,
            checked_at_ms: now_ms(),
        }
    }
}
