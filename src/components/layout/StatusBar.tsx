import { useSessionStore } from "../../stores/sessionStore";
import { useDeviceStore } from "../../stores/deviceStore";
import { useI18n } from "../../lib/i18n";
import { Wifi, Usb, Circle } from "lucide-react";

export function StatusBar() {
  const { activeDeviceId, sessions } = useSessionStore();
  const { devices } = useDeviceStore();
  const { t } = useI18n();

  const activeDevice = activeDeviceId
    ? devices.find((d) => d.id === activeDeviceId)
    : null;
  const activeSession = activeDeviceId ? sessions[activeDeviceId] : null;

  return (
    <div className="h-7 bg-surface-alt border-t border-border flex items-center px-4 gap-5">
      {activeDevice && activeSession ? (
        <>
          <div className="flex items-center gap-1.5">
            <span className="inline-block w-1.5 h-1.5 rounded-full bg-success animate-pulse" />
            <span className="text-[11px] text-text-secondary font-medium">
              {activeDevice.name}
            </span>
          </div>
          <div className="flex items-center gap-1">
            {activeDevice.connection === "usb" ? (
              <Usb size={10} className="text-text-muted" />
            ) : (
              <Wifi size={10} className="text-text-muted" />
            )}
            <span className="text-[11px] text-text-muted">
              {activeDevice.connection === "usb" ? "USB" : "Wi-Fi"}
            </span>
          </div>
          {activeSession.screen_width > 0 && (
            <span className="text-[11px] text-text-muted">
              {activeSession.screen_width}x{activeSession.screen_height}
            </span>
          )}
          {activeSession.is_recording && (
            <div className="flex items-center gap-1.5 animate-recording">
              <Circle size={8} className="text-danger fill-danger" />
              <span className="text-[11px] text-danger font-medium">{t("status.rec")}</span>
            </div>
          )}
        </>
      ) : (
        <span className="text-[11px] text-text-muted">
          {t("status.ready")}
        </span>
      )}
      <span className="ml-auto text-[10px] text-text-muted/60">v0.1.0</span>
    </div>
  );
}
