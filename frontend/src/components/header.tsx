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
  { id: "top", label: "Home" },
  { id: "collections", label: "Moods" },
  { id: "category-collections", label: "Collections" },
  { id: "shop", label: "New Arrivals" },
  { id: "explore", label: "Explore" },
  { id: "story", label: "Story" },
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
      <div className="mx-auto w-full max-w-[var(--container-max)] px-[var(--gutter-mobile)] py-2.5 md:px-[var(--gutter-tablet)] md:py-3">
        <div className="flex items-center justify-between gap-3 md:gap-4">
          <div className="flex min-w-0 flex-1 items-center gap-2 md:gap-4">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setMenuOpen(true)}
              aria-label="Open menu"
              className="h-9 w-9 rounded-md md:hidden"
            >
              <Menu size={20} />
            </Button>
            <div className="hidden items-center gap-5 text-[10px] font-semibold uppercase tracking-[0.15em] text-[var(--color-muted)] md:flex">
              <Link href="/about" className="hover:text-[var(--color-green)]">
                About Us
              </Link>
              <Link href="/contact-support" className="hover:text-[var(--color-green)]">
                Stores
              </Link>
            </div>
          </div>

          <div className="flex shrink-0 items-center justify-end gap-0.5 md:gap-1">
            {authEnabled && authButtons ? (
              <div className="hidden items-center pr-1 lg:flex">{authButtons}</div>
            ) : null}
            {!searchOpen ? (
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setSearchOpen(true)}
                aria-label="Search"
                className="h-9 w-9 rounded-md"
              >
                <Search size={18} />
              </Button>
            ) : null}
            <Button
              variant="ghost"
              size="icon"
              aria-label="Wishlist"
              className={cn(
                "relative h-9 w-9 rounded-md",
                wishCount > 0 && "text-[var(--color-gold)]"
              )}
              asChild
            >
              <Link href="/wishlist">
                <Heart size={18} />
                {wishCount > 0 && (
                  <span className="absolute -right-1 -top-1 flex h-5 min-w-5 items-center justify-center rounded-sm bg-[var(--color-gold)] px-1 text-[10px] font-semibold text-white">
                    {wishCount}
                  </span>
                )}
              </Link>
            </Button>
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

        {searchOpen ? (
          <div className="mt-2.5 flex items-center gap-2 border-t border-[var(--color-line)] pt-2.5">
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
        ) : null}
      </div>

      <div className="hidden border-t border-[var(--color-line)]/95 md:block">
        <div className="mx-auto w-full max-w-[var(--container-max)] px-[var(--gutter-tablet)] py-2">
          <nav className="flex flex-wrap items-center justify-center gap-x-5 gap-y-1.5 lg:gap-x-6">
            {NAV_LINKS.map(({ id, label }) => (
              <button
                key={id}
                type="button"
                onClick={() => navigate(id)}
                className="group relative px-0.5 py-1 text-[11px] font-semibold uppercase tracking-[0.15em] text-[var(--color-ink)] hover:text-[var(--color-green)]"
              >
                {label}
                <span className="absolute -bottom-1 left-0 h-px w-0 bg-[var(--color-gold)] transition-all group-hover:w-full" />
              </button>
            ))}
          </nav>
        </div>
      </div>
    </header>
  );
}
