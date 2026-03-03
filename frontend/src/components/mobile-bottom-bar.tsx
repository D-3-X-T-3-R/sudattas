"use client";

import { Heart, ShoppingBag } from "lucide-react";
import { Button } from "@/components/ui/button";
import { goTo } from "@/hooks/use-scroll-to";

export interface MobileBottomBarProps {
  activeSection: string;
  wishCount: number;
  cartCount: number;
  onWishOpen: () => void;
  onCartOpen: () => void;
  reduceMotion?: boolean;
}

export function MobileBottomBar({
  activeSection,
  wishCount,
  cartCount,
  onWishOpen,
  onCartOpen,
  reduceMotion = false,
}: MobileBottomBarProps) {
  return (
    <div className="fixed bottom-0 left-0 right-0 z-20 border-t border-[var(--color-line)] bg-[rgba(247,245,240,0.88)] backdrop-blur md:hidden">
      <div className="mx-auto max-w-7xl px-4 py-2">
        <div className="flex items-center justify-between">
          <button
            type="button"
            onClick={() =>
              goTo(activeSection === "top" ? "shop" : "top", reduceMotion)
            }
            className="text-xs font-semibold tracking-[0.18em] text-[var(--color-ink)]"
          >
            {activeSection === "top" ? "SHOP" : "TOP"}
          </button>
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="icon"
              onClick={onWishOpen}
              aria-label="Wishlist"
              className="relative border-[var(--color-line)] bg-white"
            >
              <Heart className="h-5 w-5" />
              {wishCount > 0 && (
                <span className="absolute -right-1 -top-1 flex h-5 w-5 items-center justify-center rounded-full bg-[var(--color-ink)] text-[10px] font-semibold text-white">
                  {wishCount}
                </span>
              )}
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={onCartOpen}
              aria-label="Bag"
              className="relative border-[var(--color-line)] bg-white"
            >
              <ShoppingBag className="h-5 w-5" />
              {cartCount > 0 && (
                <span className="absolute -right-1 -top-1 flex h-5 w-5 items-center justify-center rounded-full bg-[var(--color-ink)] text-[10px] font-semibold text-white">
                  {cartCount}
                </span>
              )}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
