import { useState } from "react";
import { Smartphone, Wifi, Usb, Monitor, Search, Zap, Scale, Sparkles } from "lucide-react";
import { useDeviceStore } from "../../stores/deviceStore";
import { useSessionStore } from "../../stores/sessionStore";
import { useSettingsStore, type QualityPreset } from "../../stores/settingsStore";
import { useI18n } from "../../lib/i18n";
import { pairWifi } from "../../lib/tauri-commands";
import { showError, showSuccess } from "../../stores/toastStore";

export function Sidebar() {
  const { devices, selectedDeviceId, scanNow, selectDevice } = useDeviceStore();
  const { sessions, startMirroring, stopMirroring } = useSessionStore();
  const { t } = useI18n();
  const { preset, setPreset } = useSettingsStore();
  const hasActiveSession = Object.keys(sessions).length > 0;

  const [wifiIp, setWifiIp] = useState("");
  const [pairPort, setPairPort] = useState("");
  const [pairCode, setPairCode] = useState("");
  const [connectPort, setConnectPort] = useState("");
  const [wifiStatus, setWifiStatus] = useState<"idle" | "connecting" | "success" | "error">("idle");

  const handleMirrorToggle = async (deviceId: string) => {
    try {
      if (sessions[deviceId]) {
        await stopMirroring(deviceId);
      } else {
        await startMirroring(deviceId);
      }
    } catch {
      // Error already shown via toast in sessionStore
    }
  };

  const handlePair = async () => {
    if (!wifiIp.trim() || !pairPort.trim() || !pairCode.trim() || !connectPort.trim()) return;
    setWifiStatus("connecting");
    try {
      const ip = wifiIp.trim();
      const pPort = parseInt(pairPort.trim(), 10);
      const cPort = parseInt(connectPort.trim(), 10);
      if (isNaN(pPort) || isNaN(cPort)) throw new Error("Ports invalides");

      await pairWifi(ip, pPort, pairCode.trim(), cPort);
      setWifiStatus("success");
      setWifiIp("");
      setPairPort("");
      setPairCode("");
      setConnectPort("");
      await scanNow();
      showSuccess(t("success.pairing"));
      setTimeout(() => setWifiStatus("idle"), 2000);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      showError(`${t("error.pairing_failed")}: ${msg}`);
      setWifiStatus("error");
      setTimeout(() => setWifiStatus("idle"), 3000);
    }
  };

  return (
    <div className="w-60 bg-surface-alt border-r border-border flex flex-col h-full">
      {/* Header */}
      <div className="px-4 py-3 border-b border-border flex items-center justify-between">
        <h2 className="text-[11px] font-semibold text-text-muted uppercase tracking-widest">
          {t("sidebar.devices")}
        </h2>
        <button
          onClick={scanNow}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[11px] font-semibold transition-all duration-200 press-feedback bg-accent text-white hover:bg-accent-hover shadow-sm"
        >
          <Search size={12} />
          {t("sidebar.scan")}
        </button>
      </div>

      {/* Quality selector — animated slide */}
      <div
        className="grid transition-all duration-300 ease-in-out border-b border-border"
        style={{
          gridTemplateRows: hasActiveSession ? "0fr" : "1fr",
          borderColor: hasActiveSession ? "transparent" : undefined,
        }}
      >
        <div className="overflow-hidden">
          <div className="px-3 py-2.5">
            <p className="text-[10px] font-semibold text-text-muted uppercase tracking-wider mb-2">
              {t("settings.quality")}
            </p>
            <div className="flex gap-1">
              {([
                { key: "performance" as QualityPreset, icon: <Zap size={12} /> },
                { key: "balanced" as QualityPreset, icon: <Scale size={12} /> },
                { key: "quality" as QualityPreset, icon: <Sparkles size={12} /> },
              ]).map(({ key, icon }) => (
                <button
                  key={key}
                  onClick={() => setPreset(key)}
                  title={t(`settings.${key === "quality" ? "quality_preset" : key}_desc`)}
                  className={`flex-1 flex flex-col items-center gap-1 py-1.5 rounded-lg text-[10px] transition-all duration-150 press-feedback ${
                    preset === key
                      ? "bg-accent/15 text-accent border border-accent/25"
                      : "text-text-muted hover:bg-surface-hover border border-transparent"
                  }`}
                >
                  {icon}
                  <span className="font-medium">
                    {t(`settings.${key === "quality" ? "quality_preset" : key}`)}
                  </span>
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Device list */}
      <div className="flex-1 overflow-y-auto p-2 space-y-1">
        {devices.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-text-muted animate-fade-in">
            <div className="w-14 h-14 rounded-2xl border border-border bg-surface flex items-center justify-center mb-3">
              <Smartphone size={24} strokeWidth={1.5} className="opacity-40" />
            </div>
            <p className="text-xs text-center leading-relaxed">
              {t("sidebar.no_device")}
            </p>
            <p className="text-[10px] text-text-muted/50 mt-2">
              {t("sidebar.help_hint")}
            </p>
          </div>
        ) : (
          devices.map((device, index) => {
            const isMirroring = !!sessions[device.id];
            const isSelected = selectedDeviceId === device.id;

            return (
              <button
                key={device.id}
                onClick={() => selectDevice(device.id)}
                onDoubleClick={() => handleMirrorToggle(device.id)}
                className={`w-full text-left p-3 rounded-xl transition-all duration-200 press-feedback ${
                  isSelected
                    ? "bg-accent/10 border border-accent/25 shadow-sm"
                    : "hover:bg-surface-hover border border-transparent"
                }`}
                style={{ animationDelay: `${index * 50}ms` }}
              >
                <div className="flex items-center gap-2.5">
                  <div className="relative">
                    <div
                      className={`w-8 h-8 rounded-lg flex items-center justify-center transition-colors ${
                        isMirroring
                          ? "bg-success/15 text-success"
                          : "bg-surface text-text-muted"
                      }`}
                    >
                      {device.connection === "usb" ? (
                        <Usb size={15} />
                      ) : (
                        <Wifi size={15} />
                      )}
                    </div>
                    <div
                      className={`absolute -bottom-0.5 -right-0.5 w-2.5 h-2.5 rounded-full border-2 border-surface-alt transition-colors ${
                        isMirroring
                          ? "bg-success"
                          : device.status === "connected"
                            ? "bg-accent"
                            : "bg-text-muted"
                      }`}
                    />
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-[13px] font-medium truncate leading-tight">
                      {device.name}
                    </p>
                    <p className="text-[10px] text-text-muted truncate mt-0.5">
                      {device.device_type === "android" ? "Android" : "iOS"}{" "}
                      {device.connection === "usb" ? "USB" : "Wi-Fi"}
                    </p>
                  </div>
                  {isMirroring && (
                    <Monitor size={14} className="text-success animate-fade-in" />
                  )}
                </div>
              </button>
            );
          })
        )}
      </div>

      {/* Wi-Fi pairing section */}
      <div
        className="grid transition-all duration-300 ease-in-out border-t border-border"
        style={{
          gridTemplateRows: hasActiveSession ? "0fr" : "1fr",
          borderColor: hasActiveSession ? "transparent" : undefined,
        }}
      >
        <div className="overflow-hidden">
          <div className="px-2 py-2 space-y-1.5">
            <p className="text-[10px] font-semibold text-text-muted uppercase tracking-wider px-1">
              {t("sidebar.wifi")}
            </p>

            <div className="grid grid-cols-[1fr_auto_auto] gap-1">
              <input
                type="text"
                value={wifiIp}
                onChange={(e) => setWifiIp(e.target.value)}
                placeholder="IP"
                className="min-w-0 px-2 py-1 rounded bg-surface border border-border text-[11px] text-text-primary placeholder:text-text-muted/40 focus:outline-none focus:border-accent/50"
              />
              <input
                type="text"
                value={pairPort}
                onChange={(e) => setPairPort(e.target.value)}
                placeholder="P.assoc"
                className="w-14 px-1.5 py-1 rounded bg-surface border border-border text-[11px] text-text-primary placeholder:text-text-muted/40 focus:outline-none focus:border-accent/50"
              />
              <input
                type="text"
                value={connectPort}
                onChange={(e) => setConnectPort(e.target.value)}
                placeholder="P.conn"
                className="w-14 px-1.5 py-1 rounded bg-surface border border-border text-[11px] text-text-primary placeholder:text-text-muted/40 focus:outline-none focus:border-accent/50"
              />
            </div>
            <div className="flex gap-1">
              <input
                type="text"
                value={pairCode}
                onChange={(e) => setPairCode(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handlePair()}
                placeholder={t("sidebar.wifi_pair_code")}
                className="flex-1 min-w-0 px-2 py-1 rounded bg-surface border border-border text-[11px] text-text-primary placeholder:text-text-muted/40 focus:outline-none focus:border-accent/50"
              />
              <button
                onClick={handlePair}
                disabled={!wifiIp.trim() || !pairPort.trim() || !pairCode.trim() || !connectPort.trim() || wifiStatus === "connecting"}
                className={`px-3 py-1 rounded text-[11px] font-semibold transition-all press-feedback shrink-0 ${
                  wifiStatus === "success"
                    ? "bg-success/20 text-success"
                    : wifiStatus === "error"
                      ? "bg-danger/20 text-danger"
                      : wifiStatus === "connecting"
                        ? "bg-accent/20 text-accent"
                        : "bg-accent text-white hover:bg-accent-hover disabled:opacity-40"
                }`}
              >
                {wifiStatus === "connecting"
                  ? "..."
                  : wifiStatus === "success"
                    ? "OK"
                    : wifiStatus === "error"
                      ? "!"
                      : t("sidebar.wifi_pair")}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Footer hint */}
      {devices.length > 0 && !hasActiveSession && (
        <div className="px-3 py-2 border-t border-border">
          <p className="text-[10px] text-text-muted/60 text-center">
            {t("sidebar.hint")}
          </p>
        </div>
      )}
    </div>
  );
}
