"use client";

import Link from "next/link";
import { Heart, Search, ShoppingBag, User } from "lucide-react";
import { cn } from "@/lib/utils";

export interface MobileBottomBarProps {
  wishCount: number;
  cartCount: number;
  authenticated: boolean;
  onProfileOpen: () => void;
  onSearchOpen: () => void;
}

export function MobileBottomBar({
  wishCount,
  cartCount,
  authenticated,
  onProfileOpen,
  onSearchOpen,
}: MobileBottomBarProps) {
  const itemClass =
    "relative flex min-h-14 min-w-0 flex-col items-center justify-center gap-1 rounded-md px-1 text-[10px] font-semibold uppercase tracking-[0.12em] text-[var(--color-ink)] transition-colors hover:bg-[var(--color-surface-soft)] hover:text-[var(--color-green)]";
  const iconWrapClass =
    "relative inline-flex h-6 w-6 items-center justify-center";

  return (
    <nav
      className="mobile-bottom-bar fixed bottom-0 left-0 right-0 z-40 border-t border-[var(--color-line)] bg-[var(--color-surface)]/98 pb-[env(safe-area-inset-bottom)] shadow-[0_-10px_28px_rgba(45,42,38,0.08)] backdrop-blur lg:hidden"
      aria-label="Mobile utilities"
    >
      <div className="mx-auto grid w-full max-w-[var(--container-max)] grid-cols-4 gap-1 px-[var(--gutter-mobile)] py-1.5">
        {authenticated ? (
          <Link href="/profile" className={itemClass} aria-label="Profile">
            <span className={iconWrapClass}>
              <User className="h-5 w-5" />
            </span>
            <span className="truncate">Profile</span>
          </Link>
        ) : (
          <button type="button" onClick={onProfileOpen} className={itemClass} aria-label="Sign in">
            <span className={iconWrapClass}>
              <User className="h-5 w-5" />
            </span>
            <span className="truncate">Profile</span>
          </button>
        )}

        <button type="button" onClick={onSearchOpen} className={itemClass} aria-label="Search">
          <span className={iconWrapClass}>
            <Search className="h-5 w-5" />
          </span>
          <span className="truncate">Search</span>
        </button>

        <Link
          href="/wishlist"
          className={cn(itemClass, wishCount > 0 && "text-[var(--color-gold)]")}
          aria-label="Wishlist"
        >
          <span className={iconWrapClass}>
            <Heart className="h-5 w-5" />
            {wishCount > 0 && (
              <span className="absolute -right-2 -top-2 flex h-5 min-w-5 items-center justify-center rounded-sm bg-[var(--color-gold)] px-1 font-sans text-[10px] font-semibold text-white">
                {wishCount}
              </span>
            )}
          </span>
          <span className="truncate">Wishlist</span>
        </Link>

        <Link
          href="/bag"
          className={cn(itemClass, cartCount > 0 && "text-[var(--color-gold)]")}
          aria-label="Bag"
        >
          <span className={iconWrapClass}>
            <ShoppingBag className="h-5 w-5" />
            {cartCount > 0 && (
              <span className="absolute -right-2 -top-2 flex h-5 min-w-5 items-center justify-center rounded-sm bg-[var(--color-gold)] px-1 font-sans text-[10px] font-semibold text-white">
                {cartCount}
              </span>
            )}
          </span>
          <span className="truncate">Bag</span>
        </Link>
      </div>
    </nav>
  );
}
