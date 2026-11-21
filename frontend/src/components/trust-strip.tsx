"use client";

import type { LucideIcon } from "lucide-react";
import { ShieldCheck, BadgeCheck, MessageCircle } from "lucide-react";
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
      <ul className="grid grid-cols-3 items-start gap-1 md:flex md:flex-wrap md:justify-center md:gap-x-20 md:gap-y-5">
        {items.map((item) => {
          const Icon = item.icon ?? BadgeCheck;
          return (
            <li key={item.title} className="flex min-w-0 items-start justify-center text-center md:justify-start md:gap-2.5 md:text-left">
              <Icon className="mt-0.5 hidden h-4 w-4 shrink-0 text-[var(--color-gold)] md:block" strokeWidth={1.75} />
              <span className="min-w-0">
                <span className="flex items-center justify-center gap-1 whitespace-nowrap text-[9px] font-semibold uppercase text-[var(--color-ink)] sm:text-[11px] md:block md:tracking-[0.16em]">
                  <Icon aria-hidden="true" className="h-2.5 w-2.5 shrink-0 text-[var(--color-gold)] md:hidden" strokeWidth={1.75} />
                  {item.title}
                </span>
                <span className="mt-1 block whitespace-nowrap text-[8px] leading-snug text-[var(--color-muted)] sm:text-xs">{item.detail}</span>
              </span>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
