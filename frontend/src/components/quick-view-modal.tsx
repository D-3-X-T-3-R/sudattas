"use client";

import Image from "next/image";
import {
  Dialog,
  DialogContent,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Rating } from "@/components/rating";
import { INR } from "@/lib/constants";
import type { Product } from "@/lib/schemas";

export interface QuickViewModalProps {
  product: Product | null;
  open: boolean;
  onClose: () => void;
  wished: boolean;
  onToggleWish: (p: Product) => void;
  onAddToCart: (p: Product) => void;
}

export function QuickViewModal({
  product,
  open,
  onClose,
  wished,
  onToggleWish,
  onAddToCart,
}: QuickViewModalProps) {
  if (!product) return null;

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent
        title={product.name.toUpperCase()}
        showClose
        onPointerDownOutside={onClose}
        onEscapeKeyDown={onClose}
      >
        <div className="grid gap-8 md:grid-cols-2">
          <div className="bg-white">
            <div className="relative aspect-[3/4] w-full">
              <Image
                src={product.image}
                alt={product.imageAlt || product.name}
                fill
                className="object-cover"
                sizes="(max-width: 768px) 100vw, 50vw"
              />
            </div>
          </div>
          <div>
            <div className="text-[11px] font-semibold tracking-[0.24em] text-[var(--color-muted)]">
              {product.collection.toUpperCase()}
            </div>
            <div className="mt-2 font-display text-2xl font-semibold tracking-tight text-[var(--color-ink)]">
              {product.name}
            </div>
            <div className="mt-4 flex items-center justify-between">
              <div className="text-lg font-semibold">
                {INR.format(product.price)}
              </div>
              <Rating value={product.rating} />
            </div>

            <p className="mt-5 text-sm leading-relaxed text-[var(--color-muted)]">
              {product.description}
            </p>

            <div className="mt-6 space-y-2 text-sm">
              <div>
                <span className="font-semibold">Fabric:</span> {product.fabric}
              </div>
              <div>
                <span className="font-semibold">Occasion:</span>{" "}
                {product.occasion}
              </div>
            </div>

            <div className="mt-8 flex flex-col gap-3 sm:flex-row">
              <Button
                variant="outline"
                onClick={() => onToggleWish(product)}
                className="rounded-full border-[var(--color-line)] bg-white hover:bg-white/80"
              >
                {wished ? "Wishlisted" : "Add to wishlist"}
              </Button>
              <Button
                onClick={() => onAddToCart(product)}
                className="rounded-full bg-[var(--color-ink)] hover:bg-[var(--color-ink)]/90"
              >
                Add to bag
              </Button>
            </div>

            <div className="mt-8 rounded-2xl border border-[var(--color-line)] bg-white p-4">
              <div className="text-[11px] font-semibold tracking-[0.24em] text-[var(--color-muted)]">
                CARE
              </div>
              <ul className="mt-3 list-disc space-y-1 pl-5 text-sm text-[var(--color-muted)]">
                <li>Dry clean recommended for first wash.</li>
                <li>Store folded with muslin.</li>
                <li>Avoid direct perfume spray on zari.</li>
              </ul>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
