import { create } from "zustand";

export type QualityPreset = "performance" | "balanced" | "quality";

interface MirrorSettings {
  maxSize: number;
  bitRate: number;
  maxFps: number;
}

interface SettingsStore {
  preset: QualityPreset;
  settings: MirrorSettings;
  setPreset: (preset: QualityPreset) => void;
}

const presets: Record<QualityPreset, MirrorSettings> = {
  performance: {
    maxSize: 720,
    bitRate: 2_000_000,
    maxFps: 30,
  },
  balanced: {
    maxSize: 1280,
    bitRate: 4_000_000,
    maxFps: 30,
  },
  quality: {
    maxSize: 1920,
    bitRate: 8_000_000,
    maxFps: 60,
  },
};

export const useSettingsStore = create<SettingsStore>((set) => ({
  preset: (localStorage.getItem("quality_preset") as QualityPreset) || "balanced",
  settings: presets[(localStorage.getItem("quality_preset") as QualityPreset) || "balanced"],

  setPreset: (preset) => {
    localStorage.setItem("quality_preset", preset);
    set({ preset, settings: presets[preset] });
  },
}));
