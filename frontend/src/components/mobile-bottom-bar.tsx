"use client";

import Link from "next/link";
import { Heart, ShoppingBag } from "lucide-react";
import { Button } from "@/components/ui/button";
import { goTo } from "@/hooks/use-scroll-to";
import { cn } from "@/lib/utils";

export interface MobileBottomBarProps {
  activeSection: string;
  wishCount: number;
  cartCount: number;
  onCartOpen: () => void;
  reduceMotion?: boolean;
}

export function MobileBottomBar({
  activeSection,
  wishCount,
  cartCount,
  onCartOpen,
  reduceMotion = false,
}: MobileBottomBarProps) {
  return (
    <div className="fixed bottom-0 left-0 right-0 z-20 border-t border-[var(--color-line)] bg-[var(--color-surface)]/95 backdrop-blur md:hidden">
      <div className="mx-auto max-w-[var(--container-max)] px-[var(--gutter-mobile)] py-2">
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
              aria-label="Wishlist"
              className={cn(
                "relative h-9 w-9 rounded-md border-[var(--color-line)] bg-[var(--color-surface)]",
                wishCount > 0 && "border-[var(--color-gold)] text-[var(--color-gold)]"
              )}
              asChild
            >
              <Link href="/wishlist">
                <Heart className="h-5 w-5" />
                {wishCount > 0 && (
                  <span className="absolute -right-1 -top-1 flex h-5 w-5 items-center justify-center rounded-sm bg-[var(--color-gold)] font-sans text-[10px] font-semibold text-white">
                    {wishCount}
                  </span>
                )}
              </Link>
            </Button>
            <Button
              variant="outline"
              size="icon"
              onClick={onCartOpen}
              aria-label="Bag"
              className={cn(
                "relative h-9 w-9 rounded-md border-[var(--color-line)] bg-[var(--color-surface)]",
                cartCount > 0 && "text-[var(--color-gold)] border-[var(--color-gold)]"
              )}
            >
              <ShoppingBag className="h-5 w-5" />
              {cartCount > 0 && (
                <span className="absolute -right-1 -top-1 flex h-5 w-5 items-center justify-center rounded-sm bg-[var(--color-gold)] font-sans text-[10px] font-semibold text-white">
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
