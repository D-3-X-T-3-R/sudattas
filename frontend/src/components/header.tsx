"use client";

import { Search, ChevronRight, Menu, Heart, ShoppingBag } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

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
  return (
    <header
      className="sticky top-0 z-30 border-b border-[var(--color-line)] backdrop-blur-md"
      style={{ background: "rgba(250,248,245,0.92)" }}
    >
      <div className="mx-auto grid max-w-7xl grid-cols-3 items-center px-4 py-4">
        <div className="flex items-center gap-3">
          <Button
            variant="outline"
            size="icon"
            onClick={() => setMenuOpen(true)}
            aria-label="Open menu"
            className="border-[var(--color-line)] bg-[var(--color-ivory)] hover:bg-white"
          >
            <Menu className="h-5 w-5" />
          </Button>
          <button
            type="button"
            onClick={() => goTo("shop", false)}
            className="hidden md:inline-flex items-center gap-2 text-xs font-semibold tracking-[0.2em] text-[var(--color-ink)] hover:text-[var(--color-accent-brown)] transition-colors"
          >
            SHOP
            <ChevronRight className="h-4 w-4" />
          </button>
        </div>

        <button
          type="button"
          onClick={() => goTo("top", false)}
          className="mx-auto flex flex-col items-center justify-center font-display text-2xl font-medium tracking-[0.12em] text-[var(--color-ink)]"
          aria-label="Go to top"
        >
          <span>Sudatta&apos;s</span>
          <span className="mt-0.5 h-px w-8 bg-[var(--color-accent-gold)]" />
        </button>

        <div className="ml-auto flex items-center justify-end gap-2">
          {authEnabled && authButtons && (
            <div className="hidden items-center sm:flex">{authButtons}</div>
          )}
          <div className="hidden md:flex items-center">
            <div className="relative w-[320px]">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--color-muted)]" />
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search sarees, fabric, occasion"
                className="pl-10 w-full"
              />
            </div>
          </div>

          <Button
            variant="outline"
            size="icon"
            onClick={() => setWishOpen(true)}
            aria-label="Wishlist"
            className="relative border-[var(--color-line)] bg-[var(--color-ivory)] hover:bg-white"
          >
            <Heart className="h-5 w-5" />
            {wishCount > 0 && (
              <span className="absolute -right-1 -top-1 flex h-6 w-6 items-center justify-center rounded-full text-xs font-semibold text-white bg-[var(--color-ink)]">
                {wishCount}
              </span>
            )}
          </Button>

          <Button
            variant="outline"
            size="icon"
            onClick={() => setCartOpen(true)}
            aria-label="Bag"
            className="relative border-[var(--color-line)] bg-[var(--color-ivory)] hover:bg-white"
          >
            <ShoppingBag className="h-5 w-5" />
            {cartCount > 0 && (
              <span className="absolute -right-1 -top-1 flex h-6 w-6 items-center justify-center rounded-full text-xs font-semibold text-white bg-[var(--color-ink)]">
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
              className="pl-10 w-full py-3"
            />
          </div>
        </div>
      </div>
    </header>
  );
}
