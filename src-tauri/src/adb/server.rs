use crate::adb::client::AdbClient;
use crate::error::{AppError, AppResult};
use std::process::{Child, Command, Stdio};
use tracing::{debug, info};

const SCRCPY_SERVER_PATH: &str = "/data/local/tmp/scrcpy-server.jar";
const SCRCPY_SERVER_VERSION: &str = "2.7";

/// Configuration for the scrcpy server
#[derive(Debug, Clone)]
pub struct ScrcpyConfig {
    pub max_size: u32,
    pub bit_rate: u32,
    pub max_fps: u32,
    pub codec: String,
}

impl Default for ScrcpyConfig {
    fn default() -> Self {
        Self {
            max_size: 1920,
            bit_rate: 8_000_000,
            max_fps: 60,
            codec: "h264".to_string(),
        }
    }
}

/// Manages the scrcpy server lifecycle on an Android device
pub struct ScrcpyServer {
    adb: AdbClient,
    serial: String,
    process: Option<Child>,
}

impl ScrcpyServer {
    pub fn new(adb: AdbClient, serial: String) -> Self {
        Self {
            adb,
            serial,
            process: None,
        }
    }

    /// Push the scrcpy server JAR to the device
    pub fn push_server(&self, local_jar_path: &str) -> AppResult<()> {
        info!("Pushing scrcpy server to {}", self.serial);
        self.adb
            .push_file(&self.serial, local_jar_path, SCRCPY_SERVER_PATH)?;
        debug!("scrcpy server pushed successfully");
        Ok(())
    }

    /// Launch the scrcpy server on the device
    pub fn start(&mut self, config: &ScrcpyConfig) -> AppResult<()> {
        info!(
            "Starting scrcpy server on {} ({}x{} @ {}fps, {}bps)",
            self.serial, config.max_size, config.max_size, config.max_fps, config.bit_rate
        );

        let child = Command::new("adb")
            .args([
                "-s",
                &self.serial,
                "shell",
                &format!(
                    "CLASSPATH={} app_process / com.genymobile.scrcpy.Server {} \
                     log_level=info max_size={} video_bit_rate={} max_fps={} \
                     video_codec={} tunnel_forward=true audio=false control=true \
                     send_frame_meta=true",
                    SCRCPY_SERVER_PATH,
                    SCRCPY_SERVER_VERSION,
                    config.max_size,
                    config.bit_rate,
                    config.max_fps,
                    config.codec,
                ),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Adb(format!("Failed to start scrcpy server: {}", e)))?;

        self.process = Some(child);
        info!("scrcpy server process started");
        Ok(())
    }

    /// Stop the scrcpy server
    pub fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill();
            let _ = child.wait();
            info!("scrcpy server stopped on {}", self.serial);
        }
    }

    pub fn is_running(&self) -> bool {
        self.process.is_some()
    }
}

impl Drop for ScrcpyServer {
    fn drop(&mut self) {
        self.stop();
    }
}
