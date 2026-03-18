use std::sync::Arc;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[tauri::command]
pub async fn start_recording(
    device_id: String,
    state: State<'_, Arc<AppState>>,
) -> AppResult<String> {
    let mut sessions = state.sessions.write();
    let session = sessions
        .get_mut(&device_id)
        .ok_or_else(|| AppError::Other("No active mirror session".into()))?;

    // TODO: Start recording the H.264 stream to MP4
    let path = directories::UserDirs::new()
        .and_then(|dirs| dirs.video_dir().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("PhoneMirror")
        .join(format!(
            "recording_{}.mp4",
            chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
        ));

    session.is_recording = true;
    session.recording_path = Some(path.to_string_lossy().to_string());

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn stop_recording(
    device_id: String,
    state: State<'_, Arc<AppState>>,
) -> AppResult<String> {
    let mut sessions = state.sessions.write();
    let session = sessions
        .get_mut(&device_id)
        .ok_or_else(|| AppError::Other("No active mirror session".into()))?;

    session.is_recording = false;
    let path = session
        .recording_path
        .take()
        .unwrap_or_default();

    // TODO: Finalize MP4 file
    Ok(path)
}

#[tauri::command]
pub async fn take_screenshot(
    device_id: String,
    state: State<'_, Arc<AppState>>,
) -> AppResult<String> {
    let sessions = state.sessions.read();
    if !sessions.contains_key(&device_id) {
        return Err(AppError::Other("No active mirror session".into()));
    }

    // TODO: Decode current frame and save as PNG
    let path = directories::UserDirs::new()
        .and_then(|dirs| dirs.picture_dir().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("PhoneMirror")
        .join(format!(
            "screenshot_{}.png",
            chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
        ));

    Ok(path.to_string_lossy().to_string())
}
