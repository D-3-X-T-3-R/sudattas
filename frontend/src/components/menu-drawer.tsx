"use client";

import Link from "next/link";
import { usePathname, useRouter } from "next/navigation";
import { ChevronRight } from "lucide-react";
import { Sheet } from "@/components/ui/sheet";
import { goTo, setPendingHomeSection } from "@/hooks/use-scroll-to";

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
    { type: "section", label: "Collections", goToId: "category-collections" },
    { type: "section", label: "New Arrivals", goToId: "shop" },
    { type: "section", label: "Moods", goToId: "collections" },
    { type: "section", label: "Explore", goToId: "explore" },
    { type: "route", label: "About Us", href: "/about" },
  ];

  return (
    <Sheet
      open={open}
      onClose={onClose}
      title="Menu"
      side="left"
      className="w-[86vw] max-w-[21rem] border-[var(--color-line)] bg-[var(--color-surface)] shadow-[18px_0_44px_rgba(45,42,38,0.12)]"
      overlayClassName="z-[45] bg-[var(--color-green)]/18 backdrop-blur-[1px]"
      headerClassName="px-5 pb-4 pt-[calc(0.95rem+env(safe-area-inset-top))]"
      bodyClassName="px-5 pb-8 pt-5"
      closeButtonClassName="h-10 w-10 rounded-full border-[var(--color-line)] bg-[var(--color-surface-soft)] text-[var(--color-green)] hover:border-[var(--color-gold)] hover:bg-[var(--color-surface)]"
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
