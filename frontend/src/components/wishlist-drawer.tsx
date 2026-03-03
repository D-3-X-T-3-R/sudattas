"use client";

import { Sheet } from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { INR } from "@/lib/constants";
import type { Product } from "@/lib/schemas";

export interface WishlistDrawerProps {
  open: boolean;
  onClose: () => void;
  wishCount: number;
  wishedProducts: Product[];
  onQuickView: (p: Product) => void;
  onAddToCart: (p: Product) => void;
  onToggleWish: (p: Product) => void;
}

export function WishlistDrawer({
  open,
  onClose,
  wishCount,
  wishedProducts,
  onQuickView,
  onAddToCart,
  onToggleWish,
}: WishlistDrawerProps) {
  return (
    <Sheet
      open={open}
      onClose={onClose}
      title={`WISHLIST (${wishCount})`}
      side="right"
    >
      {wishedProducts.length === 0 ? (
        <div className="rounded-2xl bg-white p-6 text-sm text-[var(--color-muted)]">
          No items yet.
        </div>
      ) : (
        <div className="space-y-4">
          {wishedProducts.map((p) => (
            <div
              key={p.id}
              className="border-b border-[var(--color-line)] pb-4"
            >
              <div className="text-[11px] tracking-[0.18em] text-[var(--color-muted)]">
                {p.collection.toUpperCase()}
              </div>
              <div className="mt-1 text-sm font-semibold text-[var(--color-ink)]">
                {p.name}
              </div>
              <div className="mt-2 flex items-center justify-between">
                <div className="text-sm font-semibold">{INR.format(p.price)}</div>
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => onQuickView(p)}
                    className="rounded-full border-[var(--color-line)] bg-white hover:bg-white/80"
                  >
                    Quick view
                  </Button>
                  <Button
                    size="sm"
                    onClick={() => onAddToCart(p)}
                    className="rounded-full bg-[var(--color-ink)] hover:bg-[var(--color-ink)]/90"
                  >
                    Add
                  </Button>
                </div>
              </div>
              <button
                type="button"
                onClick={() => onToggleWish(p)}
                className="mt-3 text-xs font-semibold tracking-[0.18em] text-[var(--color-muted)] hover:text-[var(--color-ink)]"
              >
                REMOVE
              </button>
            </div>
          ))}
        </div>
      )}
    </Sheet>
  );
}
