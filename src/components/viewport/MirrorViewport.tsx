import { Smartphone, Loader2 } from "lucide-react";
import { useSessionStore } from "../../stores/sessionStore";
import { useI18n } from "../../lib/i18n";
import { VideoPlayer } from "./VideoPlayer";
import { ControlBar } from "./ControlBar";

export function MirrorViewport() {
  const activeDeviceId = useSessionStore((s) => s.activeDeviceId);
  const sessions = useSessionStore((s) => s.sessions);
  const isConnecting = useSessionStore((s) => s.isConnecting);
  const session = activeDeviceId ? sessions[activeDeviceId] ?? null : null;
  const { t } = useI18n();

  // State: connecting (loading spinner)
  if (isConnecting && !session) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center bg-surface">
        <div className="flex flex-col items-center gap-5 animate-fade-in">
          <div className="w-20 h-20 rounded-2xl border border-accent/20 flex items-center justify-center bg-accent/5 animate-pulse-glow">
            <Loader2 size={32} className="text-accent animate-spin" />
          </div>
          <div className="text-center">
            <p className="text-sm font-medium text-text-secondary">
              {t("viewport.connecting")}
            </p>
            <p className="text-xs mt-1.5 text-text-muted leading-relaxed">
              {t("viewport.connecting_hint")}
            </p>
          </div>
        </div>
      </div>
    );
  }

  // State: no active session (idle)
  if (!session) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center bg-surface">
        <div className="flex flex-col items-center gap-5 text-text-muted animate-fade-in">
          <div className="w-20 h-20 rounded-2xl border border-dashed border-border-light/50 flex items-center justify-center bg-surface-alt/50">
            <Smartphone size={36} strokeWidth={1} className="opacity-30" />
          </div>
          <div className="text-center">
            <p className="text-sm font-medium text-text-secondary">
              {t("viewport.no_mirror")}
            </p>
            <p className="text-xs mt-1.5 text-text-muted leading-relaxed">
              {t("viewport.hint")}
            </p>
          </div>
        </div>
      </div>
    );
  }

  // State: mirroring
  return (
    <div className="flex-1 flex flex-col bg-black relative min-w-0 min-h-0 animate-fade-in">
      <div className="flex-1 flex items-center justify-center p-3 min-h-0 overflow-hidden">
        <VideoPlayer streamUrl={session.stream_url} />
      </div>
      <ControlBar deviceId={session.device_id} />
    </div>
  );
}
