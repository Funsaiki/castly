use crate::error::{AppError, AppResult};
use crate::state::{ConnectionType, DeviceInfo, DeviceStatus, DeviceType};
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, error, info};

pub struct AdbClient {
    adb_path: String,
}

impl AdbClient {
    pub fn new() -> Self {
        let adb_path = Self::find_adb().unwrap_or_else(|| "adb".to_string());
        info!("Using ADB at: {}", adb_path);
        Self { adb_path }
    }

    pub fn with_path(adb_path: String) -> Self {
        Self { adb_path }
    }

    pub fn adb_path(&self) -> &str {
        &self.adb_path
    }

    /// Try to find ADB in common locations
    fn find_adb() -> Option<String> {
        // 1. Check bundled resources next to the executable (Tauri release)
        if let Ok(exe_dir) = std::env::current_exe() {
            if let Some(dir) = exe_dir.parent() {
                // Tauri bundles resources in resources/ subfolder
                for sub in &["resources/platform-tools", "platform-tools"] {
                    let path = dir.join(sub).join("adb.exe");
                    if path.exists() {
                        return Some(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        // 2. Check project-level platform-tools (dev mode)
        let dev_paths = [
            PathBuf::from("../platform-tools/adb.exe"),
            PathBuf::from("../../platform-tools/adb.exe"),
            PathBuf::from("platform-tools/adb.exe"),
        ];
        for path in &dev_paths {
            if path.exists() {
                if let Ok(abs) = std::fs::canonicalize(path) {
                    return Some(abs.to_string_lossy().to_string());
                }
            }
        }

        // 3. Check ANDROID_HOME / ANDROID_SDK_ROOT
        for var in &["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
            if let Ok(sdk) = std::env::var(var) {
                let path = PathBuf::from(&sdk)
                    .join("platform-tools")
                    .join("adb.exe");
                if path.exists() {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }

        // 4. Check if adb is on PATH
        if Command::new("adb").arg("version").output().is_ok() {
            return Some("adb".to_string());
        }

        None
    }

    /// List all connected ADB devices
    pub fn list_devices(&self) -> AppResult<Vec<DeviceInfo>> {
        let output = Command::new(&self.adb_path)
            .args(["devices", "-l"])
            .output()
            .map_err(|e| AppError::Adb(format!("Failed to run adb: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Adb(format!("adb devices failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let devices = Self::parse_device_list(&stdout);
        debug!("Found {} ADB devices", devices.len());
        Ok(devices)
    }

    fn parse_device_list(output: &str) -> Vec<DeviceInfo> {
        output
            .lines()
            .skip(1)
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return None;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 2 || parts[1] != "device" {
                    return None;
                }

                let serial = parts[0].to_string();
                let is_wifi = serial.contains(':');

                let name = parts
                    .iter()
                    .find(|p| p.starts_with("model:"))
                    .map(|p| p.trim_start_matches("model:").replace('_', " "))
                    .unwrap_or_else(|| serial.clone());

                Some(DeviceInfo {
                    id: serial,
                    name,
                    device_type: DeviceType::Android,
                    connection: if is_wifi {
                        ConnectionType::Wifi
                    } else {
                        ConnectionType::Usb
                    },
                    status: DeviceStatus::Connected,
                    screen_width: 0,
                    screen_height: 0,
                })
            })
            .collect()
    }

    /// Push a file to the device
    pub fn push_file(&self, serial: &str, local: &str, remote: &str) -> AppResult<()> {
        info!("Pushing {} to {}:{}", local, serial, remote);
        let output = Command::new(&self.adb_path)
            .args(["-s", serial, "push", local, remote])
            .output()
            .map_err(|e| AppError::Adb(format!("Failed to push file: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Adb(format!("adb push failed: {}", stderr)));
        }
        Ok(())
    }

    /// Execute a shell command on the device
    pub fn shell(&self, serial: &str, command: &[&str]) -> AppResult<String> {
        let mut args = vec!["-s", serial, "shell"];
        args.extend(command);

        let output = Command::new(&self.adb_path)
            .args(&args)
            .output()
            .map_err(|e| AppError::Adb(format!("Failed to run shell command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("adb shell failed: {}", stderr);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Set up ADB port forwarding
    pub fn forward(&self, serial: &str, local: &str, remote: &str) -> AppResult<()> {
        let output = Command::new(&self.adb_path)
            .args(["-s", serial, "forward", local, remote])
            .output()
            .map_err(|e| AppError::Adb(format!("Failed to forward port: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Adb(format!("adb forward failed: {}", stderr)));
        }
        Ok(())
    }

    /// Remove port forwarding
    pub fn forward_remove(&self, serial: &str, local: &str) -> AppResult<()> {
        let _ = Command::new(&self.adb_path)
            .args(["-s", serial, "forward", "--remove", local])
            .output();
        Ok(())
    }

    /// Pair with a device using wireless debugging code
    pub fn pair(&self, ip: &str, port: u16, code: &str) -> AppResult<()> {
        let addr = format!("{}:{}", ip, port);
        info!("Pairing with {}", addr);

        let output = Command::new(&self.adb_path)
            .args(["pair", &addr, code])
            .output()
            .map_err(|e| AppError::Adb(format!("Failed to pair: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        if combined.contains("Successfully paired") {
            info!("Paired with {}", addr);
            Ok(())
        } else {
            Err(AppError::Connection(format!(
                "Pairing failed: {}",
                combined.trim()
            )))
        }
    }

    /// Connect to a device over Wi-Fi
    pub fn connect_tcp(&self, ip: &str, port: u16) -> AppResult<()> {
        let addr = format!("{}:{}", ip, port);
        info!("Connecting to {}", addr);

        let output = Command::new(&self.adb_path)
            .args(["connect", &addr])
            .output()
            .map_err(|e| AppError::Adb(format!("Failed to connect: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("cannot connect") || stdout.contains("failed") {
            return Err(AppError::Connection(format!(
                "Could not connect to {}: {}",
                addr, stdout
            )));
        }

        info!("Connected to {}", addr);
        Ok(())
    }

    /// Switch device to TCP/IP mode
    pub fn tcpip(&self, serial: &str, port: u16) -> AppResult<()> {
        let output = Command::new(&self.adb_path)
            .args(["-s", serial, "tcpip", &port.to_string()])
            .output()
            .map_err(|e| AppError::Adb(format!("Failed to switch to tcpip: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Adb(format!("adb tcpip failed: {}", stderr)));
        }
        Ok(())
    }

    /// Get the IP address of a USB-connected device
    pub fn get_device_ip(&self, serial: &str) -> AppResult<Option<String>> {
        let output = self.shell(serial, &["ip", "route"])?;
        for line in output.lines() {
            if let Some(idx) = line.find("src ") {
                let ip_start = idx + 4;
                let ip: String = line[ip_start..]
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                if !ip.is_empty() {
                    return Ok(Some(ip));
                }
            }
        }
        Ok(None)
    }
}
