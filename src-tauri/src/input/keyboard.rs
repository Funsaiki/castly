// Keyboard input handling - maps browser key events to Android keycodes
// TODO: Implement fully in Phase 2

use crate::adb::protocol::{ControlMessage, KeyAction};
use std::collections::HashMap;

/// Maps browser KeyboardEvent.code to Android KEYCODE_* values
pub fn browser_key_to_android(key_code: &str) -> Option<u32> {
    // Android KeyEvent keycodes
    let map: HashMap<&str, u32> = HashMap::from([
        ("Backspace", 67),
        ("Enter", 66),
        ("Escape", 4),    // KEYCODE_BACK
        ("Home", 3),      // KEYCODE_HOME
        ("Tab", 61),
        ("Space", 62),
        ("ArrowUp", 19),
        ("ArrowDown", 20),
        ("ArrowLeft", 21),
        ("ArrowRight", 22),
        ("Delete", 112),
        // Volume
        ("AudioVolumeUp", 24),
        ("AudioVolumeDown", 25),
        // Letters
        ("KeyA", 29), ("KeyB", 30), ("KeyC", 31), ("KeyD", 32),
        ("KeyE", 33), ("KeyF", 34), ("KeyG", 35), ("KeyH", 36),
        ("KeyI", 37), ("KeyJ", 38), ("KeyK", 39), ("KeyL", 40),
        ("KeyM", 41), ("KeyN", 42), ("KeyO", 43), ("KeyP", 44),
        ("KeyQ", 45), ("KeyR", 46), ("KeyS", 47), ("KeyT", 48),
        ("KeyU", 49), ("KeyV", 50), ("KeyW", 51), ("KeyX", 52),
        ("KeyY", 53), ("KeyZ", 54),
        // Numbers
        ("Digit0", 7), ("Digit1", 8), ("Digit2", 9), ("Digit3", 10),
        ("Digit4", 11), ("Digit5", 12), ("Digit6", 13), ("Digit7", 14),
        ("Digit8", 15), ("Digit9", 16),
    ]);

    map.get(key_code).copied()
}

/// Create a key injection control message
pub fn inject_key(action: KeyAction, android_keycode: u32, metastate: u32) -> ControlMessage {
    ControlMessage::InjectKeycode {
        action,
        keycode: android_keycode,
        repeat: 0,
        metastate,
    }
}
