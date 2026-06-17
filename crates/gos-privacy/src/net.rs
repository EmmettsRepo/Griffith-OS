//! Minimal HTTP-over-any-stream helper, used to read the apparent public IP.
//!
//! Deliberately dependency-free (no reqwest/TLS): we do a tiny HTTP/1.0 GET over
//! whatever byte stream we're given — a direct TCP socket (to learn the *real*
//! IP) or a Tor `DataStream` (to learn the *exit* IP). Same code, two streams.

use gos_core::types::{GosError, Result};
use serde::Deserialize;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// A free, no-key, HTTP (port 80) IP-echo endpoint that also returns country.
pub const IP_API_HOST: &str = "ip-api.com";
pub const IP_API_PATH: &str = "/json/?fields=status,country,countryCode,query";

#[derive(Debug, Clone)]
pub struct IpInfo {
    pub ip: String,
    pub country: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Deserialize)]
struct IpApiResp {
    query: Option<String>,
    country: Option<String>,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
}

/// Issue `GET path` to `host` over an already-connected stream and return the body.
pub async fn http_get<S>(mut stream: S, host: &str, path: &str, accept: &str) -> Result<String>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let req = format!(
        "GET {path} HTTP/1.0\r\nHost: {host}\r\nUser-Agent: gos/0.1\r\nAccept: {accept}\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(req.as_bytes())
        .await
        .map_err(|e| GosError::Privacy(format!("http write: {e}")))?;
    let _ = stream.flush().await;

    let mut buf = Vec::with_capacity(4096);
    stream
        .read_to_end(&mut buf)
        .await
        .map_err(|e| GosError::Privacy(format!("http read: {e}")))?;

    let text = String::from_utf8_lossy(&buf);
    let body = text
        .splitn(2, "\r\n\r\n")
        .nth(1)
        .unwrap_or("")
        .trim()
        .to_string();
    Ok(body)
}

pub fn parse_ip_api(body: &str) -> Result<IpInfo> {
    let r: IpApiResp = serde_json::from_str(body).map_err(|e| {
        let preview: String = body.chars().take(80).collect();
        GosError::Privacy(format!("parse ip-api failed ({preview}): {e}"))
    })?;
    let ip = r
        .query
        .ok_or_else(|| GosError::Privacy("ip-api response had no IP".into()))?;
    Ok(IpInfo {
        ip,
        country: r.country,
        country_code: r.country_code,
    })
}

/// Learn the real public IP via a direct connection (no Tor, no proxy).
pub async fn direct_ip() -> Result<IpInfo> {
    let stream = tokio::net::TcpStream::connect((IP_API_HOST, 80))
        .await
        .map_err(|e| GosError::Privacy(format!("direct connect: {e}")))?;
    let body = http_get(stream, IP_API_HOST, IP_API_PATH, "*/*").await?;
    parse_ip_api(&body)
}
