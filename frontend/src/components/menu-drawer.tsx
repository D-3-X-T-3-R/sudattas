"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { ChevronRight } from "lucide-react";
import { Sheet } from "@/components/ui/sheet";
import { COLLECTIONS } from "@/lib/constants";
import { goTo, setPendingHomeCollection, setPendingHomeSection } from "@/hooks/use-scroll-to";

export interface MenuDrawerProps {
  open: boolean;
  onClose: () => void;
  setCollection: (c: string) => void;
  reduceMotion?: boolean;
}

type DrawerNavItem =
  | {
      type: "section";
      label: string;
      goToId: "top" | "collections" | "category-collections" | "shop" | "explore";
    }
  | { type: "route"; label: string; href: "/about" };

export function MenuDrawer({
  open,
  onClose,
  setCollection,
  reduceMotion = false,
}: MenuDrawerProps) {
  const pathname = usePathname();
  const router = useRouter();
  const isHome = pathname === "/";

  const navigateToSection = (id: string) => {
    if (!isHome) {
      setPendingHomeSection(id, { fromOtherPage: true });
      router.push("/");
    } else {
      goTo(id, reduceMotion);
    }
  };

  const nav: DrawerNavItem[] = [
    { type: "section", label: "Home", goToId: "top" },
    { type: "section", label: "Moods", goToId: "collections" },
    { type: "section", label: "Collections", goToId: "category-collections" },
    { type: "section", label: "New arrivals", goToId: "shop" },
    { type: "section", label: "Explore", goToId: "explore" },
    { type: "route", label: "About Us", href: "/about" },
  ];

  return (
    <Sheet open={open} onClose={onClose} title="Menu" side="left">
      <div className="space-y-8">
        <div className="space-y-3">
          {nav.map((x) =>
            x.type === "route" ? (
              <Link
                key={x.label}
                href={x.href}
                onClick={onClose}
                className="flex min-h-11 w-full items-center justify-between border-b border-[var(--color-line)] py-3 text-left"
              >
                <span className="text-sm font-semibold uppercase tracking-[0.14em] text-[var(--color-ink)]">
                  {x.label}
                </span>
                <ChevronRight className="h-4 w-4 text-[var(--color-muted)]" />
              </Link>
            ) : (
              <button
                key={x.label}
                type="button"
                onClick={() => {
                  navigateToSection(x.goToId);
                  onClose();
                }}
                className="flex min-h-11 w-full items-center justify-between border-b border-[var(--color-line)] py-3 text-left"
              >
                <span className="text-sm font-semibold uppercase tracking-[0.14em] text-[var(--color-ink)]">
                  {x.label}
                </span>
                <ChevronRight className="h-4 w-4 text-[var(--color-muted)]" />
              </button>
            )
          )}
        </div>

        <div>
          <div className="text-[11px] font-semibold uppercase tracking-[0.24em] text-[var(--color-muted)]">
            COLLECTIONS
          </div>
          <div className="mt-4 grid grid-cols-2 gap-2">
            {COLLECTIONS.map((c) => (
              <button
                key={c.key}
                type="button"
                onClick={() => {
                  setPendingHomeCollection(c.key);
                  setCollection(c.key);
                  navigateToSection("shop");
                  onClose();
                }}
                className="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-xs font-semibold uppercase tracking-[0.14em] hover:border-[var(--color-gold)] hover:text-[var(--color-green)]"
              >
                {c.key}
              </button>
            ))}
            <button
              type="button"
              onClick={() => {
                setPendingHomeCollection("All");
                setCollection("All");
                navigateToSection("shop");
                onClose();
              }}
              className="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-2 text-xs font-semibold uppercase tracking-[0.14em] hover:border-[var(--color-gold)] hover:text-[var(--color-green)]"
            >
              All
            </button>
          </div>
        </div>

        <div className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-4">
          <div className="text-[11px] font-semibold uppercase tracking-[0.24em] text-[var(--color-muted)]">
            CURATED
          </div>
          <div className="mt-2 text-sm text-[var(--color-muted)]">
            Explore collections and moods to discover the drape that fits your
            moment.
          </div>
        </div>
      </div>
    </Sheet>
  );
}
