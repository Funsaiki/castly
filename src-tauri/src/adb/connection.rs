use crate::adb::client::AdbClient;
use crate::error::{AppError, AppResult};
use bytes::BytesMut;
use std::net::TcpStream;
use std::io::Read;
use tracing::{debug, info};

/// Information about the connected device, received from scrcpy server
#[derive(Debug, Clone)]
pub struct DeviceScreenInfo {
    pub device_name: String,
    pub width: u32,
    pub height: u32,
}

/// Manages the socket connections to the scrcpy server
pub struct ScrcpyConnection {
    pub video_stream: Option<TcpStream>,
    pub control_stream: Option<TcpStream>,
    pub screen_info: Option<DeviceScreenInfo>,
    local_port: u16,
}

impl ScrcpyConnection {
    /// Set up port forwarding and connect to the scrcpy server sockets
    pub fn connect(adb: &AdbClient, serial: &str, local_port: u16) -> AppResult<Self> {
        // Set up port forwarding for the scrcpy server
        let local = format!("tcp:{}", local_port);
        let remote = "localabstract:scrcpy";
        adb.forward(serial, &local, remote)?;

        info!(
            "Port forwarding established: {} -> {} on {}",
            local, remote, serial
        );

        // Connect to the video socket
        let video_stream = TcpStream::connect(format!("127.0.0.1:{}", local_port))
            .map_err(|e| AppError::Connection(format!("Failed to connect video socket: {}", e)))?;

        // Connect to the control socket (second connection)
        let control_stream = TcpStream::connect(format!("127.0.0.1:{}", local_port))
            .map_err(|e| {
                AppError::Connection(format!("Failed to connect control socket: {}", e))
            })?;

        info!("Connected to scrcpy server sockets on port {}", local_port);

        let mut conn = Self {
            video_stream: Some(video_stream),
            control_stream: Some(control_stream),
            screen_info: None,
            local_port,
        };

        // Read the device info header from the video socket
        conn.read_device_info()?;

        Ok(conn)
    }

    /// Read the initial device info from the video socket
    fn read_device_info(&mut self) -> AppResult<()> {
        let stream = self
            .video_stream
            .as_mut()
            .ok_or_else(|| AppError::Connection("Video stream not available".into()))?;

        // scrcpy sends a 64-byte device name followed by video info
        let mut name_buf = [0u8; 64];
        stream
            .read_exact(&mut name_buf)
            .map_err(|e| AppError::Connection(format!("Failed to read device name: {}", e)))?;

        let device_name = String::from_utf8_lossy(&name_buf)
            .trim_end_matches('\0')
            .to_string();

        debug!("Device name: {}", device_name);

        self.screen_info = Some(DeviceScreenInfo {
            device_name,
            width: 0,  // Will be updated from the video stream SPS
            height: 0,
        });

        Ok(())
    }

    /// Read raw H.264 data from the video socket
    pub fn read_video_frame(&mut self) -> AppResult<BytesMut> {
        let stream = self
            .video_stream
            .as_mut()
            .ok_or_else(|| AppError::Connection("Video stream not available".into()))?;

        // Read frame header: PTS (8 bytes) + size (4 bytes)
        let mut header = [0u8; 12];
        stream
            .read_exact(&mut header)
            .map_err(|e| AppError::Connection(format!("Failed to read frame header: {}", e)))?;

        let frame_size = u32::from_be_bytes([header[8], header[9], header[10], header[11]]) as usize;

        // Read frame data
        let mut frame_data = BytesMut::zeroed(frame_size);
        stream
            .read_exact(&mut frame_data)
            .map_err(|e| AppError::Connection(format!("Failed to read frame data: {}", e)))?;

        Ok(frame_data)
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }
}

impl Drop for ScrcpyConnection {
    fn drop(&mut self) {
        self.video_stream.take();
        self.control_stream.take();
    }
}
