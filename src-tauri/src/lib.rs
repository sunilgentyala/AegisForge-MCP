mod commands;
mod error;
mod plugins;
mod state;

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aegisforge_core_lib=info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(state::AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::recon::scan_ports,
            commands::recon::get_scan_status,
        ])
        .run(tauri::generate_context!())
        .expect("fatal: tauri runtime failed to start");
}
