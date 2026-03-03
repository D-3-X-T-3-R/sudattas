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
}

const Sheet = ({
  open,
  onClose,
  title,
  side = "right",
  children,
  className,
}: SheetProps) => {
  const fromX = side === "left" ? -420 : 420;

  return (
    <AnimatePresence>
      {open && (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
            className="fixed inset-0 z-40 bg-black/40"
            aria-hidden
          />
          <motion.div
            initial={{ x: fromX }}
            animate={{ x: 0 }}
            exit={{ x: fromX }}
            transition={{ type: "spring", stiffness: 320, damping: 30 }}
            className={cn(
              "fixed top-0 z-50 flex h-full w-full max-w-md flex-col bg-[var(--color-ivory)] shadow-2xl",
              side === "left" ? "left-0 border-r" : "right-0 border-l",
              "border-[var(--color-line)]",
              className
            )}
            role="dialog"
            aria-modal="true"
            aria-labelledby="sheet-title"
          >
            <div className="flex items-center justify-between border-b border-[var(--color-line)] p-4">
              <span
                id="sheet-title"
                className="text-xs font-semibold tracking-[0.18em] text-[var(--color-ink)]"
              >
                {title}
              </span>
              <Button
                variant="outline"
                size="icon"
                onClick={onClose}
                aria-label="Close"
              >
                <X className="h-5 w-5" />
              </Button>
            </div>
            <div className="flex-1 overflow-auto p-5">{children}</div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
};

export { Sheet };
