// Touch input handling - maps mouse events to Android touch events
// TODO: Implement fully in Phase 2

use crate::adb::protocol::{ControlMessage, TouchAction};

/// Convert a mouse event to a scrcpy touch injection message
pub fn mouse_to_touch(
    action: TouchAction,
    x: f32,
    y: f32,
    screen_width: u32,
    screen_height: u32,
) -> ControlMessage {
    ControlMessage::InjectTouchEvent {
        action,
        pointer_id: 0xFFFFFFFFFFFFFFFF, // "finger" pointer ID used by scrcpy
        x,
        y,
        width: screen_width,
        height: screen_height,
        pressure: match action {
            TouchAction::Up => 0.0,
            _ => 1.0,
        },
        action_button: 0,
        buttons: 0,
    }
}
