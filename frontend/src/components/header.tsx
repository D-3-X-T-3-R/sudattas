"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
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
          ? "border-b border-[var(--color-line)] bg-[var(--color-deep-green-elevated)]/95 backdrop-blur-md"
          : "border-b border-transparent bg-[var(--color-deep-green)]/80"
      )}
    >
      <div className="mx-auto grid max-w-[2000px] grid-cols-3 items-center gap-4 px-4 py-4">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setMenuOpen(true)}
            aria-label="Open menu"
            className="md:hidden"
          >
            <Menu className="h-6 w-6 shrink-0" strokeWidth={2.5} />
          </Button>
          <nav className="hidden md:flex md:items-center md:gap-6">
            {NAV_LINKS.map(({ id, label }) => (
              <button
                key={id}
                type="button"
                onClick={() => goTo(id, false)}
                className="text-sm font-medium uppercase tracking-[0.18em] text-[var(--color-ink)] transition-colors hover:text-[var(--color-accent-gold)]"
              >
                {label}
              </button>
            ))}
          </nav>
        </div>

        {/* Center column left empty (logo removed) to keep nav and actions aligned */}
        <div />

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
              <Search className="absolute left-3 top-1/2 h-6 w-6 -translate-y-1/2 text-[var(--color-muted)]" strokeWidth={2.5} />
            </div>
          ) : (
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setSearchOpen(true)}
              aria-label="Search"
              className="md:flex"
            >
              <Search className="h-6 w-6 shrink-0" strokeWidth={2.5} />
            </Button>
          )}
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setWishOpen(true)}
            aria-label="Wishlist"
            className={cn("relative", wishCount > 0 && "text-[var(--color-accent-gold)]")}
          >
            <Heart className="h-6 w-6 shrink-0" strokeWidth={2.5} />
            {wishCount > 0 && (
              <span className="absolute -right-0.5 -top-0.5 flex h-5 w-5 items-center justify-center rounded-full bg-[var(--color-accent-gold)] font-sans text-xs font-semibold text-white">
                {wishCount}
              </span>
            )}
          </Button>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Bag"
            className={cn("relative", cartCount > 0 && "text-[var(--color-accent-gold)]")}
            asChild
          >
            <Link href="/bag">
              <ShoppingBag className="h-6 w-6 shrink-0" strokeWidth={2.5} />
              {cartCount > 0 && (
                <span className="absolute -right-0.5 -top-0.5 flex h-5 w-5 items-center justify-center rounded-full bg-[var(--color-accent-gold)] font-sans text-xs font-semibold text-white">
                  {cartCount}
                </span>
              )}
            </Link>
          </Button>
        </div>
      </div>
    </header>
  );
}
