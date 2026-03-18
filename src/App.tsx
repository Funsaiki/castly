import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { Titlebar } from "./components/layout/Titlebar";
import { Sidebar } from "./components/layout/Sidebar";
import { StatusBar } from "./components/layout/StatusBar";
import { MirrorViewport } from "./components/viewport/MirrorViewport";
import { useDeviceStore } from "./stores/deviceStore";
import type { DeviceInfo } from "./lib/types";
import "./styles/animations.css";

function App() {
  const startContinuousScan = useDeviceStore((s) => s.startContinuousScan);
  const stopContinuousScan = useDeviceStore((s) => s.stopContinuousScan);

  useEffect(() => {
    startContinuousScan();
    return () => stopContinuousScan();
  }, [startContinuousScan, stopContinuousScan]);

  // Listen for AirPlay device events from backend
  useEffect(() => {
    const unConnect = listen<DeviceInfo>("airplay-device-connected", (e) => {
      useDeviceStore.setState((state) => ({
        devices: [...state.devices.filter((d) => d.id !== e.payload.id), e.payload],
      }));
    });
    const unDisconnect = listen<string>("airplay-device-disconnected", (e) => {
      useDeviceStore.setState((state) => ({
        devices: state.devices.filter((d) => d.id !== e.payload),
      }));
    });
    return () => {
      unConnect.then((fn) => fn());
      unDisconnect.then((fn) => fn());
    };
  }, []);

  return (
    <div className="flex flex-col h-screen">
      <Titlebar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <MirrorViewport />
      </div>
      <StatusBar />
    </div>
  );
}

export default App;
