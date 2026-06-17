//! Integration tests for the `gos-bridge` host cross-over API.
//!
//! These verify that each capability genuinely honors its toggle: turning a
//! bridge off must return an `Err`, not silently succeed. Temp directories live
//! under `std::env::temp_dir()` and are cleaned up at the end of each test.

use std::path::PathBuf;
use std::process;

use gos_bridge::HostBridge;
use gos_core::types::CrossoverToggles;

/// Build a unique-ish temp path so parallel test binaries don't collide.
fn unique_temp_path(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let name = format!("gos-bridge-test-{label}-{}-{}", process::id(), nanos);
    std::env::temp_dir().join(name)
}

#[test]
fn all_off_toggles_reject_every_crossover_call() {
    // Arrange — fully isolated bridge plus a real directory to point at.
    let bridge = HostBridge::new(CrossoverToggles::all_off());
    let temp = std::env::temp_dir();
    let temp_str = temp.to_string_lossy().to_string();
    assert!(bridge.toggles().is_isolated());

    // Act / Assert — files: gated, must error even on a valid path.
    let listing = bridge.list_dir(&temp_str);
    assert!(listing.is_err(), "list_dir should be gated by files toggle");

    // Act / Assert — devices: gated, must error.
    let interfaces = bridge.list_network_interfaces();
    assert!(
        interfaces.is_err(),
        "list_network_interfaces should be gated by devices toggle"
    );

    // Act / Assert — launch_apps: gated, must error without ever spawning.
    let launch = bridge.launch_app("Finder");
    assert!(
        launch.is_err(),
        "launch_app should be gated by launch_apps toggle"
    );
}

#[test]
fn set_toggles_updates_reported_state() {
    // Arrange
    let mut bridge = HostBridge::new(CrossoverToggles::all_off());
    assert!(bridge.toggles().is_isolated());

    // Act
    bridge.set_toggles(CrossoverToggles::default());

    // Assert
    assert!(!bridge.toggles().is_isolated());
    assert!(bridge.toggles().files);
}

#[test]
fn default_toggles_list_dir_returns_created_entry() {
    // Arrange — a fresh temp subdir containing exactly one known file.
    let dir = unique_temp_path("listdir");
    std::fs::create_dir_all(&dir).expect("temp subdir should be creatable");
    let file_path = dir.join("marker.txt");
    std::fs::write(&file_path, b"hello").expect("marker file should be writable");

    let bridge = HostBridge::new(CrossoverToggles::default());
    let dir_str = dir.to_string_lossy().to_string();

    // Act
    let entries = bridge
        .list_dir(&dir_str)
        .expect("list_dir should succeed when files toggle is on");

    // Assert — the entry we created shows up in the listing.
    assert!(!entries.is_empty(), "listing should be non-empty");
    let found = entries.iter().find(|e| e.name == "marker.txt");
    assert!(found.is_some(), "created marker.txt should appear in listing");
    let marker = found.unwrap();
    assert!(!marker.is_dir, "marker.txt should not be a directory");
    assert_eq!(marker.size_bytes, 5, "marker.txt should be 5 bytes");

    // Cleanup
    let _ = std::fs::remove_file(&file_path);
    let _ = std::fs::remove_dir(&dir);
}

#[cfg(target_os = "macos")]
#[test]
fn default_toggles_list_network_interfaces_is_non_empty_on_macos() {
    // Arrange
    let bridge = HostBridge::new(CrossoverToggles::default());

    // Act
    let interfaces = bridge
        .list_network_interfaces()
        .expect("list_network_interfaces should succeed when devices toggle is on");

    // Assert — every Mac has at least a loopback interface.
    assert!(
        !interfaces.is_empty(),
        "macOS should report at least one network interface"
    );
}
