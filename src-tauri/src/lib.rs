mod clash;
mod commands;
mod config;
mod core;
mod diagnostics;
mod error;
mod fsutil;
mod logs;
mod net;
mod notify;
mod paths;
mod profile;
mod reliability;
mod runner;
mod session;
mod settings;
mod singbox;
mod speed_icon;
mod state;
mod store;
mod subscriptions;
mod tray;
mod traffic;
mod usage;

use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_deep_link::DeepLinkExt;
use tauri_plugin_autostart::MacosLauncher;

use crate::paths::AppPaths;
use crate::state::AppState;

const HIDDEN_FLAG: &str = "--hidden";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut app = tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        tray::toggle_connection(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![HIDDEN_FLAG])))
        .setup(|app| {
            let paths = AppPaths::new(app.path().app_data_dir()?)?;
            app.manage(AppState::new(paths));
            notify::setup(app.handle());
            warn_if_data_was_recovered(app.handle());
            logs::spawn_tailer(app.handle().clone());
            tray::setup(app.handle())?;
            auto_connect(app.handle());
            core::commands::spawn_background_check(app.handle().clone());
            usage::collector::spawn_maintenance(app.handle().clone());
            subscriptions::spawn_scheduler(app.handle().clone());
            reliability::apply_hotkey(app.handle());
            register_deep_links(app.handle());

            if !std::env::args().any(|arg| arg == HIDDEN_FLAG) {
                tray::show_main_window(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            tray::handle_window_event(window, event);
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::snapshot,
            commands::import_text,
            commands::save_profile,
            commands::duplicate_profile,
            commands::set_favorite,
            commands::set_group,
            commands::reorder_profiles,
            commands::profile_link,
            commands::profile_qr,
            commands::set_auto,
            commands::test_all,
            commands::set_routing,
            commands::set_reliability,
            commands::restore_network,
            commands::add_subscription,
            commands::refresh_subscription,
            commands::remove_subscription,
            commands::run_diagnostics,
            commands::run_speed_test,
            commands::cancel_speed_test,
            commands::connections_list,
            commands::close_connection,
            commands::close_all_connections,
            usage::commands::usage_report,
            usage::commands::usage_top,
            usage::commands::usage_export,
            usage::commands::usage_get_settings,
            usage::commands::usage_set_settings,
            usage::commands::usage_quota,
            usage::commands::usage_reset,
            usage::commands::usage_clear_details,
            usage::commands::latency_history,
            commands::delete_profile,
            commands::select_profile,
            commands::set_mode,
            commands::trust_key,
            commands::clear_logs,
            commands::connect,
            commands::disconnect,
            commands::ping_profile,
            commands::test_connection,
            commands::exit_ip,
            commands::rename_profile,
            commands::update_settings,
            commands::set_launch_at_login,
            commands::open_main_window,
            commands::hide_tray,
            commands::notify_os,
            commands::quit_app,
            core::commands::core_status,
            core::commands::core_check,
            core::commands::core_install,
            core::commands::core_use,
            core::commands::core_remove,
            core::commands::update_core_settings,
            core::commands::core_brew_upgrade,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    app.run(|_, _| {});
}

fn auto_connect(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    let wanted = {
        let store = state.store.lock().unwrap();
        store.settings().auto_connect && store.active_profile().is_some()
    };
    if wanted {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(error) = session::connect(&app).await {
                notify::send(&app, "Connection failed", &error.to_string());
            }
        });
    }
}

fn register_deep_links(app: &tauri::AppHandle) {
    let handle = app.clone();
    app.deep_link().on_open_url(move |event| {
        for url in event.urls() {
            handle_deep_link(&handle, url.as_str());
        }
    });
}

/// `singboxgui://connect|disconnect|toggle` act directly; `import?url=` only asks
/// the UI to confirm, so a link can never add a profile silently.
fn handle_deep_link(app: &tauri::AppHandle, raw: &str) {
    let Ok(url) = url::Url::parse(raw) else { return };
    match url.host_str().unwrap_or_default() {
        "connect" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = session::connect(&app).await {
                    notify::send(&app, "Connection failed", &error.to_string());
                }
            });
        }
        "disconnect" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = session::disconnect(&app).await;
            });
        }
        "toggle" => tray::toggle_connection(app),
        "import" => {
            if let Some((_, value)) = url.query_pairs().find(|(key, _)| key == "url") {
                tray::show_main_window(app);
                let _ = app.emit("import-request", value.into_owned());
            }
        }
        _ => {}
    }
}

/// Tells the user when the saved servers could not be read and were set aside.
fn warn_if_data_was_recovered(app: &tauri::AppHandle) {
    let recovered = app.state::<AppState>().store.lock().unwrap().take_recovery();
    if let Some(path) = recovered {
        notify::send(
            app,
            "Saved servers could not be read",
            &format!("The file was kept as {} so it can be recovered.", path.display()),
        );
    }
}
