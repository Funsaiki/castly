use crate::error::AppResult;
use crate::pipeline::{AirPlayPipelineHandle, PipelineHandle};
use crate::state::{AppState, DeviceInfo, DeviceStatus, DeviceType, ConnectionType, MirrorSession};
use crate::video::frame_server::FrameServer;
use bytes::Bytes;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tracing::{debug, error, info, warn};

const AIRPLAY_PORT: u16 = 7100;
const SERVER_NAME: &str = "AirTunes/220.68";

/// Minimal RTSP/HTTP server for AirPlay screen mirroring handshake.
pub struct RtspServer {
    stop_flag: Arc<AtomicBool>,
}

impl RtspServer {
    pub fn new() -> Self {
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start listening for AirPlay connections on port 7100.
    pub fn start(
        &self,
        app_handle: AppHandle,
        state: Arc<AppState>,
    ) -> anyhow::Result<()> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", AIRPLAY_PORT))?;
        listener.set_nonblocking(false)?;
        info!("AirPlay RTSP server listening on port {}", AIRPLAY_PORT);

        let stop_flag = self.stop_flag.clone();

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }

                match stream {
                    Ok(tcp_stream) => {
                        let peer = tcp_stream.peer_addr().ok();
                        info!("AirPlay connection from {:?}", peer);

                        let app = app_handle.clone();
                        let st = state.clone();

                        std::thread::spawn(move || {
                            if let Err(e) = handle_connection(tcp_stream, app, st) {
                                error!("AirPlay session error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        if !stop_flag.load(Ordering::Relaxed) {
                            error!("AirPlay accept error: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}

/// Parse and handle one AirPlay RTSP/HTTP session.
fn handle_connection(
    mut stream: TcpStream,
    app_handle: AppHandle,
    state: Arc<AppState>,
) -> anyhow::Result<()> {
    let peer_ip = stream.peer_addr()?.ip().to_string();
    let device_id = format!("airplay-{}", peer_ip.replace('.', "-"));
    let mut device_name = format!("iPhone ({})", peer_ip);
    let mut video_port: u16 = 0;
    let mut _audio_port: u16 = 0;
    let mut frame_server: Option<Arc<FrameServer>> = None;

    info!("Handling AirPlay session from {}", peer_ip);

    loop {
        // Read RTSP request
        let request = match read_rtsp_request(&mut stream) {
            Ok(Some(req)) => req,
            Ok(None) => {
                info!("AirPlay client disconnected: {}", peer_ip);
                break;
            }
            Err(e) => {
                debug!("AirPlay read error: {}", e);
                break;
            }
        };

        debug!(
            "AirPlay request: {} {} (body: {} bytes)",
            request.method, request.path, request.body.len()
        );

        let cseq = request
            .headers
            .get("cseq")
            .cloned()
            .unwrap_or_else(|| "0".to_string());

        match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/info") => {
                let info_plist = build_info_plist();
                send_response(
                    &mut stream,
                    200,
                    &cseq,
                    "application/x-apple-binary-plist",
                    &info_plist,
                )?;
            }

            ("POST", "/pair-setup") | ("POST", "/pair-verify") => {
                // Minimal stub — respond with empty success
                send_response(&mut stream, 200, &cseq, "application/octet-stream", &[])?;
            }

            ("POST", path) if path.starts_with("/fp-setup") => {
                // FairPlay stub — respond with minimal data
                let fp_response = crate::airplay::fairplay::handle_fp_setup(&request.body);
                send_response(
                    &mut stream,
                    200,
                    &cseq,
                    "application/octet-stream",
                    &fp_response,
                )?;
            }

            ("SETUP", _) => {
                // Parse the SETUP request body (binary plist with stream config)
                let (vp, ap) = parse_setup_body(&request.body);

                // Assign ports for receiving data
                video_port = portpicker::pick_unused_port().unwrap_or(7101);
                _audio_port = portpicker::pick_unused_port().unwrap_or(7102);

                info!(
                    "AirPlay SETUP: client wants video_type={}, audio_type={}. Assigned video_port={}, audio_port={}",
                    vp, ap, video_port, _audio_port
                );

                // Build SETUP response (binary plist)
                let response_plist = build_setup_response(video_port, _audio_port);
                send_response(
                    &mut stream,
                    200,
                    &cseq,
                    "application/x-apple-binary-plist",
                    &response_plist,
                )?;
            }

            ("RECORD", _) => {
                info!("AirPlay RECORD — starting mirroring pipeline");

                // Start frame server
                let mut fs = FrameServer::new();
                let rt = tokio::runtime::Handle::current();
                let fs_port = rt.block_on(async { fs.start().await })?;
                let fs = Arc::new(fs);
                frame_server = Some(fs.clone());

                // Register device in state
                let device = DeviceInfo {
                    id: device_id.clone(),
                    name: device_name.clone(),
                    device_type: DeviceType::Ios,
                    connection: ConnectionType::Wifi,
                    status: DeviceStatus::Mirroring,
                    screen_width: 0,
                    screen_height: 0,
                };
                state.devices.write().insert(device_id.clone(), device.clone());

                let session = MirrorSession {
                    device_id: device_id.clone(),
                    stream_url: format!("http://127.0.0.1:{}/stream", fs_port),
                    screen_width: 0,
                    screen_height: 0,
                    audio_codec: "aac-eld".to_string(),
                    is_recording: false,
                    recording_path: None,
                };
                state.sessions.write().insert(device_id.clone(), session);

                let stop = Arc::new(AtomicBool::new(false));

                // Register pipeline handle
                let handle = PipelineHandle::Ios(AirPlayPipelineHandle {
                    stop_flag: stop.clone(),
                });
                state.pipelines.write().insert(device_id.clone(), handle);

                let _ = app_handle.emit("airplay-device-connected", device);

                send_response(&mut stream, 200, &cseq, "text/parameters", b"")?;

                // Start receiving video on the video port
                let fs_clone = fs.clone();
                let stop_clone = stop.clone();
                let vp = video_port;

                std::thread::spawn(move || {
                    crate::airplay::receiver::video_receive_loop(vp, fs_clone, stop_clone);
                });

                info!("AirPlay mirroring active for {}", device_name);
            }

            ("GET", "/feedback") | ("POST", "/feedback") => {
                send_response(&mut stream, 200, &cseq, "text/parameters", b"")?;
            }

            ("GET_PARAMETER", _) | ("SET_PARAMETER", _) => {
                send_response(&mut stream, 200, &cseq, "text/parameters", b"")?;
            }

            ("OPTIONS", _) => {
                let methods = "ANNOUNCE, SETUP, RECORD, PAUSE, FLUSH, TEARDOWN, OPTIONS, GET_PARAMETER, SET_PARAMETER, POST, GET";
                send_response(
                    &mut stream,
                    200,
                    &cseq,
                    "text/parameters",
                    methods.as_bytes(),
                )?;
            }

            ("TEARDOWN", _) => {
                info!("AirPlay TEARDOWN from {}", peer_ip);
                send_response(&mut stream, 200, &cseq, "text/parameters", b"")?;
                break;
            }

            _ => {
                debug!("AirPlay unhandled: {} {}", request.method, request.path);
                send_response(&mut stream, 200, &cseq, "text/parameters", b"")?;
            }
        }
    }

    // Cleanup
    info!("AirPlay session ended for {}", device_id);
    state.devices.write().remove(&device_id);
    state.sessions.write().remove(&device_id);
    state.pipelines.write().remove(&device_id);
    let _ = app_handle.emit("airplay-device-disconnected", device_id);

    Ok(())
}

// --- RTSP Request Parsing ---

struct RtspRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

fn read_rtsp_request(stream: &mut TcpStream) -> anyhow::Result<Option<RtspRequest>> {
    let mut reader = BufReader::new(stream.try_clone()?);

    // Read request line
    let mut request_line = String::new();
    let n = reader.read_line(&mut request_line)?;
    if n == 0 {
        return Ok(None); // Connection closed
    }

    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(None);
    }

    let method = parts[0].to_string();
    let path = parts[1].to_string();

    // Read headers
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let line = line.trim().to_string();
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(
                key.trim().to_lowercase(),
                value.trim().to_string(),
            );
        }
    }

    // Read body if Content-Length is present
    let content_length: usize = headers
        .get("content-length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    Ok(Some(RtspRequest {
        method,
        path,
        headers,
        body,
    }))
}

// --- RTSP Response ---

fn send_response(
    stream: &mut TcpStream,
    status: u16,
    cseq: &str,
    content_type: &str,
    body: &[u8],
) -> anyhow::Result<()> {
    let status_text = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "OK",
    };

    let header = format!(
        "RTSP/1.0 {} {}\r\nCSeq: {}\r\nServer: {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        status, status_text, cseq, SERVER_NAME, content_type, body.len()
    );

    stream.write_all(header.as_bytes())?;
    if !body.is_empty() {
        stream.write_all(body)?;
    }
    stream.flush()?;

    Ok(())
}

// --- Protocol Helpers ---

/// Build the /info response plist
fn build_info_plist() -> Vec<u8> {
    use plist::Value;

    let mut dict = plist::Dictionary::new();
    dict.insert("deviceid".to_string(), Value::String("58:55:CA:1A:E2:88".to_string()));
    dict.insert("model".to_string(), Value::String("AppleTV3,2".to_string()));
    dict.insert("features".to_string(), Value::Integer(0x527FFFF7.into()));
    dict.insert("statusFlags".to_string(), Value::Integer(68.into()));
    dict.insert("srcvers".to_string(), Value::String("220.68".to_string()));
    dict.insert("macAddress".to_string(), Value::String("58:55:CA:1A:E2:88".to_string()));
    dict.insert("vv".to_string(), Value::Integer(2.into()));
    dict.insert("pi".to_string(), Value::String(uuid::Uuid::new_v4().to_string()));
    dict.insert("pk".to_string(), Value::Data(vec![0u8; 32]));

    let mut buf = Vec::new();
    plist::to_writer_binary(&mut buf, &Value::Dictionary(dict)).unwrap_or_default();
    buf
}

/// Parse SETUP body to extract stream types
fn parse_setup_body(body: &[u8]) -> (u32, u32) {
    // Try to parse as binary plist
    if let Ok(plist::Value::Dictionary(dict)) = plist::from_bytes::<plist::Value>(body) {
        if let Some(plist::Value::Array(streams)) = dict.get("streams") {
            let mut video_type = 0u32;
            let mut audio_type = 0u32;
            for stream in streams {
                if let plist::Value::Dictionary(sd) = stream {
                    let stype = sd
                        .get("type")
                        .and_then(|v| v.as_unsigned_integer())
                        .unwrap_or(0) as u32;
                    if stype == 110 {
                        video_type = 110;
                    } else if stype == 96 {
                        audio_type = 96;
                    }
                }
            }
            return (video_type, audio_type);
        }
    }
    debug!("Could not parse SETUP body ({} bytes)", body.len());
    (110, 96) // Default: assume video + audio
}

/// Build SETUP response plist with assigned ports
fn build_setup_response(video_port: u16, audio_port: u16) -> Vec<u8> {
    use plist::Value;

    let mut streams = Vec::new();

    // Video stream response
    let mut video = plist::Dictionary::new();
    video.insert("type".to_string(), Value::Integer(110.into()));
    video.insert("dataPort".to_string(), Value::Integer(video_port.into()));
    streams.push(Value::Dictionary(video));

    // Audio stream response
    let mut audio = plist::Dictionary::new();
    audio.insert("type".to_string(), Value::Integer(96.into()));
    audio.insert("dataPort".to_string(), Value::Integer(audio_port.into()));
    streams.push(Value::Dictionary(audio));

    let mut dict = plist::Dictionary::new();
    dict.insert("streams".to_string(), Value::Array(streams));

    let mut buf = Vec::new();
    plist::to_writer_binary(&mut buf, &Value::Dictionary(dict)).unwrap_or_default();
    buf
}
