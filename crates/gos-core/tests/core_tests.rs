//! Integration tests for the `gos-core` public API.
//!
//! These exercise the crate strictly through its published surface: settings
//! persistence, node identity, crossover toggles, timestamps, and status.
//! Temp files live under `std::env::temp_dir()` and are cleaned up at the end.

use std::path::PathBuf;
use std::process;

use gos_core::identity::NodeIdentity;
use gos_core::settings::{DohProvider, GosSettings};
use gos_core::types::{CrossoverToggles, PrivacyStatus};

/// Build a unique-ish temp path so parallel test binaries don't collide.
fn unique_temp_path(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let name = format!("gos-core-test-{label}-{}-{}.tmp", process::id(), nanos);
    std::env::temp_dir().join(name)
}

#[test]
fn crossover_toggles_default_is_not_isolated() {
    // Arrange
    let toggles = CrossoverToggles::default();

    // Act
    let isolated = toggles.is_isolated();

    // Assert — defaults turn every host bridge on.
    assert!(!isolated, "default toggles should not be isolated");
    assert!(toggles.files);
    assert!(toggles.launch_apps);
    assert!(toggles.clipboard);
    assert!(toggles.devices);
}

#[test]
fn crossover_toggles_all_off_is_isolated() {
    // Arrange
    let toggles = CrossoverToggles::all_off();

    // Act
    let isolated = toggles.is_isolated();

    // Assert — all bridges severed means full isolation.
    assert!(isolated, "all_off toggles should be isolated");
    assert!(!toggles.files);
    assert!(!toggles.launch_apps);
    assert!(!toggles.clipboard);
    assert!(!toggles.devices);
}

#[test]
fn crossover_is_isolated_reflects_partial_state() {
    // Arrange — start fully off, then re-enable a single bridge.
    let mut toggles = CrossoverToggles::all_off();
    assert!(toggles.is_isolated());

    // Act
    toggles.clipboard = true;

    // Assert — any one bridge being on breaks isolation.
    assert!(
        !toggles.is_isolated(),
        "enabling a single bridge should break isolation"
    );
}

#[test]
fn gos_settings_round_trip_through_disk_preserves_fields() {
    // Arrange
    let path = unique_temp_path("settings-roundtrip");
    let original = GosSettings::default();

    // Act — save then load back from the same path.
    original.save(&path).expect("settings should save");
    let loaded = GosSettings::load(&path).expect("settings should load");

    // Assert — key fields survive the JSON round-trip intact.
    assert_eq!(loaded.doh_enabled, original.doh_enabled);
    assert_eq!(loaded.kill_switch, original.kill_switch);
    assert_eq!(loaded.tor_enabled, original.tor_enabled);
    assert_eq!(loaded.mac_randomized, original.mac_randomized);
    assert_eq!(loaded.doh_provider, original.doh_provider);
    assert_eq!(loaded.doh_provider, DohProvider::Cloudflare);
    assert_eq!(loaded.theme_accent, original.theme_accent);
    assert_eq!(
        loaded.crossover.is_isolated(),
        original.crossover.is_isolated()
    );

    // Cleanup
    let _ = std::fs::remove_file(&path);
}

#[test]
fn gos_settings_load_on_missing_path_returns_defaults() {
    // Arrange — a path that is guaranteed not to exist.
    let path = unique_temp_path("settings-missing");
    assert!(!path.exists(), "precondition: temp path must not exist");

    // Act
    let loaded = GosSettings::load(&path).expect("missing path should yield defaults");

    // Assert — falls back to defaults rather than erroring.
    let defaults = GosSettings::default();
    assert_eq!(loaded.doh_enabled, defaults.doh_enabled);
    assert_eq!(loaded.kill_switch, defaults.kill_switch);
    assert_eq!(loaded.doh_provider, defaults.doh_provider);
    assert_eq!(loaded.theme_accent, defaults.theme_accent);
}

#[test]
fn node_identity_persists_same_node_id_across_loads() {
    // Arrange
    let path = unique_temp_path("identity");
    assert!(!path.exists(), "precondition: identity path must not exist");

    // Act — first call generates + persists, second reuses the stored key.
    let first = NodeIdentity::load_or_create(&path).expect("first load_or_create should succeed");
    let first_id = first.node_id();
    let second = NodeIdentity::load_or_create(&path).expect("second load_or_create should succeed");
    let second_id = second.node_id();

    // Assert — stable identity and a well-formed 64-char hex id.
    assert_eq!(first_id, second_id, "node id must persist across loads");
    assert_eq!(first_id.len(), 64, "node id should be 64 hex chars");
    assert!(
        first_id.chars().all(|c| c.is_ascii_hexdigit()),
        "node id should be hex-encoded"
    );
    assert!(
        first.short_id().contains('…'),
        "short_id should be an abbreviated form"
    );

    // Cleanup
    let _ = std::fs::remove_file(&path);
}

#[test]
fn now_ms_returns_positive_timestamp() {
    // Act
    let ms = gos_core::now_ms();

    // Assert — a real epoch timestamp is well past zero.
    assert!(ms > 0, "now_ms should return a positive epoch timestamp");
}

#[test]
fn privacy_status_new_sets_node_id_and_secure_defaults() {
    // Arrange / Act
    let status = PrivacyStatus::new("abc".to_string());

    // Assert — node id carried through, DoH on by default.
    assert_eq!(status.node_id, "abc");
    assert!(status.doh_enabled, "DoH should default to enabled");
    assert!(status.kill_switch, "kill switch should default to enabled");
    assert!(!status.tor_enabled, "Tor should default to disabled");
}
