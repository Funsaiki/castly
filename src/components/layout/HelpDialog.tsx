import { useState } from "react";
import { X, Usb, Wifi, Airplay } from "lucide-react";
import { useI18n } from "../../lib/i18n";

interface HelpDialogProps {
  open: boolean;
  onClose: () => void;
}

type Tab = "usb" | "wifi" | "airplay";

export function HelpDialog({ open, onClose }: HelpDialogProps) {
  const { t } = useI18n();
  const [tab, setTab] = useState<Tab>("usb");

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 animate-fade-in">
      <div className="bg-surface-alt border border-border rounded-2xl shadow-lg w-[800px] max-h-[85vh] flex flex-col animate-scale-in">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-border">
          <h2 className="text-base font-semibold">{t("help.title")}</h2>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg hover:bg-surface-hover transition-colors"
          >
            <X size={16} className="text-text-muted" />
          </button>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-border px-5">
          <TabButton
            active={tab === "usb"}
            icon={<Usb size={14} />}
            label={t("help.tab_usb")}
            onClick={() => setTab("usb")}
          />
          <TabButton
            active={tab === "wifi"}
            icon={<Wifi size={14} />}
            label={t("help.tab_wifi")}
            onClick={() => setTab("wifi")}
          />
          <TabButton
            active={tab === "airplay"}
            icon={<Airplay size={14} />}
            label={t("help.tab_airplay")}
            onClick={() => setTab("airplay")}
          />
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-5 py-4">
          {tab === "usb" && <UsbGuide />}
          {tab === "wifi" && <WifiGuide />}
          {tab === "airplay" && <AirPlayGuide />}
        </div>
      </div>
    </div>
  );
}

function TabButton({
  active,
  icon,
  label,
  onClick,
}: {
  active: boolean;
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2 px-4 py-2.5 text-[13px] font-medium border-b-2 transition-colors ${
        active
          ? "border-accent text-accent"
          : "border-transparent text-text-muted hover:text-text-secondary"
      }`}
    >
      {icon}
      {label}
    </button>
  );
}

function UsbGuide() {
  const { t } = useI18n();
  return (
    <div className="space-y-4">
      <p className="text-sm text-text-secondary leading-relaxed">
        {t("help.usb_intro")}
      </p>
      <StepList
        steps={[
          t("help.usb_step1"),
          t("help.usb_step2"),
          t("help.usb_step3"),
          t("help.usb_step4"),
          t("help.usb_step5"),
          t("help.usb_step6"),
          t("help.usb_step7"),
        ]}
      />
      {/* Placeholder for screenshot */}
      <ImagePlaceholder id="usb-debug" label={t("help.usb_screenshot")} />
    </div>
  );
}

function WifiGuide() {
  const { t, locale } = useI18n();
  return (
    <div className="space-y-3">
      <p className="text-sm text-text-secondary leading-relaxed">
        {t("help.wifi_intro")}
      </p>
      <div className="flex gap-4">
        <div className="flex-1 min-w-0">
          <StepList
            steps={[
              t("help.wifi_step1"),
              t("help.wifi_step2"),
              t("help.wifi_step3"),
              t("help.wifi_step4"),
              t("help.wifi_step5"),
              t("help.wifi_step6"),
            ]}
          />
        </div>
        <div className="shrink-0 w-48">
          {locale === "fr" ? (
            <img
              src="/wifi-fr.jpg"
              alt={t("help.wifi_screenshot")}
              className="w-full rounded-lg border border-border bg-black sticky top-0"
            />
          ) : (
            <ImagePlaceholder id="wifi-pair" label={t("help.wifi_screenshot")} />
          )}
        </div>
      </div>
    </div>
  );
}

function AirPlayGuide() {
  const { t } = useI18n();
  return (
    <div className="space-y-4">
      <p className="text-sm text-text-secondary leading-relaxed">
        {t("help.airplay_intro")}
      </p>
      <StepList
        steps={[
          t("help.airplay_step1"),
          t("help.airplay_step2"),
          t("help.airplay_step3"),
          t("help.airplay_step4"),
        ]}
      />
      <div className="p-3 rounded-lg bg-accent/10 border border-accent/20">
        <p className="text-xs text-accent">{t("help.airplay_note")}</p>
      </div>
      <ImagePlaceholder id="airplay-mirror" label={t("help.airplay_screenshot")} />
    </div>
  );
}

function StepList({ steps }: { steps: string[] }) {
  return (
    <ol className="space-y-2.5">
      {steps.map((step, i) => (
        <li key={i} className="flex gap-3 text-sm">
          <span className="shrink-0 w-6 h-6 rounded-full bg-accent/15 text-accent text-xs font-bold flex items-center justify-center">
            {i + 1}
          </span>
          <span className="text-text-secondary leading-relaxed pt-0.5">{step}</span>
        </li>
      ))}
    </ol>
  );
}

function ImagePlaceholder({ id, label }: { id: string; label: string }) {
  return (
    <div
      className="w-full h-40 rounded-lg border border-dashed border-border-light/50 bg-surface flex items-center justify-center"
      data-screenshot-id={id}
    >
      <p className="text-xs text-text-muted/50">{label}</p>
    </div>
  );
}
