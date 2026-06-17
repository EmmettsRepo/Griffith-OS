//! Per-install node identity — an ed25519 keypair persisted to disk.
//!
//! The public key is this GOS install's `node_id`, used later for authenticated,
//! end-to-end-encrypted GOS<->GOS transfer. Generated once, reused thereafter.

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::path::Path;

use crate::types::{GosError, Result};

pub struct NodeIdentity {
    signing: SigningKey,
}

impl NodeIdentity {
    /// Load the keypair from `path`, or generate + persist a new one if absent.
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            let raw = std::fs::read(path)?;
            if raw.len() != 32 {
                return Err(GosError::Other(format!(
                    "identity file {} is corrupt ({} bytes)",
                    path.display(),
                    raw.len()
                )));
            }
            let mut bytes = [0u8; 32];
            bytes.copy_from_slice(&raw);
            Ok(NodeIdentity {
                signing: SigningKey::from_bytes(&bytes),
            })
        } else {
            let signing = SigningKey::generate(&mut OsRng);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, signing.to_bytes())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
            }
            Ok(NodeIdentity { signing })
        }
    }

    /// Hex-encoded public key — the stable id for this install.
    pub fn node_id(&self) -> String {
        hex::encode(self.signing.verifying_key().to_bytes())
    }

    /// A short, human-friendly form of the node id for the UI.
    pub fn short_id(&self) -> String {
        let full = self.node_id();
        format!("{}…{}", &full[..6], &full[full.len() - 4..])
    }
}
