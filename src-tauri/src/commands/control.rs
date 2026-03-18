use std::sync::Arc;
use tauri::State;

use crate::adb::protocol::{ControlMessage, KeyAction, ScreenPowerMode, TouchAction};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[tauri::command]
pub async fn inject_touch(
    device_id: String,
    action: String,
    x: f32,
    y: f32,
    screen_width: u32,
    screen_height: u32,
    state: State<'_, Arc<AppState>>,
) -> AppResult<()> {
    let pipelines = state.pipelines.read();
    let pipeline = pipelines
        .get(&device_id)
        .ok_or_else(|| AppError::Other("No active pipeline".into()))?;

    let touch_action = match action.as_str() {
        "down" => TouchAction::Down,
        "up" => TouchAction::Up,
        "move" => TouchAction::Move,
        _ => return Err(AppError::Other(format!("Invalid touch action: {}", action))),
    };

    let msg = ControlMessage::InjectTouchEvent {
        action: touch_action,
        pointer_id: 0xFFFFFFFFFFFFFFFF, // "finger" pointer
        x,
        y,
        width: screen_width,
        height: screen_height,
        pressure: if matches!(touch_action, TouchAction::Up) {
            0.0
        } else {
            1.0
        },
        action_button: 0,
        buttons: 0,
    };

    pipeline.send_control(msg)
}

#[tauri::command]
pub async fn inject_key(
    device_id: String,
    action: String,
    keycode: u32,
    state: State<'_, Arc<AppState>>,
) -> AppResult<()> {
    let pipelines = state.pipelines.read();
    let pipeline = pipelines
        .get(&device_id)
        .ok_or_else(|| AppError::Other("No active pipeline".into()))?;

    let key_action = match action.as_str() {
        "down" => KeyAction::Down,
        "up" => KeyAction::Up,
        _ => return Err(AppError::Other(format!("Invalid key action: {}", action))),
    };

    let msg = ControlMessage::InjectKeycode {
        action: key_action,
        keycode,
        repeat: 0,
        metastate: 0,
    };

    pipeline.send_control(msg)
}

#[tauri::command]
pub async fn inject_scroll(
    device_id: String,
    x: f32,
    y: f32,
    screen_width: u32,
    screen_height: u32,
    hscroll: f32,
    vscroll: f32,
    state: State<'_, Arc<AppState>>,
) -> AppResult<()> {
    let pipelines = state.pipelines.read();
    let pipeline = pipelines
        .get(&device_id)
        .ok_or_else(|| AppError::Other("No active pipeline".into()))?;

    let msg = ControlMessage::InjectScrollEvent {
        x,
        y,
        width: screen_width,
        height: screen_height,
        hscroll,
        vscroll,
        buttons: 0,
    };

    pipeline.send_control(msg)
}

#[tauri::command]
pub async fn press_back(
    device_id: String,
    state: State<'_, Arc<AppState>>,
) -> AppResult<()> {
    let pipelines = state.pipelines.read();
    let pipeline = pipelines
        .get(&device_id)
        .ok_or_else(|| AppError::Other("No active pipeline".into()))?;

    // KEYCODE_BACK = 4
    pipeline.send_control(ControlMessage::InjectKeycode {
        action: KeyAction::Down,
        keycode: 4,
        repeat: 0,
        metastate: 0,
    })?;
    pipeline.send_control(ControlMessage::InjectKeycode {
        action: KeyAction::Up,
        keycode: 4,
        repeat: 0,
        metastate: 0,
    })
}

#[tauri::command]
pub async fn press_home(
    device_id: String,
    state: State<'_, Arc<AppState>>,
) -> AppResult<()> {
    let pipelines = state.pipelines.read();
    let pipeline = pipelines
        .get(&device_id)
        .ok_or_else(|| AppError::Other("No active pipeline".into()))?;

    // KEYCODE_HOME = 3
    pipeline.send_control(ControlMessage::InjectKeycode {
        action: KeyAction::Down,
        keycode: 3,
        repeat: 0,
        metastate: 0,
    })?;
    pipeline.send_control(ControlMessage::InjectKeycode {
        action: KeyAction::Up,
        keycode: 3,
        repeat: 0,
        metastate: 0,
    })
}

#[tauri::command]
pub async fn press_recent(
    device_id: String,
    state: State<'_, Arc<AppState>>,
) -> AppResult<()> {
    let pipelines = state.pipelines.read();
    let pipeline = pipelines
        .get(&device_id)
        .ok_or_else(|| AppError::Other("No active pipeline".into()))?;

    // KEYCODE_APP_SWITCH = 187
    pipeline.send_control(ControlMessage::InjectKeycode {
        action: KeyAction::Down,
        keycode: 187,
        repeat: 0,
        metastate: 0,
    })?;
    pipeline.send_control(ControlMessage::InjectKeycode {
        action: KeyAction::Up,
        keycode: 187,
        repeat: 0,
        metastate: 0,
    })
}

#[tauri::command]
pub async fn set_screen_power(
    device_id: String,
    on: bool,
    state: State<'_, Arc<AppState>>,
) -> AppResult<()> {
    let pipelines = state.pipelines.read();
    let pipeline = pipelines
        .get(&device_id)
        .ok_or_else(|| AppError::Other("No active pipeline".into()))?;

    pipeline.send_control(ControlMessage::SetScreenPowerMode {
        mode: if on {
            ScreenPowerMode::Normal
        } else {
            ScreenPowerMode::Off
        },
    })
}
