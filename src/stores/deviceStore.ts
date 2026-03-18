import { create } from "zustand";
import type { DeviceInfo } from "../lib/types";
import { scanDevices, connectDevice, disconnectDevice } from "../lib/tauri-commands";

interface DeviceStore {
  devices: DeviceInfo[];
  selectedDeviceId: string | null;
  scanning: boolean;
  _intervalId: ReturnType<typeof setInterval> | null;

  startContinuousScan: () => void;
  stopContinuousScan: () => void;
  scanNow: () => Promise<void>;
  selectDevice: (id: string | null) => void;
  connect: (id: string) => Promise<void>;
  disconnect: (id: string) => Promise<void>;
}

export const useDeviceStore = create<DeviceStore>((set, get) => ({
  devices: [],
  selectedDeviceId: null,
  scanning: false,
  _intervalId: null,

  startContinuousScan: () => {
    if (get()._intervalId) return; // Already scanning

    const doScan = async () => {
      try {
        const devices = await scanDevices();
        set({ devices, scanning: true });
      } catch (e) {
        console.error("Scan failed:", e);
      }
    };

    doScan();
    const id = setInterval(doScan, 3000);
    set({ _intervalId: id, scanning: true });
  },

  stopContinuousScan: () => {
    const id = get()._intervalId;
    if (id) {
      clearInterval(id);
      set({ _intervalId: null, scanning: false });
    }
  },

  scanNow: async () => {
    try {
      const devices = await scanDevices();
      set({ devices });
    } catch (e) {
      console.error("Scan failed:", e);
    }
  },

  selectDevice: (id) => {
    set({ selectedDeviceId: id });
  },

  connect: async (id) => {
    try {
      const device = await connectDevice(id);
      const devices = get().devices.map((d) => (d.id === id ? device : d));
      set({ devices });
    } catch (e) {
      console.error("Failed to connect:", e);
    }
  },

  disconnect: async (id) => {
    try {
      await disconnectDevice(id);
      const devices = get().devices.map((d) =>
        d.id === id ? { ...d, status: "disconnected" as const } : d,
      );
      set({ devices });
    } catch (e) {
      console.error("Failed to disconnect:", e);
    }
  },
}));
