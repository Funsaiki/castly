use byteorder::{BigEndian, WriteBytesExt};
use std::io::{self, Write};

/// Control message types for the scrcpy protocol
#[derive(Debug)]
pub enum ControlMessage {
    InjectKeycode {
        action: KeyAction,
        keycode: u32,
        repeat: u32,
        metastate: u32,
    },
    InjectText {
        text: String,
    },
    InjectTouchEvent {
        action: TouchAction,
        pointer_id: u64,
        x: f32,
        y: f32,
        width: u32,
        height: u32,
        pressure: f32,
        action_button: u32,
        buttons: u32,
    },
    InjectScrollEvent {
        x: f32,
        y: f32,
        width: u32,
        height: u32,
        hscroll: f32,
        vscroll: f32,
        buttons: u32,
    },
    BackOrScreenOn {
        action: KeyAction,
    },
    SetScreenPowerMode {
        mode: ScreenPowerMode,
    },
    ExpandNotificationPanel,
    ExpandSettingsPanel,
    CollapseNotificationPanel,
    SetClipboard {
        sequence: u64,
        paste: bool,
        text: String,
    },
    RotateDevice,
}

#[derive(Debug, Clone, Copy)]
pub enum KeyAction {
    Down = 0,
    Up = 1,
}

#[derive(Debug, Clone, Copy)]
pub enum TouchAction {
    Down = 0,
    Up = 1,
    Move = 2,
}

#[derive(Debug, Clone, Copy)]
pub enum ScreenPowerMode {
    Off = 0,
    Normal = 2,
}

// Message type IDs
const TYPE_INJECT_KEYCODE: u8 = 0;
const TYPE_INJECT_TEXT: u8 = 1;
const TYPE_INJECT_TOUCH_EVENT: u8 = 2;
const TYPE_INJECT_SCROLL_EVENT: u8 = 3;
const TYPE_BACK_OR_SCREEN_ON: u8 = 4;
const TYPE_EXPAND_NOTIFICATION_PANEL: u8 = 5;
const TYPE_EXPAND_SETTINGS_PANEL: u8 = 6;
const TYPE_COLLAPSE_PANELS: u8 = 7;
const TYPE_SET_CLIPBOARD: u8 = 9;
const TYPE_SET_SCREEN_POWER_MODE: u8 = 10;
const TYPE_ROTATE_DEVICE: u8 = 11;

impl ControlMessage {
    /// Serialize this control message to bytes for the scrcpy protocol
    pub fn serialize(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();

        match self {
            ControlMessage::InjectKeycode {
                action,
                keycode,
                repeat,
                metastate,
            } => {
                buf.write_u8(TYPE_INJECT_KEYCODE)?;
                buf.write_u8(*action as u8)?;
                buf.write_u32::<BigEndian>(*keycode)?;
                buf.write_u32::<BigEndian>(*repeat)?;
                buf.write_u32::<BigEndian>(*metastate)?;
            }
            ControlMessage::InjectText { text } => {
                buf.write_u8(TYPE_INJECT_TEXT)?;
                let text_bytes = text.as_bytes();
                buf.write_u32::<BigEndian>(text_bytes.len() as u32)?;
                buf.write_all(text_bytes)?;
            }
            ControlMessage::InjectTouchEvent {
                action,
                pointer_id,
                x,
                y,
                width,
                height,
                pressure,
                action_button,
                buttons,
            } => {
                buf.write_u8(TYPE_INJECT_TOUCH_EVENT)?;
                buf.write_u8(*action as u8)?;
                buf.write_u64::<BigEndian>(*pointer_id)?;
                // Position as signed pixel coordinates (supports negative for out-of-bounds)
                buf.write_i32::<BigEndian>(*x as i32)?;
                buf.write_i32::<BigEndian>(*y as i32)?;
                buf.write_u16::<BigEndian>(*width as u16)?;
                buf.write_u16::<BigEndian>(*height as u16)?;
                let pressure_u16 = (*pressure * 65535.0) as u16;
                buf.write_u16::<BigEndian>(pressure_u16)?;
                buf.write_u32::<BigEndian>(*action_button)?;
                buf.write_u32::<BigEndian>(*buttons)?;
            }
            ControlMessage::InjectScrollEvent {
                x,
                y,
                width,
                height,
                hscroll,
                vscroll,
                buttons,
            } => {
                buf.write_u8(TYPE_INJECT_SCROLL_EVENT)?;
                buf.write_i32::<BigEndian>(*x as i32)?;
                buf.write_i32::<BigEndian>(*y as i32)?;
                buf.write_u16::<BigEndian>(*width as u16)?;
                buf.write_u16::<BigEndian>(*height as u16)?;
                buf.write_i16::<BigEndian>((*hscroll * 120.0) as i16)?;
                buf.write_i16::<BigEndian>((*vscroll * 120.0) as i16)?;
                buf.write_u32::<BigEndian>(*buttons)?;
            }
            ControlMessage::BackOrScreenOn { action } => {
                buf.write_u8(TYPE_BACK_OR_SCREEN_ON)?;
                buf.write_u8(*action as u8)?;
            }
            ControlMessage::ExpandNotificationPanel => {
                buf.write_u8(TYPE_EXPAND_NOTIFICATION_PANEL)?;
            }
            ControlMessage::ExpandSettingsPanel => {
                buf.write_u8(TYPE_EXPAND_SETTINGS_PANEL)?;
            }
            ControlMessage::CollapseNotificationPanel => {
                buf.write_u8(TYPE_COLLAPSE_PANELS)?;
            }
            ControlMessage::SetClipboard {
                sequence,
                paste,
                text,
            } => {
                buf.write_u8(TYPE_SET_CLIPBOARD)?;
                buf.write_u64::<BigEndian>(*sequence)?;
                buf.write_u8(if *paste { 1 } else { 0 })?;
                let text_bytes = text.as_bytes();
                buf.write_u32::<BigEndian>(text_bytes.len() as u32)?;
                buf.write_all(text_bytes)?;
            }
            ControlMessage::SetScreenPowerMode { mode } => {
                buf.write_u8(TYPE_SET_SCREEN_POWER_MODE)?;
                buf.write_u8(*mode as u8)?;
            }
            ControlMessage::RotateDevice => {
                buf.write_u8(TYPE_ROTATE_DEVICE)?;
            }
        }

        Ok(buf)
    }
}
