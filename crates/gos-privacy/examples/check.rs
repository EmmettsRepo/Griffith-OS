//! Quick functional check of the privacy engine, runnable from the CLI:
//!
//!   cargo run -p gos-privacy --example check                 # DoH + real IP
//!   cargo run -p gos-privacy --example check -- --tor        # also bootstrap Tor
//!
//! Prints a leak report as JSON so we can verify behavior without the full app.

use gos_privacy::PrivacyEngine;

#[tokio::main]
async fn main() {
    let want_tor = std::env::args().any(|a| a == "--tor");
    let engine = PrivacyEngine::new("example-node".into());

    if want_tor {
        #[cfg(feature = "tor")]
        {
            eprintln!("bootstrapping Tor (can take ~20-40s)…");
            match engine.start_tor().await {
                Ok(s) => eprintln!(
                    "Tor connected — exit {:?} ({:?})",
                    s.exit_ip, s.exit_country
                ),
                Err(e) => eprintln!("Tor failed: {e}"),
            }
        }
        #[cfg(not(feature = "tor"))]
        eprintln!("built without the `tor` feature; skipping Tor");
    }

    let report = engine.leak_test().await;
    println!("{}", serde_json::to_string_pretty(&report).unwrap());
}
