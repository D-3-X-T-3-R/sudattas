"use client";

import type { LucideIcon } from "lucide-react";
import { ShieldCheck, RotateCcw, BadgeCheck, MessageCircle } from "lucide-react";
import { cn } from "@/lib/utils";

export type TrustItem = {
  title: string;
  detail: string;
  icon?: LucideIcon;
};

const DEFAULT_ITEMS: TrustItem[] = [
  {
    title: "Premium Fabrics",
    detail: "Carefully sourced quality",
    icon: BadgeCheck,
  },
  {
    title: "Secure Checkout",
    detail: "Protected payment journey",
    icon: ShieldCheck,
  },
  {
    title: "Easy Returns",
    detail: "Hassle-free support",
    icon: RotateCcw,
  },
  {
    title: "Customer Care",
    detail: "Fast human assistance",
    icon: MessageCircle,
  },
];

export function TrustStrip({
  items = DEFAULT_ITEMS,
  className,
}: {
  items?: TrustItem[];
  className?: string;
}) {
  return (
    <section className={cn("rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-4 shadow-[var(--shadow-subtle)]", className)}>
      <ul className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        {items.map((item) => {
          const Icon = item.icon ?? BadgeCheck;
          return (
            <li key={item.title} className="flex items-start gap-3 rounded-md border border-[var(--color-line)]/70 bg-[var(--color-surface-soft)] px-3 py-3">
              <span className="mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-sm border border-[var(--color-line)] bg-white text-[var(--color-green)]">
                <Icon className="h-4 w-4" />
              </span>
              <span>
                <span className="block text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)]">{item.title}</span>
                <span className="mt-1 block text-xs text-[var(--color-muted)]">{item.detail}</span>
              </span>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
