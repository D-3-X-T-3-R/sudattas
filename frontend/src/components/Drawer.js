import React from "react";
import { AnimatePresence, motion } from "framer-motion";
import { X } from "lucide-react";

function classNames(...xs) {
  return xs.filter(Boolean).join(" ");
}

export default function Drawer({
  open,
  title,
  children,
  onClose,
  side = "left",
  theme,
}) {
  const fromX = side === "left" ? -420 : 420;
  const palette = theme || { ivory: "#F7F5F0", line: "#E7E1D6" };

  return (
    <AnimatePresence>
      {open ? (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
            className="fixed inset-0 z-40 bg-black/40"
          />
          <motion.div
            initial={{ x: fromX }}
            animate={{ x: 0 }}
            exit={{ x: fromX }}
            transition={{ type: "spring", stiffness: 320, damping: 30 }}
            className={classNames(
              "fixed top-0 z-50 h-full w-full max-w-md bg-[var(--ivory)] shadow-2xl",
              side === "left" ? "left-0 border-r" : "right-0 border-l",
              "border-[var(--line)]"
            )}
            style={{ "--ivory": palette.ivory, "--line": palette.line }}
          >
            <div className="flex items-center justify-between border-b border-[var(--line)] p-4">
              <div className="text-xs font-semibold tracking-[0.18em] text-[#111]">
                {title}
              </div>
              <button
                onClick={onClose}
                className="grid h-10 w-10 place-items-center rounded-full border border-[var(--line)] bg-[var(--ivory)] text-[#111] hover:bg-white"
                aria-label="Close"
              >
                <X className="h-5 w-5" />
              </button>
            </div>
            <div className="h-[calc(100%-64px)] overflow-auto p-5">{children}</div>
          </motion.div>
        </>
      ) : null}
    </AnimatePresence>
  );
}

