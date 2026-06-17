//! gos-platform — the one place OS-specific behavior lives.
//!
//! A single `PlatformOps` trait, implemented per-OS behind `#[cfg(target_os)]`.
//! This is what makes "a different version of GOS per OS" one codebase: the app
//! calls the trait, and the right implementation is compiled in per target.

pub mod traits;
pub use traits::PlatformOps;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

/// The `PlatformOps` implementation for the OS this build targets.
pub fn current() -> Box<dyn PlatformOps> {
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOs::default())
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::Linux::default())
    }
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::Windows::default())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Box::new(traits::Unsupported)
    }
}

/// Run an external command, returning stdout or a descriptive error.
pub(crate) fn run(cmd: &str, args: &[&str]) -> gos_core::types::Result<String> {
    use gos_core::types::GosError;
    let out = std::process::Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| GosError::Platform(format!("{cmd}: {e}")))?;
    if !out.status.success() {
        return Err(GosError::Platform(format!(
            "{cmd} {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// A random locally-administered, unicast MAC address (e.g. for randomization).
pub(crate) fn random_mac() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut b = [0u8; 6];
    rng.fill(&mut b);
    // Set locally-administered bit, clear multicast bit on the first octet.
    b[0] = (b[0] | 0b0000_0010) & 0b1111_1110;
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        b[0], b[1], b[2], b[3], b[4], b[5]
    )
}
