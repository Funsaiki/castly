use crate::video::frame_server::FrameServer;
use bytes::Bytes;
use std::io::Read;
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// AirPlay video header size (128 bytes, only first 16 used)
const AIRPLAY_HEADER_SIZE: usize = 128;

/// Annex-B start code
const START_CODE: [u8; 4] = [0x00, 0x00, 0x00, 0x01];

/// Listen on the assigned video port and receive H.264 frames from iPhone.
/// AirPlay sends H.264 in AVCC format (length-prefixed NALUs) with a 128-byte header per frame.
pub fn video_receive_loop(
    port: u16,
    frame_server: Arc<FrameServer>,
    stop_flag: Arc<AtomicBool>,
) {
    info!("AirPlay video receiver starting on port {}", port);

    let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)) {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind AirPlay video port {}: {}", port, e);
            return;
        }
    };

    // Accept one video stream connection
    let (mut stream, peer) = match listener.accept() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to accept AirPlay video connection: {}", e);
            return;
        }
    };

    info!("AirPlay video stream connected from {:?}", peer);

    let mut frame_count: u64 = 0;

    loop {
        if stop_flag.load(Ordering::Relaxed) {
            break;
        }

        // Read 128-byte AirPlay header
        let mut header = [0u8; AIRPLAY_HEADER_SIZE];
        match stream.read_exact(&mut header) {
            Ok(_) => {}
            Err(e) => {
                if !stop_flag.load(Ordering::Relaxed) {
                    error!("AirPlay video header read error: {}", e);
                }
                break;
            }
        }

        // Parse header (little-endian)
        let payload_size = u32::from_le_bytes([header[0], header[1], header[2], header[3]]) as usize;
        let payload_type = u16::from_le_bytes([header[4], header[5]]);
        // Bytes 8-15: NTP timestamp (unused for now)

        if payload_size == 0 || payload_size > 10_000_000 {
            if payload_type == 2 {
                // Heartbeat packet, skip
                continue;
            }
            warn!("AirPlay invalid payload size: {}", payload_size);
            continue;
        }

        // Read payload
        let mut payload = vec![0u8; payload_size];
        match stream.read_exact(&mut payload) {
            Ok(_) => {}
            Err(e) => {
                error!("AirPlay video payload read error: {}", e);
                break;
            }
        }

        // Convert AVCC to Annex-B format and send to frame server
        let annex_b = match payload_type {
            0 => {
                // Regular video frame (AVCC format)
                avcc_to_annex_b(&payload)
            }
            1 => {
                // SPS/PPS config data (avcC format from ISO 14496-15)
                parse_avcc_config(&payload)
            }
            2 => {
                // Heartbeat
                continue;
            }
            _ => {
                debug!("AirPlay unknown payload type: {}", payload_type);
                avcc_to_annex_b(&payload)
            }
        };

        if !annex_b.is_empty() {
            frame_server.send_frame(Bytes::from(annex_b));
            frame_count += 1;

            if frame_count <= 5 || frame_count % 60 == 0 {
                debug!("AirPlay video: {} frames (type={}, size={})", frame_count, payload_type, payload_size);
            }
        }
    }

    info!("AirPlay video receiver ended after {} frames", frame_count);
}

/// Convert AVCC (length-prefixed NALUs) to Annex-B (start code NALUs)
fn avcc_to_annex_b(avcc: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(avcc.len() + 32);
    let mut offset = 0;

    while offset + 4 <= avcc.len() {
        let nal_size = u32::from_be_bytes([
            avcc[offset],
            avcc[offset + 1],
            avcc[offset + 2],
            avcc[offset + 3],
        ]) as usize;
        offset += 4;

        if nal_size == 0 || offset + nal_size > avcc.len() {
            break;
        }

        result.extend_from_slice(&START_CODE);
        result.extend_from_slice(&avcc[offset..offset + nal_size]);
        offset += nal_size;
    }

    result
}

/// Parse avcC configuration record and extract SPS/PPS as Annex-B
fn parse_avcc_config(data: &[u8]) -> Vec<u8> {
    if data.len() < 8 {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut offset = 5; // Skip configurationVersion(1), profile(1), compat(1), level(1), lengthSizeMinusOne(1)

    if offset >= data.len() {
        return result;
    }

    // Number of SPS
    let num_sps = (data[offset] & 0x1F) as usize;
    offset += 1;

    for _ in 0..num_sps {
        if offset + 2 > data.len() {
            break;
        }
        let sps_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;
        if offset + sps_len > data.len() {
            break;
        }
        result.extend_from_slice(&START_CODE);
        result.extend_from_slice(&data[offset..offset + sps_len]);
        offset += sps_len;
    }

    if offset >= data.len() {
        return result;
    }

    // Number of PPS
    let num_pps = data[offset] as usize;
    offset += 1;

    for _ in 0..num_pps {
        if offset + 2 > data.len() {
            break;
        }
        let pps_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;
        if offset + pps_len > data.len() {
            break;
        }
        result.extend_from_slice(&START_CODE);
        result.extend_from_slice(&data[offset..offset + pps_len]);
        offset += pps_len;
    }

    if !result.is_empty() {
        info!("AirPlay parsed avcC config: {} SPS + {} PPS ({} bytes)", num_sps, num_pps, result.len());
    }

    result
}
