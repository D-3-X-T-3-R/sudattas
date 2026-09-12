"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { cn } from "@/lib/utils";
import type { AdminGroupTab } from "@/lib/admin-nav-groups";

/**
 * Underline tab strip for switching between sibling pages within one grouped nav section
 * (e.g. Orders/Shipments/Returns). Deliberately styled differently from in-page filter
 * pills (see reviews/returns status tabs) so the two aren't mistaken for each other —
 * this one navigates to a different route, those just re-filter the current one.
 */
export function AdminGroupTabs({ tabs }: { tabs: AdminGroupTab[] }) {
  const pathname = usePathname() ?? "";

  return (
    <div className="-mt-2 flex gap-1 border-b border-[var(--color-line)]" role="tablist">
      {tabs.map((tab) => {
        const isActive = pathname === tab.href || pathname.startsWith(`${tab.href}/`);
        return (
          <Link
            key={tab.href}
            href={tab.href}
            role="tab"
            aria-selected={isActive}
            className={cn(
              "-mb-px border-b-2 px-4 py-2.5 text-sm font-semibold transition-colors",
              isActive
                ? "border-[var(--color-green)] text-[var(--color-ink)]"
                : "border-transparent text-[var(--color-muted)] hover:text-[var(--color-ink)]"
            )}
          >
            {tab.label}
          </Link>
        );
      })}
    </div>
  );
}
