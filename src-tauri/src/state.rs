use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub connection: ConnectionType,
    pub status: DeviceStatus,
    pub screen_width: u32,
    pub screen_height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Android,
    Ios,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    Usb,
    Wifi,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Disconnected,
    Connecting,
    Connected,
    Mirroring,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorSession {
    pub device_id: String,
    pub stream_url: String,
    pub screen_width: u32,
    pub screen_height: u32,
    pub audio_codec: String,
    pub is_recording: bool,
    pub recording_path: Option<String>,
}

pub struct AppState {
    pub devices: RwLock<HashMap<String, DeviceInfo>>,
    pub sessions: RwLock<HashMap<String, MirrorSession>>,
    pub pipelines: RwLock<HashMap<String, crate::pipeline::PipelineHandle>>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            devices: RwLock::new(HashMap::new()),
            sessions: RwLock::new(HashMap::new()),
            pipelines: RwLock::new(HashMap::new()),
        })
    }
}
