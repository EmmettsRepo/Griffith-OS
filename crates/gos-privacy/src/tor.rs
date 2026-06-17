//! Embedded Tor client via `arti` (pure-Rust Tor). Compiled only with `--features tor`.
//!
//! We bootstrap a `TorClient`, then connect to the IP-echo endpoint *through Tor*
//! to learn the exit IP. A SOCKS listener for routing other apps is a later phase;
//! this proves real Tor circuits end-to-end.

use std::sync::Arc;

use arti_client::{TorClient, TorClientConfig};
use tor_rtcompat::PreferredRuntime;

use crate::crypto::ensure_crypto_provider;
use crate::net::{http_get, parse_ip_api, IpInfo, IP_API_HOST, IP_API_PATH};
use gos_core::types::{GosError, Result};

pub type GosTorClient = TorClient<PreferredRuntime>;

/// Bootstrap a Tor client (connects to the Tor network). arti returns an `Arc`.
pub async fn bootstrap() -> Result<Arc<GosTorClient>> {
    ensure_crypto_provider();
    let config = TorClientConfig::default();
    TorClient::create_bootstrapped(config)
        .await
        .map_err(|e| GosError::Privacy(format!("Tor bootstrap failed: {e}")))
}

/// Connect to the IP-echo endpoint through Tor and return the exit IP.
pub async fn exit_ip(client: &GosTorClient) -> Result<IpInfo> {
    let stream = client
        .connect((IP_API_HOST, 80))
        .await
        .map_err(|e| GosError::Privacy(format!("Tor connect: {e}")))?;
    let body = http_get(stream, IP_API_HOST, IP_API_PATH, "*/*").await?;
    parse_ip_api(&body)
}

/// "New identity": an isolated client uses fresh circuits, yielding a new exit.
pub async fn exit_ip_isolated(client: &GosTorClient) -> Result<IpInfo> {
    let iso = client.isolated_client();
    exit_ip(iso.as_ref()).await
}
