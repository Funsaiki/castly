mod commands;
mod error;
mod state;

mod adb;
mod airplay;
mod discovery;
mod input;
mod pipeline;
mod video;

use state::AppState;
use std::sync::Arc;
use tauri::Manager;

pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::device::list_devices,
            commands::device::scan_devices,
            commands::device::connect_device,
            commands::device::disconnect_device,
            commands::device::connect_wifi,
            commands::device::connect_wifi_ip,
            commands::device::pair_wifi,
            commands::stream::start_mirror,
            commands::stream::stop_mirror,
            commands::control::inject_touch,
            commands::control::inject_key,
            commands::control::inject_scroll,
            commands::control::press_back,
            commands::control::press_home,
            commands::control::press_recent,
            commands::control::set_screen_power,
        ])
        .setup(|app| {
            // Log to file in AppData so logs are always accessible
            let log_dir = directories::BaseDirs::new()
                .map(|d| d.data_local_dir().join("Castly"))
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            let _ = std::fs::create_dir_all(&log_dir);
            let log_file = std::fs::File::create(log_dir.join("castly.log"))
                .unwrap_or_else(|_| std::fs::File::create("castly.log").unwrap());

            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "castly=debug".into()),
                )
                .with_writer(std::sync::Mutex::new(log_file))
                .with_ansi(false)
                .init();
            tracing::info!("Castly starting up — log file: {}", log_dir.join("castly.log").display());

            // Start AirPlay mDNS advertiser
            let mut advertiser = airplay::mdns::AirPlayAdvertiser::new();
            match advertiser.start() {
                Ok(()) => tracing::info!("AirPlay advertiser started"),
                Err(e) => tracing::warn!("Failed to start AirPlay advertiser: {}", e),
            }
            std::mem::forget(advertiser);

            // Start AirPlay RTSP server
            let app_handle = app.handle().clone();
            let airplay_state: Arc<AppState> = app.state::<Arc<AppState>>().inner().clone();
            let rtsp_server = airplay::rtsp::RtspServer::new();
            match rtsp_server.start(app_handle, airplay_state) {
                Ok(()) => tracing::info!("AirPlay RTSP server started"),
                Err(e) => tracing::warn!("Failed to start AirPlay RTSP server: {}", e),
            }
            std::mem::forget(rtsp_server);

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Castly")
        .run(|_app, event| {
            if let tauri::RunEvent::Exit = event {
                // Kill adb server on exit so it doesn't lock files for the installer
                let adb = adb::client::AdbClient::new();
                let mut cmd = std::process::Command::new(adb.adb_path());
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(0x08000000);
                }
                let _ = cmd.arg("kill-server").output();
                tracing::info!("ADB server killed on exit");
            }
        });
}
