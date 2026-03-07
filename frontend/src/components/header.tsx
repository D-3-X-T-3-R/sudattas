"use client";

import { useState, useEffect } from "react";
import { Search, Menu, Heart, ShoppingBag } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

const NAV_LINKS = [
  { id: "top", label: "Home" },
  { id: "collections", label: "Collections" },
  { id: "shop", label: "Shop" },
  { id: "story", label: "Story" },
] as const;

export interface HeaderProps {
  query: string;
  setQuery: (q: string) => void;
  cartCount: number;
  wishCount: number;
  setMenuOpen: (open: boolean) => void;
  setCartOpen: (open: boolean) => void;
  setWishOpen: (open: boolean) => void;
  goTo: (id: string, instant?: boolean) => void;
  authEnabled?: boolean;
  authButtons?: React.ReactNode;
}

export function Header({
  query,
  setQuery,
  cartCount,
  wishCount,
  setMenuOpen,
  setCartOpen,
  setWishOpen,
  goTo,
  authEnabled,
  authButtons,
}: HeaderProps) {
  const [scrolled, setScrolled] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 24);
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header
      className={cn(
        "sticky top-0 z-30 transition-colors duration-300",
        scrolled
          ? "border-b border-[var(--color-line)] bg-[var(--color-warm-white)]/95 backdrop-blur-md"
          : "border-b border-transparent bg-transparent"
      )}
    >
      <div className="mx-auto grid max-w-7xl grid-cols-3 items-center gap-4 px-4 py-4">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setMenuOpen(true)}
            aria-label="Open menu"
            className="md:hidden"
          >
            <Menu className="h-5 w-5" />
          </Button>
          <nav className="hidden md:flex md:items-center md:gap-6">
            {NAV_LINKS.map(({ id, label }) => (
              <button
                key={id}
                type="button"
                onClick={() => goTo(id, false)}
                className="text-xs font-medium uppercase tracking-[0.18em] text-[var(--color-ink)] transition-colors hover:text-[var(--color-accent-brown)]"
              >
                {label}
              </button>
            ))}
          </nav>
        </div>

        <button
          type="button"
          onClick={() => goTo("top", false)}
          className="justify-self-center font-display text-xl font-medium tracking-[0.12em] text-[var(--color-ink)] md:text-2xl"
          aria-label="Go to top"
        >
          Sudatta&apos;s
        </button>

        <div className="flex items-center justify-end gap-1">
          {authEnabled && authButtons && (
            <div className="hidden items-center sm:flex">{authButtons}</div>
          )}
          {searchOpen ? (
            <div className="absolute right-4 top-full mt-2 w-[280px] md:relative md:right-0 md:mt-0 md:block md:w-[240px]">
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search"
                className="pl-10 py-2.5"
                autoFocus
                onBlur={() => setSearchOpen(false)}
              />
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--color-muted)]" />
            </div>
          ) : (
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setSearchOpen(true)}
              aria-label="Search"
              className="md:flex"
            >
              <Search className="h-5 w-5" />
            </Button>
          )}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setWishOpen(true)}
            aria-label="Wishlist"
            className="relative"
          >
            <Heart className="h-5 w-5" />
            {wishCount > 0 && (
              <span className="absolute -right-0.5 -top-0.5 flex h-4 w-4 items-center justify-center rounded-full bg-[var(--color-ink)] text-[10px] font-semibold text-white">
                {wishCount}
              </span>
            )}
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setCartOpen(true)}
            aria-label="Bag"
            className="relative"
          >
            <ShoppingBag className="h-5 w-5" />
            {cartCount > 0 && (
              <span className="absolute -right-0.5 -top-0.5 flex h-4 w-4 items-center justify-center rounded-full bg-[var(--color-ink)] text-[10px] font-semibold text-white">
                {cartCount}
              </span>
            )}
          </Button>
        </div>
      </div>

      <div className="border-t border-[var(--color-line)] md:hidden">
        <div className="mx-auto max-w-7xl px-4 py-3">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--color-muted)]" />
            <Input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search sarees, fabric, occasion"
              className="w-full py-3 pl-10"
            />
          </div>
        </div>
      </div>
    </header>
  );
}
