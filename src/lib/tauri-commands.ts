import { invoke } from "@tauri-apps/api/core";
import type { DeviceInfo, MirrorSession } from "./types";

export async function listDevices(): Promise<DeviceInfo[]> {
  return invoke("list_devices");
}

export async function scanDevices(): Promise<DeviceInfo[]> {
  return invoke("scan_devices");
}

export async function connectDevice(deviceId: string): Promise<DeviceInfo> {
  return invoke("connect_device", { deviceId });
}

export async function disconnectDevice(deviceId: string): Promise<void> {
  return invoke("disconnect_device", { deviceId });
}

export async function startMirror(
  deviceId: string,
  config?: { maxSize: number; bitRate: number; maxFps: number },
): Promise<MirrorSession> {
  return invoke("start_mirror", { deviceId, config });
}

export async function stopMirror(deviceId: string): Promise<void> {
  return invoke("stop_mirror", { deviceId });
}

export async function startRecording(deviceId: string): Promise<string> {
  return invoke("start_recording", { deviceId });
}

export async function stopRecording(deviceId: string): Promise<string> {
  return invoke("stop_recording", { deviceId });
}

export async function takeScreenshot(deviceId: string): Promise<string> {
  return invoke("take_screenshot", { deviceId });
}

export async function connectWifi(deviceId: string): Promise<string> {
  return invoke("connect_wifi", { deviceId });
}

export async function connectWifiIp(ip: string, port?: number): Promise<void> {
  return invoke("connect_wifi_ip", { ip, port });
}

export async function pairWifi(ip: string, pairPort: number, code: string, connectPort: number): Promise<void> {
  return invoke("pair_wifi", { ip, pairPort, code, connectPort });
}

export async function injectTouch(
  deviceId: string,
  action: "down" | "up" | "move",
  x: number,
  y: number,
  screenWidth: number,
  screenHeight: number,
): Promise<void> {
  return invoke("inject_touch", { deviceId, action, x, y, screenWidth, screenHeight });
}

export async function injectKey(
  deviceId: string,
  action: "down" | "up",
  keycode: number,
): Promise<void> {
  return invoke("inject_key", { deviceId, action, keycode });
}

export async function injectScroll(
  deviceId: string,
  x: number,
  y: number,
  screenWidth: number,
  screenHeight: number,
  hscroll: number,
  vscroll: number,
): Promise<void> {
  return invoke("inject_scroll", { deviceId, x, y, screenWidth, screenHeight, hscroll, vscroll });
}

export async function pressBack(deviceId: string): Promise<void> {
  return invoke("press_back", { deviceId });
}

export async function pressHome(deviceId: string): Promise<void> {
  return invoke("press_home", { deviceId });
}

export async function pressRecent(deviceId: string): Promise<void> {
  return invoke("press_recent", { deviceId });
}

export async function setScreenPower(deviceId: string, on: boolean): Promise<void> {
  return invoke("set_screen_power", { deviceId, on });
}
