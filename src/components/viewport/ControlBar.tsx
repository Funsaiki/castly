import { useState } from "react";
import { Camera, Circle, Unplug, Home, ArrowLeft, Layers, MonitorOff, Monitor } from "lucide-react";
import { useSessionStore } from "../../stores/sessionStore";
import { useDeviceStore } from "../../stores/deviceStore";
import { useI18n } from "../../lib/i18n";
import { pressBack, pressHome, pressRecent, setScreenPower } from "../../lib/tauri-commands";

interface ControlBarProps {
  deviceId: string;
}

export function ControlBar({ deviceId }: ControlBarProps) {
  const { sessions, toggleRecording, captureScreenshot, stopMirroring } =
    useSessionStore();
  const { t } = useI18n();
  const session = sessions[deviceId];
  const [screenOff, setScreenOff] = useState(false);

  const device = useDeviceStore((s) => s.devices.find((d) => d.id === deviceId));
  const isIos = device?.device_type === "ios";

  if (!session) return null;

  const handleScreenToggle = async () => {
    const newState = !screenOff;
    setScreenOff(newState);
    await setScreenPower(deviceId, !newState);
  };

  return (
    <div className="flex items-center justify-center gap-0.5 px-3 py-2 bg-surface-alt/90 backdrop-blur-md border-t border-border animate-fade-in-up">
      {!isIos && (
        <>
          <ControlButton
            icon={<ArrowLeft size={16} />}
            label={t("control.back")}
            onClick={() => pressBack(deviceId)}
          />
          <ControlButton
            icon={<Home size={16} />}
            label={t("control.home")}
            onClick={() => pressHome(deviceId)}
          />
          <ControlButton
            icon={<Layers size={16} />}
            label={t("control.recent")}
            onClick={() => pressRecent(deviceId)}
          />
          <Separator />
        </>
      )}

      <ControlButton
        icon={
          <Circle
            size={15}
            className={session.is_recording ? "fill-danger text-danger" : ""}
          />
        }
        label={session.is_recording ? t("control.stop_record") : t("control.record")}
        active={session.is_recording}
        onClick={() => toggleRecording(deviceId)}
      />
      <ControlButton
        icon={<Camera size={16} />}
        label={t("control.screenshot")}
        onClick={() => captureScreenshot(deviceId)}
      />

      <Separator />

      {!isIos && (
        <ControlButton
          icon={screenOff ? <Monitor size={16} /> : <MonitorOff size={16} />}
          label={screenOff ? t("control.screen_on") : t("control.screen_off")}
          active={screenOff}
          onClick={handleScreenToggle}
        />
      )}
      <ControlButton
        icon={<Unplug size={16} />}
        label={t("control.disconnect")}
        danger
        onClick={() => stopMirroring(deviceId)}
      />
    </div>
  );
}

function Separator() {
  return <div className="w-px h-5 bg-border mx-1.5" />;
}

interface ControlButtonProps {
  icon: React.ReactNode;
  label: string;
  active?: boolean;
  danger?: boolean;
  onClick: () => void;
}

function ControlButton({ icon, label, active, danger, onClick }: ControlButtonProps) {
  return (
    <button
      onClick={onClick}
      title={label}
      className={`relative p-2 rounded-lg transition-all duration-150 press-feedback group ${
        active
          ? "bg-accent/15 text-accent"
          : danger
            ? "text-text-muted hover:bg-danger/10 hover:text-danger"
            : "text-text-muted hover:bg-surface-hover hover:text-text-primary"
      }`}
    >
      {icon}
      <span className="absolute -top-8 left-1/2 -translate-x-1/2 px-2 py-0.5 rounded text-[10px] bg-surface-active text-text-secondary whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none shadow-md">
        {label}
      </span>
    </button>
  );
}
