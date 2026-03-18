use crate::adb::client::AdbClient;
use crate::adb::protocol::ControlMessage;
use crate::error::{AppError, AppResult};
use crate::video::frame_server::FrameServer;
use parking_lot::Mutex;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

const SCRCPY_SERVER_PATH: &str = "/data/local/tmp/scrcpy-server.jar";
const SCRCPY_VERSION: &str = "3.1";

/// Configuration for a mirror session
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MirrorConfig {
    pub max_size: u32,
    pub bit_rate: u32,
    pub max_fps: u32,
}

impl Default for MirrorConfig {
    fn default() -> Self {
        Self {
            max_size: 1920,
            bit_rate: 8_000_000,
            max_fps: 60,
        }
    }
}

/// A running mirror pipeline: scrcpy server + video stream + frame server
pub struct MirrorPipeline {
    serial: String,
    adb: AdbClient,
    frame_server: Arc<FrameServer>,
    stop_flag: Arc<std::sync::atomic::AtomicBool>,
    server_process: Option<std::process::Child>,
    local_port: u16,
    /// Control socket for sending input events to the phone
    control_writer: Arc<Mutex<Option<TcpStream>>>,
    /// Phone screen dimensions (for coordinate mapping)
    pub screen_width: u32,
    pub screen_height: u32,
}

impl MirrorPipeline {
    /// Start a full mirror pipeline for a device
    pub async fn start(serial: String, config: MirrorConfig) -> AppResult<Self> {
        let adb = AdbClient::new();
        let local_port = portpicker::pick_unused_port().unwrap_or(27183);

        info!("Starting mirror pipeline for {} on port {}", serial, local_port);

        // 0. Kill any existing scrcpy server and remove old port forwards
        let _ = std::process::Command::new(adb.adb_path())
            .args(["-s", &serial, "shell", "pkill -f scrcpy"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output();
        let _ = std::process::Command::new(adb.adb_path())
            .args(["-s", &serial, "forward", "--remove-all"])
            .output();
        std::thread::sleep(Duration::from_millis(500));
        info!("Cleaned up old scrcpy processes");

        // 1. Find and push scrcpy-server.jar
        let jar_path = Self::find_server_jar()?;
        info!("Found scrcpy-server at: {:?}", jar_path);
        adb.push_file(&serial, &jar_path.to_string_lossy(), SCRCPY_SERVER_PATH)?;
        info!("Pushed scrcpy-server to device");

        // 2. Set up ADB forward
        let local_addr = format!("tcp:{}", local_port);
        adb.forward(&serial, &local_addr, "localabstract:scrcpy")?;
        info!("Port forwarding: {} -> localabstract:scrcpy", local_addr);

        // 3. Launch scrcpy server
        let mut server_process = Self::launch_server(&adb, &serial, &config)?;
        info!("scrcpy server launching...");

        // Give the server time to start
        std::thread::sleep(Duration::from_millis(1500));

        // Check if server is still running
        match server_process.try_wait() {
            Ok(Some(status)) => {
                // Server exited - read stderr for error info
                let mut stderr_output = String::new();
                if let Some(ref mut stderr) = server_process.stderr {
                    let _ = stderr.read_to_string(&mut stderr_output);
                }
                let mut stdout_output = String::new();
                if let Some(ref mut stdout) = server_process.stdout {
                    let _ = stdout.read_to_string(&mut stdout_output);
                }
                error!("scrcpy server exited with {}: stdout={}, stderr={}", status, stdout_output, stderr_output);
                return Err(AppError::Adb(format!(
                    "scrcpy server crashed: {}{}",
                    stdout_output, stderr_output
                )));
            }
            Ok(None) => {
                info!("scrcpy server is running");
            }
            Err(e) => {
                warn!("Could not check server status: {}", e);
            }
        }

        // 4. Connect video socket with retry
        let mut video_stream = Self::connect_with_retry(local_port, 15)?;
        let _ = video_stream.set_read_timeout(Some(Duration::from_secs(10)));
        info!("Video socket connected");

        // 5. Read dummy byte (0x00) - scrcpy handshake on video socket
        let mut dummy = [0u8; 1];
        video_stream
            .read_exact(&mut dummy)
            .map_err(|e| AppError::Connection(format!("Failed to read video dummy byte: {}", e)))?;
        info!("Video dummy byte: 0x{:02x}", dummy[0]);

        // 6. Connect audio socket (no dummy byte on audio)
        std::thread::sleep(Duration::from_millis(100));
        let mut audio_stream = Self::connect_with_retry(local_port, 10)?;
        let _ = audio_stream.set_read_timeout(Some(Duration::from_secs(10)));
        info!("Audio socket connected");

        // 7. Connect control socket - clone for reading (drain) and writing (commands)
        std::thread::sleep(Duration::from_millis(100));
        let control_writer: Arc<Mutex<Option<TcpStream>>> = Arc::new(Mutex::new(None));
        match TcpStream::connect(format!("127.0.0.1:{}", local_port)) {
            Ok(control_stream) => {
                info!("Control socket connected");
                let write_clone = control_stream
                    .try_clone()
                    .map_err(|e| AppError::Connection(format!("Failed to clone control socket: {}", e)))?;
                *control_writer.lock() = Some(write_clone);

                std::thread::spawn(move || {
                    let mut stream = control_stream;
                    let mut buf = [0u8; 4096];
                    loop {
                        match stream.read(&mut buf) {
                            Ok(0) => {
                                debug!("Control socket closed");
                                break;
                            }
                            Ok(n) => {
                                debug!("Drained {} bytes from control socket", n);
                            }
                            Err(e) => {
                                debug!("Control socket error: {}", e);
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                warn!("Control socket failed: {}", e);
            }
        }

        // Give server a moment after all sockets are connected
        std::thread::sleep(Duration::from_millis(300));

        // 8. Read video protocol data: 64-byte device name
        let mut name_buf = [0u8; 64];
        video_stream
            .read_exact(&mut name_buf)
            .map_err(|e| AppError::Connection(format!("Failed to read device name: {}", e)))?;
        let device_name = String::from_utf8_lossy(&name_buf)
            .trim_end_matches('\0')
            .to_string();
        info!("Device name: '{}'", device_name);

        // Read codec ID (4 bytes, e.g., "h264")
        let mut codec_buf = [0u8; 4];
        video_stream
            .read_exact(&mut codec_buf)
            .map_err(|e| AppError::Connection(format!("Failed to read codec: {}", e)))?;
        let codec_str = String::from_utf8_lossy(&codec_buf);
        info!("Codec: '{}' (0x{:02x}{:02x}{:02x}{:02x})", codec_str,
            codec_buf[0], codec_buf[1], codec_buf[2], codec_buf[3]);

        // Read initial video size (width: i32, height: i32)
        let mut size_buf = [0u8; 8];
        video_stream
            .read_exact(&mut size_buf)
            .map_err(|e| AppError::Connection(format!("Failed to read video size: {}", e)))?;
        let width = i32::from_be_bytes(size_buf[0..4].try_into().unwrap()) as u32;
        let height = i32::from_be_bytes(size_buf[4..8].try_into().unwrap()) as u32;
        info!("Video size: {}x{}", width, height);

        // 9. Read audio codec metadata
        let mut audio_codec_buf = [0u8; 4];
        audio_stream
            .read_exact(&mut audio_codec_buf)
            .map_err(|e| AppError::Connection(format!("Failed to read audio codec: {}", e)))?;
        let audio_codec = String::from_utf8_lossy(&audio_codec_buf);
        info!("Audio codec: '{}'", audio_codec);

        // 10. Start frame server
        let mut frame_server = FrameServer::new();
        let server_port = frame_server
            .start()
            .await
            .map_err(|e| AppError::Video(format!("Failed to start frame server: {}", e)))?;
        info!("Frame server on http://127.0.0.1:{}", server_port);

        let frame_server = Arc::new(frame_server);
        let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));

        // 11. Start video reading loop
        let fs = frame_server.clone();
        let sf = stop_flag.clone();
        std::thread::spawn(move || {
            Self::video_read_loop(video_stream, fs, sf, width, height, Vec::new());
        });

        // 12. Start audio reading loop
        let fs2 = frame_server.clone();
        let sf2 = stop_flag.clone();
        std::thread::spawn(move || {
            Self::audio_read_loop(audio_stream, fs2, sf2);
        });

        Ok(Self {
            serial,
            adb,
            frame_server,
            stop_flag,
            server_process: Some(server_process),
            local_port,
            control_writer,
            screen_width: width,
            screen_height: height,
        })
    }

    fn find_server_jar() -> AppResult<PathBuf> {
        let candidates = vec![
            PathBuf::from("../resources/scrcpy-server.jar"),
            PathBuf::from("../../resources/scrcpy-server.jar"),
            PathBuf::from("resources/scrcpy-server.jar"),
            PathBuf::from("../src-tauri/resources/scrcpy-server.jar"),
        ];

        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let path = dir.join("resources").join("scrcpy-server.jar");
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        for path in &candidates {
            if path.exists() {
                return Ok(
                    std::fs::canonicalize(path).map_err(|e| AppError::Io(e))?,
                );
            }
        }

        Err(AppError::Other(
            "scrcpy-server.jar not found. Place it in src-tauri/resources/".into(),
        ))
    }

    /// Analyze the initial bytes to determine the scrcpy protocol variant
    fn detect_protocol(data: &[u8]) -> (usize, u32, u32) {
        let len = data.len();

        if len < 64 {
            return (0, 1920, 1080);
        }

        // After 64-byte device name, check what follows
        let after = &data[64..];
        info!("Protocol detection: {} bytes after device name", after.len());

        if after.len() >= 12 {
            // Check if bytes 64-67 look like a codec ID (e.g., 0x68323634 = "h264")
            let possible_codec = &after[0..4];
            let codec_str = String::from_utf8_lossy(possible_codec);
            info!("Possible codec string: '{}' (bytes: {:02x} {:02x} {:02x} {:02x})",
                codec_str, possible_codec[0], possible_codec[1], possible_codec[2], possible_codec[3]);

            // Check if it's a known codec ID
            if possible_codec == b"h264" || possible_codec == b"h265" || possible_codec == b"av01" {
                // v2.2+ protocol: 64 name + 4 codec + 4 dimensions = 72 bytes header
                let w = u16::from_be_bytes([after[4], after[5]]) as u32;
                let h = u16::from_be_bytes([after[6], after[7]]) as u32;
                if w > 0 && w < 10000 && h > 0 && h < 10000 {
                    info!("Detected v2.2+ protocol: codec={}, {}x{}", codec_str, w, h);
                    return (72, w, h);
                }
                // Maybe dimensions are u32
                let w32 = u32::from_be_bytes([after[4], after[5], after[6], after[7]]);
                let h32 = u32::from_be_bytes([after[8], after[9], after[10], after[11]]);
                if w32 > 0 && w32 < 10000 && h32 > 0 && h32 < 10000 {
                    info!("Detected v2.2+ protocol (u32 dims): {}x{}", w32, h32);
                    return (76, w32, h32);
                }
            }

            // Check if first bytes after name look like a PTS (frame meta)
            let possible_pts = u64::from_be_bytes(after[0..8].try_into().unwrap());
            let possible_size = u32::from_be_bytes(after[8..12].try_into().unwrap());
            if possible_size > 0 && possible_size < 1_000_000 {
                info!("Detected v2.0 protocol: first frame PTS={}, size={}", possible_pts, possible_size);
                return (64, 1920, 1080);
            }
        }

        // Default: assume 64-byte header only
        info!("Using default protocol: 64-byte header");
        (64, 1920, 1080)
    }

    fn launch_server(
        adb: &AdbClient,
        serial: &str,
        config: &MirrorConfig,
    ) -> AppResult<std::process::Child> {
        // Parameters for scrcpy v3.1
        let cmd = format!(
            "CLASSPATH={} app_process / com.genymobile.scrcpy.Server {} \
             log_level=info \
             max_size={} \
             video_bit_rate={} \
             max_fps={} \
             tunnel_forward=true \
             video_codec=h264 \
             audio=true \
             audio_codec=opus \
             control=true \
             send_frame_meta=true \
             send_device_meta=true \
             send_dummy_byte=true",
            SCRCPY_SERVER_PATH,
            SCRCPY_VERSION,
            config.max_size,
            config.bit_rate,
            config.max_fps,
        );

        info!("Launching scrcpy: adb -s {} shell {}", serial, cmd);

        // Use piped stdout/stderr but drain them in background threads
        // to prevent the pipe buffer from filling up and blocking the server
        let mut child = std::process::Command::new(adb.adb_path())
            .args(["-s", serial, "shell", &cmd])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| AppError::Adb(format!("Failed to launch scrcpy: {}", e)))?;

        // Drain stdout in background
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(move || {
                use std::io::BufRead;
                let reader = std::io::BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(l) if !l.is_empty() => info!("[scrcpy stdout] {}", l),
                        Err(e) => { debug!("[scrcpy stdout closed] {}", e); break; }
                        _ => {}
                    }
                }
            });
        }

        // Drain stderr in background
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                use std::io::BufRead;
                let reader = std::io::BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(l) if !l.is_empty() => warn!("[scrcpy stderr] {}", l),
                        Err(e) => { debug!("[scrcpy stderr closed] {}", e); break; }
                        _ => {}
                    }
                }
            });
        }

        Ok(child)
    }

    fn connect_with_retry(port: u16, max_attempts: u32) -> AppResult<TcpStream> {
        for attempt in 1..=max_attempts {
            match TcpStream::connect(format!("127.0.0.1:{}", port)) {
                Ok(stream) => {
                    info!("Connected on attempt {}", attempt);
                    return Ok(stream);
                }
                Err(e) => {
                    if attempt == max_attempts {
                        return Err(AppError::Connection(format!(
                            "Failed to connect to scrcpy after {} attempts: {}",
                            max_attempts, e
                        )));
                    }
                    debug!(
                        "Connection attempt {}/{} failed: {}, retrying...",
                        attempt, max_attempts, e
                    );
                    std::thread::sleep(Duration::from_millis(300));
                }
            }
        }
        unreachable!()
    }

    /// Main loop: reads H.264 frames from scrcpy and sends raw Annex-B data via WebSocket
    fn video_read_loop(
        mut stream: TcpStream,
        frame_server: Arc<FrameServer>,
        stop_flag: Arc<std::sync::atomic::AtomicBool>,
        width: u32,
        height: u32,
        _leftover: Vec<u8>,
    ) {
        let mut frame_count: u64 = 0;

        info!("Video read loop started ({}x{})", width, height);

        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

        loop {
            if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                info!("Video read loop stopping (flag set)");
                break;
            }

            // Read frame header: PTS (8 bytes) + size (4 bytes)
            let mut header = [0u8; 12];
            match stream.read_exact(&mut header) {
                Ok(_) => {}
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut
                    || e.kind() == std::io::ErrorKind::WouldBlock =>
                {
                    continue;
                }
                Err(e) => {
                    if !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        error!("Failed to read frame header: {}", e);
                    }
                    break;
                }
            }

            let size = u32::from_be_bytes(header[8..12].try_into().unwrap()) as usize;

            if size == 0 {
                continue;
            }
            if size > 10_000_000 {
                warn!("Frame too large: {} bytes", size);
                break;
            }

            // Read the complete frame (Annex-B H.264 data)
            let mut frame_data = vec![0u8; size];
            match stream.read_exact(&mut frame_data) {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to read frame data ({} bytes): {}", size, e);
                    break;
                }
            }

            // Send raw H.264 Annex-B data directly to WebSocket clients
            frame_server.send_frame(bytes::Bytes::from(frame_data));
            frame_count += 1;

            if frame_count <= 5 || frame_count % 60 == 0 {
                debug!("Streamed {} frames", frame_count);
            }
        }

        info!("Video read loop ended after {} frames", frame_count);
    }

    /// Audio reading loop: reads Opus frames from scrcpy and sends to clients
    fn audio_read_loop(
        mut stream: TcpStream,
        frame_server: Arc<FrameServer>,
        stop_flag: Arc<std::sync::atomic::AtomicBool>,
    ) {
        let mut frame_count: u64 = 0;
        info!("Audio read loop started");

        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));

        loop {
            if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            // Same format as video: PTS (8) + size (4) + data
            let mut header = [0u8; 12];
            match stream.read_exact(&mut header) {
                Ok(_) => {}
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::TimedOut
                        || e.kind() == std::io::ErrorKind::WouldBlock =>
                {
                    continue;
                }
                Err(e) => {
                    if !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        error!("Failed to read audio header: {}", e);
                    }
                    break;
                }
            }

            let size = u32::from_be_bytes(header[8..12].try_into().unwrap()) as usize;
            if size == 0 || size > 1_000_000 {
                continue;
            }

            let mut audio_data = vec![0u8; size];
            match stream.read_exact(&mut audio_data) {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to read audio data: {}", e);
                    break;
                }
            }

            frame_server.send_audio(bytes::Bytes::from(audio_data));
            frame_count += 1;

            if frame_count <= 3 || frame_count % 500 == 0 {
                debug!("Audio frames: {}", frame_count);
            }
        }

        info!("Audio read loop ended after {} frames", frame_count);
    }

    pub fn stream_url(&self) -> String {
        format!("http://127.0.0.1:{}/stream", self.frame_server.port())
    }

    /// Send a control message to the phone
    pub fn send_control(&self, msg: ControlMessage) -> AppResult<()> {
        let data = msg
            .serialize()
            .map_err(|e| AppError::Connection(format!("Failed to serialize control: {}", e)))?;

        let mut guard = self.control_writer.lock();
        let stream = guard
            .as_mut()
            .ok_or_else(|| AppError::Connection("Control socket not available".into()))?;

        stream
            .write_all(&data)
            .map_err(|e| AppError::Connection(format!("Failed to send control: {}", e)))?;

        Ok(())
    }

    pub fn stop(&mut self) {
        info!("Stopping mirror pipeline for {}", self.serial);
        self.stop_flag
            .store(true, std::sync::atomic::Ordering::Relaxed);

        if let Some(mut proc) = self.server_process.take() {
            let _ = proc.kill();
            let _ = proc.wait();
        }

        let _ = self
            .adb
            .forward_remove(&self.serial, &format!("tcp:{}", self.local_port));
    }
}

impl Drop for MirrorPipeline {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Wrapper enum to support both Android and iOS pipelines in the same state map.
pub enum PipelineHandle {
    Android(MirrorPipeline),
    Ios(AirPlayPipelineHandle),
}

/// Lightweight handle for an AirPlay pipeline (no control, display-only).
pub struct AirPlayPipelineHandle {
    pub stop_flag: Arc<std::sync::atomic::AtomicBool>,
}

impl PipelineHandle {
    pub fn stop(&mut self) {
        match self {
            PipelineHandle::Android(p) => p.stop(),
            PipelineHandle::Ios(p) => {
                p.stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    pub fn send_control(&self, msg: ControlMessage) -> AppResult<()> {
        match self {
            PipelineHandle::Android(p) => p.send_control(msg),
            PipelineHandle::Ios(_) => Err(AppError::AirPlay(
                "iOS does not support remote control".into(),
            )),
        }
    }
}
