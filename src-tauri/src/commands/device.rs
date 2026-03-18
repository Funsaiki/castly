use std::sync::Arc;
use tauri::State;

use crate::adb::client::AdbClient;
use crate::discovery::adb_scanner;
use crate::error::{AppError, AppResult};
use crate::state::{AppState, DeviceInfo};

#[tauri::command]
pub async fn list_devices(state: State<'_, Arc<AppState>>) -> AppResult<Vec<DeviceInfo>> {
    let devices = state.devices.read();
    Ok(devices.values().cloned().collect())
}

#[tauri::command]
pub async fn scan_devices(state: State<'_, Arc<AppState>>) -> AppResult<Vec<DeviceInfo>> {
    let state_inner = state.inner().clone();
    Ok(adb_scanner::scan_once(&state_inner))
}

#[tauri::command]
pub async fn connect_device(
    device_id: String,
    state: State<'_, Arc<AppState>>,
) -> AppResult<DeviceInfo> {
    let devices = state.devices.read();
    match devices.get(&device_id) {
        Some(device) => Ok(device.clone()),
        None => Err(AppError::DeviceNotFound(device_id)),
    }
}

#[tauri::command]
pub async fn disconnect_device(
    device_id: String,
    state: State<'_, Arc<AppState>>,
) -> AppResult<()> {
    let mut sessions = state.sessions.write();
    sessions.remove(&device_id);
    Ok(())
}

/// Switch a USB-connected device to TCP/IP mode and connect via Wi-Fi
#[tauri::command]
pub async fn connect_wifi(
    device_id: String,
    state: State<'_, Arc<AppState>>,
) -> AppResult<String> {
    let adb = AdbClient::new();

    // Get the device's IP address
    let ip = adb
        .get_device_ip(&device_id)?
        .ok_or_else(|| AppError::Connection("Could not find device IP. Make sure it's on Wi-Fi.".into()))?;

    // Switch to TCP/IP mode
    adb.tcpip(&device_id, 5555)?;

    // Wait for the device to restart in TCP mode
    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Connect via Wi-Fi
    adb.connect_tcp(&ip, 5555)?;

    // Scan to pick up the new Wi-Fi device
    let state_inner = state.inner().clone();
    adb_scanner::scan_once(&state_inner);

    Ok(ip)
}

/// Connect to a device directly by IP address
#[tauri::command]
pub async fn connect_wifi_ip(
    ip: String,
    port: Option<u16>,
    state: State<'_, Arc<AppState>>,
) -> AppResult<()> {
    let adb = AdbClient::new();
    let port = port.unwrap_or(5555);

    adb.connect_tcp(&ip, port)?;

    // Scan to pick up the new device
    let state_inner = state.inner().clone();
    adb_scanner::scan_once(&state_inner);

    Ok(())
}

/// Pair with a device using wireless debugging pairing code, then connect
#[tauri::command]
pub async fn pair_wifi(
    ip: String,
    pair_port: u16,
    code: String,
    connect_port: u16,
    state: State<'_, Arc<AppState>>,
) -> AppResult<()> {
    let adb = AdbClient::new();

    // Step 1: Pair
    adb.pair(&ip, pair_port, &code)?;

    // Step 2: Connect
    std::thread::sleep(std::time::Duration::from_millis(1000));
    adb.connect_tcp(&ip, connect_port)?;

    // Scan to pick up the new device
    let state_inner = state.inner().clone();
    adb_scanner::scan_once(&state_inner);

    Ok(())
}
