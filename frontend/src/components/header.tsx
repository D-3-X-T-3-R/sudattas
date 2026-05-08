"use client";

import { useState, useEffect } from "react";
import Image from "next/image";
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
      <div className="mx-auto w-full max-w-[var(--container-max)] px-[var(--gutter-mobile)] py-3 md:px-[var(--gutter-tablet)] md:py-4">
        <div className="grid grid-cols-[auto_1fr_auto] items-center gap-2 md:gap-4">
          <div className="flex items-center gap-4">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setMenuOpen(true)}
              aria-label="Open menu"
              className="md:hidden"
            >
              <Menu size={20} />
            </Button>
            <div className="hidden items-center gap-4 text-[10px] font-semibold uppercase tracking-[0.16em] text-[var(--color-muted)] md:flex">
              <Link href="/about" className="hover:text-[var(--color-green)]">
                About Us
              </Link>
              <Link href="/contact-support" className="hover:text-[var(--color-green)]">
                Stores
              </Link>
            </div>
          </div>

          <Link href="/" className="flex items-center justify-center">
            <Image
              src="/logo.png"
              alt="Sudatta's"
              width={180}
              height={56}
              className="h-11 w-auto sm:h-12 md:h-14"
              priority
            />
          </Link>

          <div className="flex items-center justify-end gap-1.5">
            {authEnabled && authButtons ? (
              <div className="hidden items-center lg:flex">{authButtons}</div>
            ) : null}
            {!searchOpen ? (
              <Button
                variant="ghost"
                size="icon"
                onClick={() => setSearchOpen(true)}
                aria-label="Search"
              >
                <Search size={18} />
              </Button>
            ) : null}
            <Button
              variant="ghost"
              size="icon"
              aria-label="Wishlist"
              className={cn("relative", wishCount > 0 && "text-[var(--color-gold)]")}
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
              className={cn("relative", cartCount > 0 && "text-[var(--color-gold)]")}
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
          <div className="mt-3 flex items-center gap-2 border-t border-[var(--color-line)] pt-3">
            <Input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search collections, fabrics, styles"
              className="h-10"
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

      <div className="hidden border-t border-[var(--color-line)] md:block">
        <div className="mx-auto w-full max-w-[var(--container-max)] px-[var(--gutter-tablet)] py-2.5">
          <nav className="flex flex-wrap items-center justify-center gap-x-7 gap-y-2">
            {NAV_LINKS.map(({ id, label }) => (
              <button
                key={id}
                type="button"
                onClick={() => navigate(id)}
                className="group relative text-[11px] font-semibold uppercase tracking-[0.17em] text-[var(--color-ink)] hover:text-[var(--color-green)]"
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
