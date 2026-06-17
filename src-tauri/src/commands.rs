//! Tauri commands — the bridge the UI calls via `invoke`.
//!
//! Each returns `Result<_, String>` so errors surface cleanly in the UI. Settings
//! are persisted after every mutation. Std mutex guards are never held across an
//! `.await`.

use tauri::State;

use gos_bridge::FileEntry;
use gos_core::settings::DohProvider;
use gos_core::types::{CrossoverToggles, LeakReport, PrivacyStatus};
use gos_core::GosSettings;

use crate::state::AppState;

fn persist(state: &AppState) -> Result<GosSettings, String> {
    let snapshot = state.settings.lock().unwrap().clone();
    snapshot
        .save(&state.settings_path)
        .map_err(|e| e.to_string())?;
    Ok(snapshot)
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<PrivacyStatus, String> {
    Ok(state.engine.status().await)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<GosSettings, String> {
    Ok(state.settings.lock().unwrap().clone())
}

#[tauri::command]
pub fn get_node_id(state: State<'_, AppState>) -> String {
    state.node_id.clone()
}

#[tauri::command]
pub async fn start_tor(state: State<'_, AppState>) -> Result<PrivacyStatus, String> {
    let status = state.engine.start_tor().await.map_err(|e| e.to_string())?;
    state.settings.lock().unwrap().tor_enabled = true;
    let _ = persist(&state);
    Ok(status)
}

#[tauri::command]
pub async fn stop_tor(state: State<'_, AppState>) -> Result<PrivacyStatus, String> {
    let status = state.engine.stop_tor().await;
    state.settings.lock().unwrap().tor_enabled = false;
    let _ = persist(&state);
    Ok(status)
}

#[tauri::command]
pub async fn new_identity(state: State<'_, AppState>) -> Result<PrivacyStatus, String> {
    state.engine.new_identity().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_doh(
    state: State<'_, AppState>,
    enabled: bool,
    provider: DohProvider,
) -> Result<PrivacyStatus, String> {
    let status = state.engine.set_doh(enabled, provider).await;
    {
        let mut s = state.settings.lock().unwrap();
        s.doh_enabled = enabled;
        s.doh_provider = provider;
    }
    let _ = persist(&state);
    Ok(status)
}

#[tauri::command]
pub async fn run_leak_test(state: State<'_, AppState>) -> Result<LeakReport, String> {
    Ok(state.engine.leak_test().await)
}

#[tauri::command]
pub fn set_kill_switch(state: State<'_, AppState>, enabled: bool) -> Result<GosSettings, String> {
    state.settings.lock().unwrap().kill_switch = enabled;
    // Real firewall enforcement runs through the privileged helper (later phase).
    persist(&state)
}

#[tauri::command]
pub fn set_mac_randomize(state: State<'_, AppState>, enabled: bool) -> Result<String, String> {
    let iface = state
        .platform
        .primary_interface()
        .map_err(|e| e.to_string())?;
    let mac = if enabled {
        state.platform.randomize_mac(&iface).map_err(|e| e.to_string())?
    } else {
        state.platform.current_mac(&iface).map_err(|e| e.to_string())?
    };
    state.settings.lock().unwrap().mac_randomized = enabled;
    let _ = persist(&state);
    Ok(mac)
}

#[tauri::command]
pub fn set_crossover(
    state: State<'_, AppState>,
    toggles: CrossoverToggles,
) -> Result<GosSettings, String> {
    {
        state.settings.lock().unwrap().crossover = toggles;
        state.bridge.lock().unwrap().set_toggles(toggles);
    }
    persist(&state)
}

#[tauri::command]
pub fn bridge_list_dir(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<FileEntry>, String> {
    state
        .bridge
        .lock()
        .unwrap()
        .list_dir(&path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bridge_launch_app(state: State<'_, AppState>, name: String) -> Result<(), String> {
    state
        .bridge
        .lock()
        .unwrap()
        .launch_app(&name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bridge_list_interfaces(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    state
        .bridge
        .lock()
        .unwrap()
        .list_network_interfaces()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bridge_clipboard_get(state: State<'_, AppState>) -> Result<String, String> {
    if !state.settings.lock().unwrap().crossover.clipboard {
        return Err("clipboard cross-over is turned off".into());
    }
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.get_text().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn bridge_clipboard_set(state: State<'_, AppState>, text: String) -> Result<(), String> {
    if !state.settings.lock().unwrap().crossover.clipboard {
        return Err("clipboard cross-over is turned off".into());
    }
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn home_dir() -> String {
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into())
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME").unwrap_or_else(|_| "/".into())
    }
}
