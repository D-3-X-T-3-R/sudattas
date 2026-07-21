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
    <section className={cn("border-t border-[var(--color-line)] py-5", className)}>
      <ul className="flex flex-wrap items-start justify-center gap-x-12 gap-y-5 sm:gap-x-16 md:gap-x-20">
        {items.map((item) => {
          const Icon = item.icon ?? BadgeCheck;
          return (
            <li key={item.title} className="flex items-start gap-2.5">
              <Icon className="mt-0.5 h-4 w-4 shrink-0 text-[var(--color-gold)]" strokeWidth={1.75} />
              <span className="min-w-0">
                <span className="block text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)]">{item.title}</span>
                <span className="mt-1 block text-xs leading-snug text-[var(--color-muted)]">{item.detail}</span>
              </span>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
