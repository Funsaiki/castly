import { useCallback, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X, MonitorSmartphone, HelpCircle } from "lucide-react";
import { useI18n } from "../../lib/i18n";
import { HelpDialog } from "./HelpDialog";

export function Titlebar() {
  const { t, locale, setLocale } = useI18n();
  const [helpOpen, setHelpOpen] = useState(false);

  const handleMinimize = useCallback(async () => {
    await getCurrentWindow().minimize();
  }, []);

  const handleMaximize = useCallback(async () => {
    await getCurrentWindow().toggleMaximize();
  }, []);

  const handleClose = useCallback(async () => {
    await getCurrentWindow().close();
  }, []);

  return (
    <div
      className="flex items-center justify-between h-10 bg-surface-alt border-b border-border select-none"
      onMouseDown={(e) => {
        if ((e.target as HTMLElement).closest("button")) return;
        getCurrentWindow().startDragging();
      }}
    >
      {/* Left: logo + title */}
      <div className="flex items-center gap-2.5 px-4">
        <div className="w-5 h-5 rounded-md bg-accent/90 flex items-center justify-center shadow-sm">
          <MonitorSmartphone size={12} className="text-white" />
        </div>
        <span className="text-[13px] font-semibold tracking-tight text-text-secondary">
          Castly
        </span>
      </div>

      {/* Right: language + window controls */}
      <HelpDialog open={helpOpen} onClose={() => setHelpOpen(false)} />

      <div className="flex items-center">
        {/* Help button */}
        <button
          onClick={() => setHelpOpen(true)}
          className="p-1.5 mr-1 rounded-md hover:bg-surface-hover transition-colors"
          title={t("help.title")}
        >
          <HelpCircle size={14} className="text-text-muted" />
        </button>

        {/* Language toggle */}
        <div className="flex items-center mr-2 bg-surface/60 rounded-md p-0.5">
          <button
            onClick={() => setLocale("fr")}
            className={`text-[10px] px-2 py-0.5 rounded transition-all duration-150 ${
              locale === "fr"
                ? "bg-surface-hover text-text-primary font-semibold shadow-sm"
                : "text-text-muted hover:text-text-secondary"
            }`}
          >
            FR
          </button>
          <button
            onClick={() => setLocale("en")}
            className={`text-[10px] px-2 py-0.5 rounded transition-all duration-150 ${
              locale === "en"
                ? "bg-surface-hover text-text-primary font-semibold shadow-sm"
                : "text-text-muted hover:text-text-secondary"
            }`}
          >
            EN
          </button>
        </div>

        {/* Window controls */}
        <button
          onClick={handleMinimize}
          className="w-12 h-10 flex items-center justify-center hover:bg-surface-hover transition-colors duration-150"
        >
          <Minus size={15} className="text-text-muted" />
        </button>
        <button
          onClick={handleMaximize}
          className="w-12 h-10 flex items-center justify-center hover:bg-surface-hover transition-colors duration-150"
        >
          <Square size={11} className="text-text-muted" />
        </button>
        <button
          onClick={handleClose}
          className="w-12 h-10 flex items-center justify-center hover:bg-danger transition-colors duration-150 group"
        >
          <X size={15} className="text-text-muted group-hover:text-white" />
        </button>
      </div>
    </div>
  );
}
