use std::sync::Arc;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::pipeline::{MirrorConfig, MirrorPipeline, PipelineHandle};
use crate::state::{AppState, DeviceStatus, DeviceType, MirrorSession};

#[tauri::command]
pub async fn start_mirror(
    device_id: String,
    config: Option<MirrorConfig>,
    state: State<'_, Arc<AppState>>,
) -> AppResult<MirrorSession> {
    // Verify device exists and check type
    let device_type = {
        let devices = state.devices.read();
        let device = devices
            .get(&device_id)
            .ok_or_else(|| AppError::DeviceNotFound(device_id.clone()))?;
        device.device_type.clone()
    };

    // For iOS: the session was already created by the AirPlay RTSP server
    if device_type == DeviceType::Ios {
        let sessions = state.sessions.read();
        if let Some(session) = sessions.get(&device_id) {
            return Ok(session.clone());
        }
        return Err(AppError::AirPlay(
            "iPhone not streaming yet. Start mirroring from the iPhone.".into(),
        ));
    }

    // Android: stop existing pipeline if any (handles page refresh)
    if let Some(mut old_pipeline) = state.pipelines.write().remove(&device_id) {
        old_pipeline.stop();
    }
    state.sessions.write().remove(&device_id);

    // Update device status
    {
        let mut devices = state.devices.write();
        if let Some(device) = devices.get_mut(&device_id) {
            device.status = DeviceStatus::Connecting;
        }
    }

    // Start the Android pipeline
    let config = config.unwrap_or_default();
    let pipeline = MirrorPipeline::start(device_id.clone(), config)
        .await
        .map_err(|e| {
            let mut devices = state.devices.write();
            if let Some(device) = devices.get_mut(&device_id) {
                device.status = DeviceStatus::Connected;
            }
            e
        })?;

    let stream_url = pipeline.stream_url();
    let screen_width = pipeline.screen_width;
    let screen_height = pipeline.screen_height;

    // Wrap in PipelineHandle::Android
    state
        .pipelines
        .write()
        .insert(device_id.clone(), PipelineHandle::Android(pipeline));

    {
        let mut devices = state.devices.write();
        if let Some(device) = devices.get_mut(&device_id) {
            device.status = DeviceStatus::Mirroring;
        }
    }

    let session = MirrorSession {
        device_id: device_id.clone(),
        stream_url,
        screen_width,
        screen_height,
        audio_codec: "opus".to_string(),
        is_recording: false,
        recording_path: None,
    };

    state
        .sessions
        .write()
        .insert(device_id, session.clone());

    Ok(session)
}

#[tauri::command]
pub async fn stop_mirror(
    device_id: String,
    state: State<'_, Arc<AppState>>,
) -> AppResult<()> {
    if let Some(mut pipeline) = state.pipelines.write().remove(&device_id) {
        pipeline.stop();
    }

    state.sessions.write().remove(&device_id);

    {
        let mut devices = state.devices.write();
        if let Some(device) = devices.get_mut(&device_id) {
            device.status = DeviceStatus::Connected;
        }
    }

    Ok(())
}
