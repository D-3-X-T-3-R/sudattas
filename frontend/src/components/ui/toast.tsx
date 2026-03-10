"use client";

import { createContext, useCallback, useContext, useMemo, useState } from "react";

type ToastVariant = "default" | "success" | "error";

export type Toast = {
  id: number;
  title?: string;
  description: string;
  variant?: ToastVariant;
  /** When set, any existing toast with the same group is removed before showing this one (e.g. "wishlist" = only one wishlist toast at a time). */
  group?: string;
};

type ToastInput = Omit<Toast, "id">;

type ToastContextValue = {
  toasts: Toast[];
  showToast: (toast: ToastInput) => void;
  dismissToast: (id: number) => void;
};

const ToastContext = createContext<ToastContextValue | null>(null);

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const dismissToast = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const showToast = useCallback(
    (toast: ToastInput) => {
      const id = Date.now();
      setToasts((prev) => {
        const next = toast.group
          ? prev.filter((t) => t.group !== toast.group)
          : prev;
        return [...next, { id, ...toast }];
      });
      window.setTimeout(() => dismissToast(id), 3000);
    },
    [dismissToast]
  );

  const value = useMemo(
    () => ({
      toasts,
      showToast,
      dismissToast,
    }),
    [toasts, showToast, dismissToast]
  );

  return (
    <ToastContext.Provider value={value}>
      {children}
      <div
        aria-live="polite"
        aria-atomic="true"
        className="pointer-events-none fixed inset-x-0 top-4 z-[60] flex justify-end px-4 sm:px-6"
      >
        <div className="flex w-full max-w-sm flex-col gap-3">
          {toasts.map((toast) => (
            <div key={toast.id} className="toast-item pointer-events-auto">
              <div className="toast-body">
                <div className="toast-icon" aria-hidden>✓</div>
                <div className="toast-content">
                  {toast.title && (
                    <div className="toast-title">{toast.title}</div>
                  )}
                  <div className="toast-description">{toast.description}</div>
                </div>
                <button
                  type="button"
                  onClick={() => dismissToast(toast.id)}
                  className="toast-close"
                  aria-label="Dismiss notification"
                >
                  ×
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </ToastContext.Provider>
  );
}

export function useToast() {
  const ctx = useContext(ToastContext);
  if (!ctx) {
    throw new Error("useToast must be used within ToastProvider");
  }
  return ctx;
}

