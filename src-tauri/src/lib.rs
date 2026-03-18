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
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
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
            commands::recording::start_recording,
            commands::recording::stop_recording,
            commands::recording::take_screenshot,
        ])
        .setup(|app| {
            tracing_subscriber::fmt()
                .with_env_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "phone_mirror=debug".into()),
                )
                .init();
            tracing::info!("Phone Mirror starting up");

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
        .run(tauri::generate_context!())
        .expect("error while running Phone Mirror");
}
