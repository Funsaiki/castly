use crate::adb::client::AdbClient;
use crate::state::AppState;
use std::sync::Arc;
use tracing::{debug, error};

/// Single scan for ADB devices, updating the app state
pub fn scan_once(state: &Arc<AppState>) -> Vec<crate::state::DeviceInfo> {
    let adb = AdbClient::new();

    match adb.list_devices() {
        Ok(devices) => {
            let mut state_devices = state.devices.write();
            let previous_ids: Vec<String> = state_devices.keys().cloned().collect();
            let current_ids: Vec<String> = devices.iter().map(|d| d.id.clone()).collect();

            for device in &devices {
                if !state_devices.contains_key(&device.id) {
                    debug!("New device detected: {} ({})", device.name, device.id);
                }
                state_devices.insert(device.id.clone(), device.clone());
            }

            for id in &previous_ids {
                if !current_ids.contains(id) {
                    debug!("Device disconnected: {}", id);
                    state_devices.remove(id);
                }
            }

            devices
        }
        Err(e) => {
            error!("ADB scan failed: {}", e);
            Vec::new()
        }
    }
}
