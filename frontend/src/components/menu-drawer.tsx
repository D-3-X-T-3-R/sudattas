"use client";

import { ChevronRight } from "lucide-react";
import { Sheet } from "@/components/ui/sheet";
import { COLLECTIONS } from "@/lib/constants";
import { goTo } from "@/hooks/use-scroll-to";

export interface MenuDrawerProps {
  open: boolean;
  onClose: () => void;
  setCollection: (c: string) => void;
  reduceMotion?: boolean;
}

export function MenuDrawer({
  open,
  onClose,
  setCollection,
  reduceMotion = false,
}: MenuDrawerProps) {
  const nav = [
    { label: "New arrivals", goToId: "shop" as const },
    { label: "Collections", goToId: "collections" as const },
    { label: "Occasion", goToId: "shop" as const },
    { label: "Best sellers", goToId: "shop" as const },
  ];

  return (
    <Sheet open={open} onClose={onClose} title="MENU" side="left">
      <div className="space-y-8">
        <div className="space-y-3">
          {nav.map((x) => (
            <button
              key={x.label}
              type="button"
              onClick={() => {
                goTo(x.goToId, reduceMotion);
                onClose();
              }}
              className="flex w-full items-center justify-between border-b border-[var(--color-line)] pb-3 text-left"
            >
              <span className="text-sm font-semibold text-[var(--color-ink)]">
                {x.label}
              </span>
              <ChevronRight className="h-4 w-4 text-[var(--color-muted)]" />
            </button>
          ))}
        </div>

        <div>
          <div className="text-[11px] font-semibold tracking-[0.24em] text-[var(--color-muted)]">
            COLLECTIONS
          </div>
          <div className="mt-4 grid grid-cols-2 gap-2">
            {COLLECTIONS.map((c) => (
              <button
                key={c.key}
                type="button"
                onClick={() => {
                  setCollection(c.key);
                  goTo("shop", reduceMotion);
                  onClose();
                }}
                className="rounded-full border border-[var(--color-line)] bg-white px-4 py-2 text-sm hover:bg-white/80"
              >
                {c.key}
              </button>
            ))}
            <button
              type="button"
              onClick={() => {
                setCollection("All");
                goTo("shop", reduceMotion);
                onClose();
              }}
              className="rounded-full border border-[var(--color-line)] bg-white px-4 py-2 text-sm hover:bg-white/80"
            >
              All
            </button>
          </div>
        </div>

        <div className="rounded-2xl border border-[var(--color-line)] bg-white p-4">
          <div className="text-[11px] font-semibold tracking-[0.24em] text-[var(--color-muted)]">
            NOTE
          </div>
          <div className="mt-2 text-sm text-[var(--color-muted)]">
            This is a visual mock. For production, replace placeholders with
            real product media and connect to your Rust + MySQL backend.
          </div>
        </div>
      </div>
    </Sheet>
  );
}
