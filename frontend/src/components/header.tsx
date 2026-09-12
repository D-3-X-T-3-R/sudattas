"use client";

import { useEffect, useState } from "react";
import Image from "next/image";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { Search, Menu, Heart, ShoppingBag } from "lucide-react";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { setPendingHomeSection } from "@/hooks/use-scroll-to";
import { freeShippingThresholdMinor } from "@/lib/env/public";

const NAV_LINKS = [
  { type: "section", id: "top", label: "Home" },
  { type: "section", id: "category-collections", label: "Collections" },
  { type: "section", id: "shop", label: "New Arrivals" },
  { type: "section", id: "collections", label: "Moods" },
  { type: "section", id: "explore", label: "Explore" },
  { type: "section", id: "journal", label: "Journal" },
  { type: "route", href: "/about", label: "About Us" },
] as const;

const thresholdMinor = freeShippingThresholdMinor();
const thresholdRupees = thresholdMinor / 100;
const thresholdDisplay = `₹${thresholdRupees.toLocaleString("en-IN")}`;
const BANNER_TEXT = `FREE SHIPPING ON ALL ORDERS ABOVE ${thresholdDisplay}`;
const BANNER_SEPARATOR = "                              ★                              ";

export interface HeaderProps {
  query: string;
  setQuery: (q: string) => void;
  cartCount: number;
  wishCount: number;
  setMenuOpen: (open: boolean) => void;
  goTo: (id: string, instant?: boolean) => void;
  searchOpen: boolean;
  setSearchOpen: (open: boolean) => void;
  navUseHashLinks?: boolean;
  authEnabled?: boolean;
  authButtons?: React.ReactNode;
}

function Logo({ className }: { className?: string }) {
  return (
    <Link href="/" aria-label="Sudatta's home" className={className}>
      <Image
        src="/hero/sudattas-logo.png"
        alt="Sudatta's Designer Boutique"
        width={168}
        height={100}
        priority
        unoptimized
        className="h-10 w-auto object-contain lg:h-12"
      />
    </Link>
  );
}

function WishlistLink({ wishCount, desktop = false }: { wishCount: number; desktop?: boolean }) {
  return (
    <Link
      href="/wishlist"
      className={cn(
        "relative inline-flex h-10 w-10 items-center justify-center rounded-md text-[var(--color-ink)] transition-colors hover:text-[var(--color-green)]",
        desktop && "border border-transparent hover:border-[var(--color-line)]",
        wishCount > 0 && "text-[var(--color-gold)]"
      )}
      aria-label="Wishlist"
    >
      <Heart size={desktop ? 20 : 18} />
      {wishCount > 0 && (
        <span className="absolute -right-1 -top-1 flex h-5 min-w-5 items-center justify-center rounded-sm bg-[var(--color-gold)] px-1 text-[10px] font-semibold text-white">
          {wishCount}
        </span>
      )}
    </Link>
  );
}

function AnnouncementBanner() {
  const segment = Array(6).fill(`${BANNER_TEXT}${BANNER_SEPARATOR}`).join("");

  return (
    <div
      className="overflow-hidden bg-[var(--color-green)] py-2"
      style={{
        backgroundImage:
          "repeating-linear-gradient(135deg, rgba(255,255,255,0.025) 0px, rgba(255,255,255,0.025) 1px, transparent 1px, transparent 7px)",
      }}
    >
      <div className="announcement-marquee flex whitespace-nowrap">
        <span className="shrink-0 text-[11px] font-semibold uppercase tracking-[0.22em] text-[var(--color-on-deep)]">
          {segment}
        </span>
        <span className="shrink-0 text-[11px] font-semibold uppercase tracking-[0.22em] text-[var(--color-on-deep)]" aria-hidden>
          {segment}
        </span>
      </div>
    </div>
  );
}

export function Header({
  query,
  setQuery,
  cartCount,
  wishCount,
  setMenuOpen,
  goTo,
  searchOpen,
  setSearchOpen,
  navUseHashLinks = false,
  authEnabled,
  authButtons,
}: HeaderProps) {
  const router = useRouter();
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 14);
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  const navigate = (id: string) => {
    if (navUseHashLinks) {
      setPendingHomeSection(id, { fromOtherPage: true });
      router.push("/");
      return;
    }
    goTo(id, false);
  };

  return (
    <header
      className={cn(
        "sticky top-0 z-40 w-full border-b border-[var(--color-line)] bg-[var(--color-surface)]/96 backdrop-blur",
        scrolled && "shadow-[0_5px_18px_rgba(45,42,38,0.07)]"
      )}
    >
      <AnnouncementBanner />

      {/* Mobile */}
      <div className="pt-[env(safe-area-inset-top)] lg:hidden">
        <div className="relative mx-auto flex h-14 w-full max-w-[var(--container-max)] items-center justify-between px-[var(--gutter-mobile)]">
          <button
            type="button"
            onClick={() => setMenuOpen(true)}
            aria-label="Open menu"
            className="inline-flex h-11 w-11 items-center justify-center rounded-md border border-[var(--color-line-strong)] bg-[var(--color-surface-soft)] text-[var(--color-green)] shadow-[var(--shadow-subtle)] transition-colors hover:border-[var(--color-gold)] hover:text-[var(--color-green-2)]"
          >
            <Menu size={20} strokeWidth={2.2} />
          </button>

          <Logo className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2" />

          <span className="h-11 w-11" aria-hidden />
        </div>
      </div>

      {/* Desktop: single row */}
      <div className="hidden lg:block">
        <div className="relative mx-auto w-full max-w-[var(--container-max)] px-[var(--gutter-tablet)]">
          <div className="flex items-center gap-6 py-1.5">
            <Logo />

            <nav
              className="flex flex-1 min-w-0 items-center justify-center gap-5 xl:gap-7"
              aria-label="Primary"
            >
              {NAV_LINKS.map((link) =>
                link.type === "route" ? (
                  <Link
                    key={link.href}
                    href={link.href}
                    className="group relative whitespace-nowrap px-0.5 py-1 text-[15px] font-semibold uppercase tracking-[0.1em] text-[var(--color-ink)] transition-colors hover:text-[var(--color-green)]"
                  >
                    {link.label}
                    <span className="absolute -bottom-1 left-0 h-px w-0 bg-[var(--color-gold)] transition-all group-hover:w-full" />
                  </Link>
                ) : (
                  <button
                    key={link.id}
                    type="button"
                    onClick={() => navigate(link.id)}
                    className="group relative whitespace-nowrap px-0.5 py-1 text-[15px] font-semibold uppercase tracking-[0.1em] text-[var(--color-ink)] transition-colors hover:text-[var(--color-green)]"
                  >
                    {link.label}
                    <span className="absolute -bottom-1 left-0 h-px w-0 bg-[var(--color-gold)] transition-all group-hover:w-full" />
                  </button>
                )
              )}
            </nav>

            <div className="flex shrink-0 items-center gap-2">
              {authEnabled && authButtons ? (
                <div className="flex items-center">{authButtons}</div>
              ) : null}
              <button
                type="button"
                onClick={() => setSearchOpen(!searchOpen)}
                className="inline-flex h-10 w-10 items-center justify-center rounded-md border border-transparent text-[var(--color-ink)] transition-colors hover:border-[var(--color-line)] hover:text-[var(--color-green)]"
                aria-label="Search"
              >
                <Search size={20} />
              </button>
              <WishlistLink wishCount={wishCount} desktop />
              <Link
                href="/bag"
                className={cn(
                  "relative inline-flex h-10 w-10 items-center justify-center rounded-md border border-transparent text-[var(--color-ink)] transition-colors hover:border-[var(--color-line)] hover:text-[var(--color-green)]",
                  cartCount > 0 && "text-[var(--color-gold)]"
                )}
                aria-label="Bag"
              >
                <ShoppingBag size={20} />
                {cartCount > 0 && (
                  <span className="absolute -right-1 -top-1 flex h-5 min-w-5 items-center justify-center rounded-sm bg-[var(--color-gold)] px-1 text-[10px] font-semibold text-white">
                    {cartCount}
                  </span>
                )}
              </Link>
            </div>
          </div>

          {searchOpen ? (
            <div className="absolute left-auto right-[var(--gutter-tablet)] top-full z-50 mt-2 w-[420px] rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-2 shadow-[var(--shadow-soft)]">
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search collections, fabrics, styles"
                className="h-9"
                autoFocus
              />
            </div>
          ) : null}
        </div>
      </div>

      {searchOpen ? (
        <div className="fixed left-[var(--gutter-mobile)] right-[var(--gutter-mobile)] top-[calc(env(safe-area-inset-top)+4.55rem)] z-50 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-2 shadow-[var(--shadow-soft)] lg:hidden">
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search collections, fabrics, styles"
            className="h-9"
            autoFocus
          />
        </div>
      ) : null}
    </header>
  );
}
