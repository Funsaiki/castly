import { X, AlertCircle, CheckCircle } from "lucide-react";
import { useToastStore } from "../../stores/toastStore";

export function Toasts() {
  const { toasts, removeToast } = useToastStore();

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-10 right-4 z-50 flex flex-col gap-2 max-w-sm">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`flex items-start gap-2.5 px-4 py-3 rounded-xl shadow-lg border animate-fade-in-up ${
            toast.type === "error"
              ? "bg-danger/15 border-danger/30 text-danger"
              : toast.type === "success"
                ? "bg-success/15 border-success/30 text-success"
                : "bg-accent/15 border-accent/30 text-accent"
          }`}
        >
          {toast.type === "error" ? (
            <AlertCircle size={16} className="shrink-0 mt-0.5" />
          ) : (
            <CheckCircle size={16} className="shrink-0 mt-0.5" />
          )}
          <p className="text-[13px] leading-relaxed flex-1">{toast.message}</p>
          <button
            onClick={() => removeToast(toast.id)}
            className="shrink-0 p-0.5 rounded hover:bg-white/10 transition-colors"
          >
            <X size={12} />
          </button>
        </div>
      ))}
    </div>
  );
}
