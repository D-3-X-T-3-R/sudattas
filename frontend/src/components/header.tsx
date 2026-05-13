"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { Search, Menu, Heart, ShoppingBag } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { setPendingHomeSection } from "@/hooks/use-scroll-to";

const NAV_LINKS = [
  { type: "section", id: "top", label: "Home" },
  { type: "section", id: "collections", label: "Moods" },
  { type: "section", id: "category-collections", label: "Collections" },
  { type: "section", id: "shop", label: "New Arrivals" },
  { type: "section", id: "explore", label: "Explore" },
  { type: "route", href: "/about", label: "About Us" },
] as const;

export interface HeaderProps {
  query: string;
  setQuery: (q: string) => void;
  cartCount: number;
  wishCount: number;
  setMenuOpen: (open: boolean) => void;
  goTo: (id: string, instant?: boolean) => void;
  navUseHashLinks?: boolean;
  authEnabled?: boolean;
  authButtons?: React.ReactNode;
}

function WishlistLink({ wishCount, desktop = false }: { wishCount: number; desktop?: boolean }) {
  return (
    <Link
      href="/wishlist"
      className={cn(
        "relative inline-flex h-9 w-9 items-center justify-center rounded-md text-[var(--color-ink)] transition-colors hover:text-[var(--color-green)]",
        desktop && "border border-transparent hover:border-[var(--color-line)]",
        wishCount > 0 && "text-[var(--color-gold)]"
      )}
      aria-label="Wishlist"
    >
      <Heart size={desktop ? 15 : 18} />
      {wishCount > 0 && (
        <span className="absolute -right-1 -top-1 flex h-5 min-w-5 items-center justify-center rounded-sm bg-[var(--color-gold)] px-1 text-[10px] font-semibold text-white">
          {wishCount}
        </span>
      )}
    </Link>
  );
}

export function Header({
  query,
  setQuery,
  cartCount,
  wishCount,
  setMenuOpen,
  goTo,
  navUseHashLinks = false,
  authEnabled,
  authButtons,
}: HeaderProps) {
  const router = useRouter();
  const [scrolled, setScrolled] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);

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
      <div className="relative mx-auto w-full max-w-[var(--container-max)] px-[var(--gutter-mobile)] py-2.5 md:px-[var(--gutter-tablet)] lg:py-3">
        <div className="flex items-center justify-between gap-3 lg:hidden">
          <div className="flex min-w-0 flex-1 items-center">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setMenuOpen(true)}
              aria-label="Open menu"
              className="h-9 w-9 rounded-md"
            >
              <Menu size={20} />
            </Button>
          </div>

          <div className="flex shrink-0 items-center justify-end gap-0.5">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setSearchOpen(true)}
              aria-label="Search"
              className="h-9 w-9 rounded-md"
            >
              <Search size={18} />
            </Button>
            <WishlistLink wishCount={wishCount} />
            <Button
              variant="ghost"
              size="icon"
              aria-label="Bag"
              className={cn(
                "relative h-9 w-9 rounded-md",
                cartCount > 0 && "text-[var(--color-gold)]"
              )}
              asChild
            >
              <Link href="/bag">
                <ShoppingBag size={18} />
                {cartCount > 0 && (
                  <span className="absolute -right-1 -top-1 flex h-5 min-w-5 items-center justify-center rounded-sm bg-[var(--color-gold)] px-1 text-[10px] font-semibold text-white">
                    {cartCount}
                  </span>
                )}
              </Link>
            </Button>
          </div>
        </div>

        <div className="hidden items-center justify-between gap-6 lg:flex">
          <nav
            className="flex min-w-0 items-center gap-4 xl:gap-5"
            aria-label="Primary"
          >
            {NAV_LINKS.map((link) =>
              link.type === "route" ? (
                <Link
                  key={link.href}
                  href={link.href}
                  className="group relative whitespace-nowrap px-0.5 py-1 text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--color-ink)] transition-colors hover:text-[var(--color-green)]"
                >
                  {link.label}
                  <span className="absolute -bottom-1 left-0 h-px w-0 bg-[var(--color-gold)] transition-all group-hover:w-full" />
                </Link>
              ) : (
                <button
                  key={link.id}
                  type="button"
                  onClick={() => navigate(link.id)}
                  className="group relative whitespace-nowrap px-0.5 py-1 text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--color-ink)] transition-colors hover:text-[var(--color-green)]"
                >
                  {link.label}
                  <span className="absolute -bottom-1 left-0 h-px w-0 bg-[var(--color-gold)] transition-all group-hover:w-full" />
                </button>
              )
            )}
          </nav>

          <div className="flex shrink-0 items-center justify-end gap-2 text-[11px] font-semibold uppercase tracking-[0.14em]">
            {authEnabled && authButtons ? <div className="flex items-center">{authButtons}</div> : null}
            <button
              type="button"
              onClick={() => setSearchOpen(true)}
              className="inline-flex h-9 w-9 items-center justify-center rounded-md border border-transparent text-[var(--color-ink)] transition-colors hover:border-[var(--color-line)] hover:text-[var(--color-green)]"
              aria-label="Search"
            >
              <Search size={15} />
            </button>
            <WishlistLink wishCount={wishCount} desktop />
            <Link
              href="/bag"
              className={cn(
                "relative inline-flex h-9 w-9 items-center justify-center rounded-md border border-transparent text-[var(--color-ink)] transition-colors hover:border-[var(--color-line)] hover:text-[var(--color-green)]",
                cartCount > 0 && "text-[var(--color-gold)]"
              )}
              aria-label="Bag"
            >
              <ShoppingBag size={15} />
              {cartCount > 0 && (
                <span className="absolute -right-1 -top-1 flex h-5 min-w-5 items-center justify-center rounded-sm bg-[var(--color-gold)] px-1 text-[10px] font-semibold text-white">
                  {cartCount}
                </span>
              )}
            </Link>
          </div>
        </div>

        {searchOpen ? (
          <div className="absolute left-[var(--gutter-mobile)] right-[var(--gutter-mobile)] top-full z-50 mt-2 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-2 shadow-[var(--shadow-soft)] md:left-[var(--gutter-tablet)] md:right-[var(--gutter-tablet)] lg:left-auto lg:w-[420px]">
            <div className="flex items-center gap-2">
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search collections, fabrics, styles"
                className="h-9"
                autoFocus
              />
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setSearchOpen(false)}
              >
                Close
              </Button>
            </div>
          </div>
        ) : null}
      </div>
    </header>
  );
}
