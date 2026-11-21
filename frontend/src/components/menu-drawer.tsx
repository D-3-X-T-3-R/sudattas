"use client";

import Link from "next/link";
import { useId, useState, type ReactNode } from "react";
import { motion } from "framer-motion";
import { usePathname, useRouter } from "next/navigation";
import { ChevronRight } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { Sheet } from "@/components/ui/sheet";
import { goTo, setPendingHomeMood, setPendingHomeSection } from "@/hooks/use-scroll-to";

export interface MenuDrawerProps {
  open: boolean;
  onClose: () => void;
  onExitComplete?: () => void;
  reduceMotion?: boolean;
}

type DrawerNavItem =
  | {
      type: "section";
      label: string;
      goToId: "top" | "collections" | "category-collections" | "shop" | "explore" | "journal";
    }
  | { type: "route"; label: string; href: "/about" };

function MenuSubmenu({ label, reduceMotion, children }: { label: string; reduceMotion: boolean; children: ReactNode }) {
  const [expanded, setExpanded] = useState(false);
  const panelId = useId();
  return (
    <div className="border-b border-[var(--color-line)]">
      <button type="button" aria-expanded={expanded} aria-controls={panelId} onClick={() => setExpanded((value) => !value)} className="flex min-h-[3.45rem] w-full items-center justify-between py-3.5 text-left">
        <span className="font-display text-[1.18rem] leading-tight text-[var(--color-green)]">{label}</span>
        <ChevronRight aria-hidden="true" className="h-4 w-4 text-[var(--color-gold)] transition-transform duration-300 motion-reduce:transition-none" style={{ transform: expanded ? "rotate(90deg)" : "rotate(0deg)", transitionDuration: reduceMotion ? "0s" : undefined }} />
      </button>
      <motion.div id={panelId} initial={false} animate={{ height: expanded ? "auto" : 0, opacity: expanded ? 1 : 0 }} transition={{ duration: reduceMotion ? 0 : 0.26, ease: [0.22, 1, 0.36, 1] }} className="overflow-hidden" inert={!expanded} aria-hidden={!expanded}>
        {children}
      </motion.div>
    </div>
  );
}

export function MenuDrawer({
  open,
  onClose,
  onExitComplete,
  reduceMotion = false,
}: MenuDrawerProps) {
  const pathname = usePathname();
  const router = useRouter();
  const isHome = pathname === "/";
  const filters = useQuery({
    queryKey: ["storefront-menu-filters", "listed-only"],
    enabled: open,
    staleTime: 120_000,
    retry: 1,
    queryFn: async () => {
      const response = await fetch("/api/storefront-filters?listedOnly=1");
      const data = await response.json() as {
        categories: { categoryId: string; name: string }[];
        moods: { moodId: string; moodName: string }[];
        error?: string | null;
      };
      if (!response.ok || data.error) throw new Error(data.error || "Unable to load menu");
      return data;
    },
  });

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
    { type: "section", label: "Categories", goToId: "category-collections" },
    { type: "section", label: "Moods", goToId: "collections" },
    { type: "section", label: "New Arrivals", goToId: "shop" },
    { type: "section", label: "Explore", goToId: "explore" },
    // { type: "section", label: "Journal", goToId: "journal" },
    { type: "route", label: "About Us", href: "/about" },
  ];

  return (
    <Sheet
      open={open}
      onClose={onClose}
      onExitComplete={onExitComplete}
      title="Menu"
      hideHeader
      externalToggleId="mobile-menu-toggle"
      side="left"
      className="w-[86vw] max-w-[21rem] border-[var(--color-line)] bg-[var(--color-surface)] shadow-[18px_0_44px_rgba(45,42,38,0.12)]"
      overlayClassName="z-[45] bg-[var(--color-green)]/18 backdrop-blur-[1px]"
      bodyClassName="px-5 pb-8 pt-[calc(5.75rem+env(safe-area-inset-top))]"
    >
      <div className="flex h-full flex-col">
        <nav className="border-y border-[var(--color-line)]" aria-label="Mobile menu">
          {nav.map((x) =>
            x.type === "route" ? (
              <Link
                key={x.label}
                href={x.href}
                onClick={onClose}
                className="group flex min-h-[3.45rem] w-full items-center justify-between border-b border-[var(--color-line)] py-3.5 text-left last:border-b-0"
              >
                <span className="font-display text-[1.18rem] leading-tight text-[var(--color-green)]">
                  {x.label}
                </span>
                <ChevronRight className="h-4 w-4 text-[var(--color-gold)] opacity-65 transition-transform group-hover:translate-x-0.5 group-hover:opacity-100" />
              </Link>
            ) : x.goToId === "category-collections" || x.goToId === "collections" ? (
              <MenuSubmenu key={x.label} label={x.label} reduceMotion={reduceMotion}>
                <ul className="mb-3 ml-2 border-l border-[var(--color-line)] pl-4 text-sm text-[var(--color-green)]">
                  <li>
                    <button type="button" className="min-h-11 py-2 text-left font-semibold" onClick={() => { navigateToSection(x.goToId); onClose(); }}>
                      View all {x.label.toLowerCase()}
                    </button>
                  </li>
                  {filters.isPending ? <li role="status" className="py-2 text-[var(--color-muted)]">Loading…</li>
                    : filters.isError ? <li><button type="button" className="min-h-11 py-2 text-left" onClick={() => void filters.refetch()}>Could not load items. Retry</button></li>
                    : x.goToId === "category-collections" ? (
                      (filters.data?.categories ?? []).filter((item) => item.name.trim() && !/^(itest|test|e2e|mock|seed)[_-]/i.test(item.name.trim())).map((item) => (
                        <li key={item.categoryId}>
                          <Link className="flex min-h-11 items-center py-2 hover:underline" href={`/collections/${encodeURIComponent(item.name.trim().toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, ""))}`} onClick={onClose}>{item.name}</Link>
                        </li>
                      ))
                    ) : (filters.data?.moods ?? []).map((item) => (
                      <li key={item.moodId}>
                        <button type="button" className="min-h-11 w-full py-2 text-left hover:underline" onClick={() => { setPendingHomeMood(item.moodId); navigateToSection("shop"); onClose(); }}>{item.moodName}</button>
                      </li>
                    ))}
                </ul>
              </MenuSubmenu>
            ) : (
              <button
                key={x.label}
                type="button"
                onClick={() => {
                  navigateToSection(x.goToId);
                  onClose();
                }}
                className="group flex min-h-[3.45rem] w-full items-center justify-between border-b border-[var(--color-line)] py-3.5 text-left last:border-b-0"
              >
                <span className="font-display text-[1.18rem] leading-tight text-[var(--color-green)]">
                  {x.label}
                </span>
                <ChevronRight className="h-4 w-4 text-[var(--color-gold)] opacity-65 transition-transform group-hover:translate-x-0.5 group-hover:opacity-100" />
              </button>
            )
          )}
        </nav>

        <div className="mt-auto border-t border-[var(--color-line)] pt-5">
          <p className="font-display text-base leading-snug text-[var(--color-green)]">Sudatta&apos;s</p>
          <p className="mt-1 text-[10px] font-semibold uppercase tracking-[0.24em] text-[var(--color-muted)]">
            Designer Boutique
          </p>
        </div>
      </div>
    </Sheet>
  );
}
