//! DNS-over-HTTPS via the JSON API (RFC 8484 `application/dns-json`).
//!
//! Rather than a heavyweight DNS-over-HTTPS stack, we do a single HTTPS GET to a
//! pinned IPv4 resolver endpoint and parse the JSON answer — reusing the same
//! `http_get` helper used for the IP checks. Lighter, and fully under our control.

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use gos_core::settings::DohProvider;
use gos_core::types::{GosError, Result};
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;

use crate::crypto::ensure_crypto_provider;
use crate::net::http_get;

/// Pinned IPv4 endpoint + TLS server name per provider (avoids needing DNS to
/// find the DNS server, and dodges IPv6-only-route failures).
fn endpoint(provider: DohProvider) -> (Ipv4Addr, &'static str) {
    match provider {
        DohProvider::Cloudflare => (Ipv4Addr::new(1, 1, 1, 1), "cloudflare-dns.com"),
        DohProvider::Quad9 => (Ipv4Addr::new(9, 9, 9, 9), "dns.quad9.net"),
        DohProvider::Google => (Ipv4Addr::new(8, 8, 8, 8), "dns.google"),
    }
}

fn tls_connector() -> Result<TlsConnector> {
    ensure_crypto_provider();
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let mut config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    // We speak HTTP/1.0 over the stream, so advertise http/1.1 at most.
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(TlsConnector::from(Arc::new(config)))
}

#[derive(Deserialize)]
struct DohResp {
    #[serde(rename = "Answer")]
    answer: Option<Vec<DohAnswer>>,
}

#[derive(Deserialize)]
struct DohAnswer {
    #[serde(rename = "type")]
    rtype: u16,
    data: String,
}

/// Resolve `host`'s A records over encrypted DoH using the chosen provider.
pub async fn resolve(provider: DohProvider, host: &str) -> Result<Vec<IpAddr>> {
    let (ip, tls_name) = endpoint(provider);
    let connector = tls_connector()?;

    let tcp = TcpStream::connect((ip, 443))
        .await
        .map_err(|e| GosError::Privacy(format!("DoH connect: {e}")))?;
    let server_name = ServerName::try_from(tls_name.to_string())
        .map_err(|e| GosError::Privacy(format!("DoH server name: {e}")))?;
    let tls = connector
        .connect(server_name, tcp)
        .await
        .map_err(|e| GosError::Privacy(format!("DoH TLS handshake: {e}")))?;

    let path = format!("/dns-query?name={host}&type=A");
    let body = http_get(tls, tls_name, &path, "application/dns-json").await?;

    let resp: DohResp = serde_json::from_str(&body).map_err(|e| {
        let preview: String = body.chars().take(80).collect();
        GosError::Privacy(format!("DoH parse ({preview}): {e}"))
    })?;

    let ips: Vec<IpAddr> = resp
        .answer
        .unwrap_or_default()
        .into_iter()
        .filter(|a| a.rtype == 1) // A records
        .filter_map(|a| a.data.parse::<IpAddr>().ok())
        .collect();

    if ips.is_empty() {
        return Err(GosError::Privacy("DoH returned no A records".into()));
    }
    Ok(ips)
}
