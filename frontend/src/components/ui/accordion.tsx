"use client";

import { ChevronUp } from "lucide-react";

export function Accordion({
  title,
  children,
  defaultOpen = false,
}: {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
}) {
  return (
    <details className="group border-b border-[var(--color-line)]" open={defaultOpen}>
      <summary className="flex cursor-pointer list-none items-center justify-between py-4 text-sm font-semibold text-[var(--color-ink)]">
        {title}
        <ChevronUp className="h-4 w-4 shrink-0 transition-transform group-open:rotate-180" />
      </summary>
      <div className="pb-4 text-sm text-[var(--color-muted)]">{children}</div>
    </details>
  );
}
