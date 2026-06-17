//! One-time install of the process-wide rustls crypto provider.
//!
//! rustls 0.23 (used by both our DoH client and arti) panics if no default
//! provider is selected. We pick `ring`. Safe to call repeatedly.

use std::sync::Once;

pub fn ensure_crypto_provider() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}
