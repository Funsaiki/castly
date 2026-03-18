import { create } from "zustand";
import type { MirrorSession } from "../lib/types";
import type { MsePlayer } from "../lib/mse-player";
import { startMirror, stopMirror } from "../lib/tauri-commands";
import { useSettingsStore } from "./settingsStore";

interface SessionStore {
  sessions: Record<string, MirrorSession>;
  activeDeviceId: string | null;
  isConnecting: boolean;
  /** Reference to the active player for screenshot/recording */
  playerRef: MsePlayer | null;

  startMirroring: (deviceId: string) => Promise<void>;
  stopMirroring: (deviceId: string) => Promise<void>;
  setPlayer: (player: MsePlayer | null) => void;
  toggleRecording: (deviceId: string) => void;
  captureScreenshot: (deviceId: string) => Promise<void>;
  getActiveSession: () => MirrorSession | null;
}

export const useSessionStore = create<SessionStore>((set, get) => ({
  sessions: {},
  activeDeviceId: null,
  isConnecting: false,
  playerRef: null,

  startMirroring: async (deviceId) => {
    set({ isConnecting: true, activeDeviceId: deviceId });
    try {
      const { settings } = useSettingsStore.getState();
      const session = await startMirror(deviceId, settings);
      set((state) => ({
        sessions: { ...state.sessions, [deviceId]: session },
        isConnecting: false,
      }));
    } catch (e) {
      console.error("Failed to start mirroring:", e);
      set({ isConnecting: false, activeDeviceId: null });
      throw e;
    }
  },

  stopMirroring: async (deviceId) => {
    // Stop recording if active
    const player = get().playerRef;
    if (player?.isRecording) {
      player.stopRecording();
    }

    try {
      await stopMirror(deviceId);
      set((state) => {
        const sessions = { ...state.sessions };
        delete sessions[deviceId];
        return {
          sessions,
          activeDeviceId:
            state.activeDeviceId === deviceId ? null : state.activeDeviceId,
          playerRef: null,
        };
      });
    } catch (e) {
      console.error("Failed to stop mirroring:", e);
    }
  },

  setPlayer: (player) => {
    set({ playerRef: player });
  },

  toggleRecording: (deviceId) => {
    const player = get().playerRef;
    const session = get().sessions[deviceId];
    if (!player || !session) return;

    if (player.isRecording) {
      player.stopRecording();
      set((state) => ({
        sessions: {
          ...state.sessions,
          [deviceId]: { ...session, is_recording: false },
        },
      }));
    } else {
      player.startRecording();
      set((state) => ({
        sessions: {
          ...state.sessions,
          [deviceId]: { ...session, is_recording: true },
        },
      }));
    }
  },

  captureScreenshot: async (_deviceId) => {
    const player = get().playerRef;
    if (!player) return;
    await player.takeScreenshot();
  },

  getActiveSession: () => {
    const { sessions, activeDeviceId } = get();
    return activeDeviceId ? sessions[activeDeviceId] ?? null : null;
  },
}));
