"use client";

import * as React from "react";
import { AnimatePresence, motion } from "framer-motion";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";

interface SheetProps {
  open: boolean;
  onClose: () => void;
  title: string;
  side?: "left" | "right";
  children: React.ReactNode;
  className?: string;
  overlayClassName?: string;
  headerClassName?: string;
  bodyClassName?: string;
  closeButtonClassName?: string;
}

const Sheet = ({
  open,
  onClose,
  title,
  side = "right",
  children,
  className,
  overlayClassName,
  headerClassName,
  bodyClassName,
  closeButtonClassName,
}: SheetProps) => {
  const fromX = side === "left" ? -420 : 420;
  const titleId = React.useId();
  const closeBtnRef = React.useRef<HTMLButtonElement | null>(null);
  const previousFocusRef = React.useRef<HTMLElement | null>(null);
  const panelRef = React.useRef<HTMLDivElement | null>(null);

  React.useEffect(() => {
    if (!open) return;
    previousFocusRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const getFocusableElements = () => {
      if (!panelRef.current) return [] as HTMLElement[];
      const selector = [
        "a[href]",
        "button:not([disabled])",
        "input:not([disabled])",
        "select:not([disabled])",
        "textarea:not([disabled])",
        "[tabindex]:not([tabindex='-1'])",
      ].join(",");
      return Array.from(panelRef.current.querySelectorAll<HTMLElement>(selector)).filter(
        (element) => !element.hasAttribute("aria-hidden")
      );
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
        return;
      }
      if (event.key !== "Tab") return;
      const focusable = getFocusableElements();
      if (focusable.length === 0) {
        event.preventDefault();
        return;
      }
      const active = document.activeElement instanceof HTMLElement ? document.activeElement : null;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];

      if (!active || !panelRef.current?.contains(active)) {
        event.preventDefault();
        first.focus();
        return;
      }

      if (event.shiftKey && active === first) {
        event.preventDefault();
        last.focus();
        return;
      }
      if (!event.shiftKey && active === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", onKeyDown);
    window.setTimeout(() => {
      const [firstFocusable] = getFocusableElements();
      (firstFocusable ?? closeBtnRef.current)?.focus();
    }, 0);
    return () => {
      document.removeEventListener("keydown", onKeyDown);
      previousFocusRef.current?.focus();
    };
  }, [onClose, open]);

  return (
    <AnimatePresence>
      {open && (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
            className={cn("fixed inset-0 z-40 bg-black/30", overlayClassName)}
            aria-hidden
          />
          <motion.div
            ref={panelRef}
            initial={{ x: fromX }}
            animate={{ x: 0 }}
            exit={{ x: fromX }}
            transition={{ type: "spring", stiffness: 320, damping: 30 }}
            className={cn(
              "fixed top-0 z-50 flex h-full w-full max-w-md flex-col bg-[var(--color-surface)] shadow-[var(--shadow-soft)]",
              side === "left" ? "left-0 border-r" : "right-0 border-l",
              "border-[var(--color-line)]",
              className
            )}
            role="dialog"
            aria-modal="true"
            aria-labelledby={titleId}
            tabIndex={-1}
          >
            <div
              className={cn(
                "flex items-center justify-between border-b border-[var(--color-line)] p-4 pt-[calc(1rem+env(safe-area-inset-top))]",
                headerClassName
              )}
            >
              <span
                id={titleId}
                className="text-[11px] font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]"
              >
                {title}
              </span>
              <Button
                ref={closeBtnRef}
                variant="outline"
                size="icon"
                onClick={onClose}
                aria-label="Close"
                className={closeButtonClassName}
              >
                <X className="h-5 w-5" />
              </Button>
            </div>
            <div className={cn("flex-1 overflow-auto p-5", bodyClassName)}>{children}</div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
};

export { Sheet };
