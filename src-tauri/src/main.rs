// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Windows starts the helper service with this argument; it is the same program as the app.
    #[cfg(windows)]
    if std::env::args().any(|a| a == "--service") {
        let _ = kage_vpn_lib::run_service();
        return;
    }
    kage_vpn_lib::run()
}
