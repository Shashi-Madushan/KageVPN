mod commands;
mod config;
mod error;
mod logs;
mod net;
mod paths;
mod profile;
mod runner;
mod session;
mod singbox;
mod speed_icon;
mod state;
mod store;
mod tray;
mod traffic;

use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

use crate::paths::AppPaths;
use crate::state::AppState;

const HIDDEN_FLAG: &str = "--hidden";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![HIDDEN_FLAG])))
        .setup(|app| {
            let paths = AppPaths::new(app.path().app_data_dir()?)?;
            app.manage(AppState::new(paths));
            logs::spawn_tailer(app.handle().clone());
            tray::setup(app.handle())?;
            auto_connect(app.handle());

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
            commands::import_profile,
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
            commands::quit_app,
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
            let _ = session::connect(&app).await;
        });
    }
}
