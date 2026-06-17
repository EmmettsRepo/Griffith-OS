//! Griffith OS — Tauri application entry point.
//!
//! Builds shared state (identity, settings, privacy engine, platform, bridge),
//! wires the command handlers, and installs a tray icon. v1 ships as a tray /
//! menu-bar "extension"; later phases grow the full-screen environment.

mod commands;
mod state;

use std::sync::{Arc, Mutex};

use tauri::Manager;

use gos_bridge::HostBridge;
use gos_core::{GosSettings, NodeIdentity};
use gos_privacy::PrivacyEngine;

use state::AppState;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let config_dir = app
                .path()
                .app_config_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            std::fs::create_dir_all(&config_dir).ok();

            let settings_path = config_dir.join("settings.json");
            let identity_path = config_dir.join("gos-identity.key");

            let identity = NodeIdentity::load_or_create(&identity_path)
                .map_err(|e| format!("identity init failed: {e}"))?;
            let node_id = identity.node_id();

            let settings = GosSettings::load(&settings_path).unwrap_or_default();
            let engine = Arc::new(PrivacyEngine::new(node_id.clone()));

            // Apply persisted DoH preference to the engine on startup.
            {
                let engine = engine.clone();
                let (enabled, provider) = (settings.doh_enabled, settings.doh_provider);
                tauri::async_runtime::spawn(async move {
                    engine.set_doh(enabled, provider).await;
                });
            }

            let bridge = HostBridge::new(settings.crossover);

            app.manage(AppState {
                engine,
                platform: gos_platform::current(),
                bridge: Mutex::new(bridge),
                settings: Mutex::new(settings),
                settings_path,
                node_id,
            });

            build_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::get_settings,
            commands::get_node_id,
            commands::start_tor,
            commands::stop_tor,
            commands::new_identity,
            commands::set_doh,
            commands::run_leak_test,
            commands::set_kill_switch,
            commands::set_mac_randomize,
            commands::set_crossover,
            commands::bridge_list_dir,
            commands::bridge_launch_app,
            commands::bridge_list_interfaces,
            commands::bridge_clipboard_get,
            commands::bridge_clipboard_set,
            commands::home_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Griffith OS");
}

fn build_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    use tauri::tray::TrayIconBuilder;

    let show = MenuItemBuilder::with_id("show", "Open GOS").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit GOS").build(app)?;
    let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

    let mut builder = TrayIconBuilder::new().menu(&menu).tooltip("Griffith OS");
    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }
    builder
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}
