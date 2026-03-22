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
  setCartOpen: (open: boolean) => void;
  goTo: (id: string, instant?: boolean) => void;
  /**
   * When true (e.g. bag / wishlist / product), nav uses the same `<button>` + underline as the
   * landing page but navigates with `router.push('/#id')`. `goTo` is unused in that mode.
   */
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
  setCartOpen,
  goTo,
  navUseHashLinks = false,
  authEnabled,
  authButtons,
}: HeaderProps) {
  const router = useRouter();
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
        "sticky top-0 z-30 w-full min-w-0 transition-all duration-500",
        scrolled
          ? "border-b border-[var(--color-line)]/60 bg-[var(--color-ivory)]/80 backdrop-blur-md shadow-[0_1px_24px_rgba(26,24,20,0.06)]"
          : "border-b border-transparent bg-[var(--color-ivory)]/40 backdrop-blur-sm"
      )}
    >
      <div className="mx-auto grid w-full max-w-[2000px] grid-cols-[1fr_0_auto] items-center gap-4 px-4 py-4 md:grid-cols-[1fr_0_auto]">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setMenuOpen(true)}
            aria-label="Open menu"
            className="md:hidden"
          >
            <Menu size={28} strokeWidth={2.5} />
          </Button>
          <nav className="hidden md:flex md:items-center md:gap-6">
            {NAV_LINKS.map(({ id, label }) => {
              const navItemClass =
                "group relative inline-flex cursor-pointer appearance-none border-0 bg-transparent p-0 text-left font-sans text-base font-medium uppercase tracking-[0.18em] text-[var(--color-ink)] no-underline transition-colors duration-300 ease-out hover:text-[var(--color-accent-gold)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent-gold)]/25 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--color-ivory)]";
              const underline = (
                <span
                  aria-hidden
                  className="pointer-events-none absolute -bottom-0.5 left-0 h-px w-0 bg-[var(--color-accent-gold)] transition-all duration-300 ease-out group-hover:w-full"
                />
              );
              if (navUseHashLinks) {
                return (
                  <button
                    key={id}
                    type="button"
                    onClick={() => {
                      setPendingHomeSection(id, { fromOtherPage: true });
                      router.push("/");
                    }}
                    className={navItemClass}
                  >
                    {label}
                    {underline}
                  </button>
                );
              }
              return (
                <button
                  key={id}
                  type="button"
                  onClick={() => goTo(id, false)}
                  className={navItemClass}
                >
                  {label}
                  {underline}
                </button>
              );
            })}
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
              <Search size={28} className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-muted)]" strokeWidth={2.5} />
            </div>
          ) : (
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setSearchOpen(true)}
              aria-label="Search"
              className="md:flex"
            >
              <Search size={28} strokeWidth={2.5} />
            </Button>
          )}
          <Button
            variant="ghost"
            size="icon"
            aria-label="Wishlist"
            className={cn("relative", wishCount > 0 && "text-[var(--color-accent-gold)]")}
            asChild
          >
            <Link href="/wishlist">
              <Heart size={28} strokeWidth={2.5} />
              {wishCount > 0 && (
                <span className="absolute -right-0.5 -top-0.5 flex h-5 w-5 items-center justify-center rounded-full bg-[var(--color-accent-gold)] font-sans text-xs font-semibold text-white">
                  {wishCount}
                </span>
              )}
            </Link>
          </Button>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Bag"
            className={cn("relative", cartCount > 0 && "text-[var(--color-accent-gold)]")}
            asChild
          >
            <Link href="/bag">
              <ShoppingBag size={28} strokeWidth={2.5} />
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
