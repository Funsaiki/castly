export interface DeviceInfo {
  id: string;
  name: string;
  device_type: "android" | "ios";
  connection: "usb" | "wifi";
  status: "disconnected" | "connecting" | "connected" | "mirroring";
  screen_width: number;
  screen_height: number;
}

export interface MirrorSession {
  device_id: string;
  stream_url: string;
  screen_width: number;
  screen_height: number;
  audio_codec: string;
  is_recording: boolean;
  recording_path: string | null;
}

export interface MirrorConfig {
  max_size: number;
  bit_rate: number;
  max_fps: number;
  codec: string;
}

export interface TouchEvent {
  action: "down" | "up" | "move";
  x: number;
  y: number;
  viewport_width: number;
  viewport_height: number;
}

export interface KeyEvent {
  action: "down" | "up";
  key_code: string;
  metastate: number;
}

export interface ScrollEvent {
  x: number;
  y: number;
  viewport_width: number;
  viewport_height: number;
  hscroll: number;
  vscroll: number;
}
