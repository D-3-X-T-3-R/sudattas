"use client";

import type { ReactNode } from "react";
import { ChevronDown } from "lucide-react";

/** Native disclosure — used to tuck secondary/technical detail behind an explicit "show more" action. */
export function CollapsibleSection({
  title,
  description,
  children,
  defaultOpen = false,
}: {
  title: string;
  description?: string;
  children: ReactNode;
  defaultOpen?: boolean;
}) {
  return (
    <details
      className="group rounded-xl border border-[var(--color-line)] bg-white/60 open:bg-white"
      open={defaultOpen}
    >
      <summary className="flex cursor-pointer list-none items-center justify-between gap-3 px-4 py-3.5 md:px-5">
        <span>
          <span className="text-sm font-semibold text-[var(--color-ink)]">{title}</span>
          {description ? (
            <span className="mt-0.5 block text-xs text-[var(--color-muted)]">{description}</span>
          ) : null}
        </span>
        <ChevronDown className="h-4 w-4 shrink-0 text-[var(--color-muted)] transition-transform group-open:rotate-180" />
      </summary>
      <div className="px-4 pb-4 md:px-5 md:pb-5">{children}</div>
    </details>
  );
}
