"use client";

import Image from "next/image";
import Link from "next/link";
import { useState } from "react";
import { Heart, X } from "lucide-react";
import { INR } from "@/lib/constants";
import type { Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent } from "@/components/ui/dialog";

const PLACEHOLDER_IMAGE =
  "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='400' height='500' viewBox='0 0 400 500'%3E%3Crect fill='%23f0ede8' width='400' height='500'/%3E%3Ctext fill='%23999' font-family='sans-serif' font-size='14' x='50%25' y='50%25' dominant-baseline='middle' text-anchor='middle'%3ENo image%3C/text%3E%3C/svg%3E";

export type CatalogSize = { sizeId: string; sizeName: string };

function isExternalProductImage(src: string | undefined): boolean {
  if (!src || src.startsWith("/") || src.startsWith("data:")) return false;
  try {
    const host = new URL(src).hostname;
    return host !== "images.unsplash.com";
  } catch {
    return false;
  }
}

function isProductOutOfStock(p: Product): boolean {
  const vs = p.variantStock ?? [];
  if (vs.length === 0) return false;
  return !vs.some((v) => v.quantity > 0);
}

/** Same as product PDP: show selector when stock exists and is not only "Free Size". */
function hasSizeSelector(product: Product): boolean {
  const stock = product.variantStock;
  if (!Array.isArray(stock) || stock.length === 0) return false;
  const onlyFreeSize =
    stock.length === 1 && stock[0].sizeName.trim().toLowerCase() === "free size";
  return !onlyFreeSize;
}

/** Mirrors bag page: catalog order when available, else in-stock variants (non–free size). */
function buildSizeOptions(
  product: Product,
  catalog: CatalogSize[]
): { sizeId: string; sizeName: string }[] {
  const stock = product.variantStock ?? [];
  const byId = new Map(stock.map((v) => [v.sizeId, v]));

  if (catalog.length > 0) {
    return catalog
      .filter((s) => s.sizeName.toLowerCase() !== "free size")
      .map((s) => {
        const v = byId.get(s.sizeId);
        if (!v || v.quantity <= 0) return null;
        return { sizeId: s.sizeId, sizeName: s.sizeName };
      })
      .filter((x): x is { sizeId: string; sizeName: string } => x !== null);
  }

  const seen = new Set<string>();
  return stock
    .filter((v) => {
      if (v.sizeName.toLowerCase() === "free size") return false;
      if (v.quantity <= 0) return false;
      if (seen.has(v.sizeId)) return false;
      seen.add(v.sizeId);
      return true;
    })
    .map((v) => ({
      sizeId: v.sizeId,
      sizeName: v.sizeName,
    }));
}

function sizeOptionsForWishlist(product: Product, catalog: CatalogSize[]) {
  const opts = buildSizeOptions(product, catalog);
  if (opts.length > 0) return opts;
  const vs = product.variantStock ?? [];
  return vs
    .filter((v) => v.quantity > 0 && v.sizeName.toLowerCase() !== "free size")
    .map((v) => ({ sizeId: v.sizeId, sizeName: v.sizeName }));
}

export interface WishlistGridProps {
  products: Product[];
  catalogSizes: CatalogSize[];
  onRemove: (p: Product) => void;
  /** Add with chosen size; pass `null` when product is free-size only / no selector. */
  onAddToBag: (p: Product, sizeName: string | null) => void;
}

export function WishlistGrid({
  products,
  catalogSizes,
  onRemove,
  onAddToBag,
}: WishlistGridProps) {
  const [sizeDialogProduct, setSizeDialogProduct] = useState<Product | null>(null);
  const [selectedSizeName, setSelectedSizeName] = useState<string | null>(null);

  const sizeDialogOptions = sizeDialogProduct
    ? sizeOptionsForWishlist(sizeDialogProduct, catalogSizes)
    : [];

  const handleMoveClick = (p: Product) => {
    if (isProductOutOfStock(p)) return;
    if (!hasSizeSelector(p)) {
      onAddToBag(p, null);
      return;
    }
    const opts = sizeOptionsForWishlist(p, catalogSizes);
    setSelectedSizeName(opts[0]?.sizeName ?? null);
    setSizeDialogProduct(p);
  };

  const confirmSizeAndAdd = () => {
    if (!sizeDialogProduct) return;
    const effectiveSizeName =
      selectedSizeName ??
      sizeOptionsForWishlist(sizeDialogProduct, catalogSizes)[0]?.sizeName ??
      null;
    if (!effectiveSizeName) return;
    onAddToBag(sizeDialogProduct, effectiveSizeName);
    setSelectedSizeName(null);
    setSizeDialogProduct(null);
  };

  if (products.length === 0) {
    return (
      <div className="mt-24 flex flex-col items-center gap-6 text-center">
        <div className="flex h-24 w-24 items-center justify-center rounded-full border border-[var(--color-line)] bg-white">
          <Heart size={36} strokeWidth={1.25} className="text-[var(--color-accent-gold)]" />
        </div>
        <div>
          <p className="font-display text-2xl font-medium text-[var(--color-ink)]">
            Your wishlist is empty
          </p>
          <p className="mt-2 text-sm text-[var(--color-muted)]">
            Save pieces you love — they will appear here.
          </p>
        </div>
        <Link
          href="/"
          className="mt-2 rounded-full bg-[var(--color-accent-gold)] px-8 py-3 text-xs font-semibold uppercase tracking-[0.2em] text-white transition-opacity hover:opacity-90"
        >
          Continue Shopping
        </Link>
      </div>
    );
  }

  return (
    <>
      <Dialog
        open={!!sizeDialogProduct}
        onOpenChange={(open) => {
          if (!open) {
            setSizeDialogProduct(null);
            setSelectedSizeName(null);
          }
        }}
      >
        <DialogContent
          title="Select size"
          titleClassName="font-display text-base font-medium tracking-[0.12em] text-[var(--color-ink)] sm:text-lg"
          className="max-w-md rounded-2xl border border-[var(--color-line)] bg-[var(--color-warm-white)] shadow-[0_20px_50px_-12px_rgba(26,24,20,0.18),0_8px_28px_-8px_rgba(26,24,20,0.1)]"
          contentClassName="space-y-5"
        >
          {sizeDialogProduct && (
            <>
              <p className="text-sm text-[var(--color-muted)]">
                <span className="font-medium text-[var(--color-ink)]">{sizeDialogProduct.name}</span>
              </p>
              {sizeDialogOptions.length === 0 ? (
                <p className="text-sm text-[var(--color-muted)]">No sizes available.</p>
              ) : (
                <div className="space-y-2" role="listbox" aria-label="Size">
                  {sizeDialogOptions.map((o) => {
                    const selected = selectedSizeName === o.sizeName;
                    return (
                      <button
                        key={o.sizeId}
                        type="button"
                        role="option"
                        aria-selected={selected}
                        onClick={() => setSelectedSizeName(o.sizeName)}
                        className={cn(
                          "flex w-full items-center rounded-full border px-4 py-3 text-left text-base font-semibold tracking-wide transition-colors",
                          selected
                            ? "border-[var(--color-accent-gold)] bg-[var(--color-accent-gold)]/10 text-[var(--color-accent-gold)]"
                            : "border-[var(--color-line)] bg-[#F9F5F0] text-[var(--color-ink)] hover:bg-white/80"
                        )}
                      >
                        {o.sizeName}
                      </button>
                    );
                  })}
                </div>
              )}
              <div className="flex gap-3 pt-1">
                <Button
                  type="button"
                  variant="outline"
                  className="flex-1 rounded-full border-[var(--color-line)]"
                  onClick={() => setSizeDialogProduct(null)}
                >
                  Cancel
                </Button>
                <Button
                  type="button"
                  className="flex-1 rounded-full bg-[var(--color-accent-gold)] font-semibold uppercase tracking-[0.14em] text-white hover:opacity-90"
                  disabled={!selectedSizeName || sizeDialogOptions.length === 0}
                  onClick={confirmSizeAndAdd}
                >
                  Add to bag
                </Button>
              </div>
            </>
          )}
        </DialogContent>
      </Dialog>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 sm:gap-5 lg:grid-cols-3 lg:gap-6 xl:grid-cols-4">
        {products.map((p) => {
          const out = isProductOutOfStock(p);
          const priceLabel = p.priceFormatted ?? INR.format(p.price);

          return (
            <article
              key={p.id}
              className="flex flex-col overflow-hidden rounded-xl border border-[var(--color-line)] bg-[var(--color-warm-white)] shadow-[0_8px_28px_-6px_rgba(26,24,20,0.1),0_2px_8px_rgba(26,24,20,0.06)]"
            >
              <div className="relative aspect-[3/4] w-full bg-[var(--color-line)]/80">
                <Link
                  href={`/product/${p.id}`}
                  className="absolute inset-0 block"
                  aria-label={`View ${p.name}`}
                >
                  <Image
                    src={p.image || PLACEHOLDER_IMAGE}
                    alt={p.imageAlt || p.name}
                    fill
                    className="object-cover transition-transform duration-500 hover:scale-[1.02]"
                    sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 25vw"
                    unoptimized={isExternalProductImage(p.image)}
                  />
                </Link>
                <button
                  type="button"
                  onClick={() => onRemove(p)}
                  className="absolute right-3 top-3 z-10 flex h-9 w-9 items-center justify-center rounded-full border border-[var(--color-line)] bg-white/95 text-[var(--color-accent-gold)] shadow-sm backdrop-blur-sm transition-colors hover:bg-white hover:text-[var(--color-ink)]"
                  aria-label={`Remove ${p.name} from wishlist`}
                >
                  <X className="h-3.5 w-3.5" strokeWidth={2.25} />
                </button>
              </div>

              <div className="flex flex-1 flex-col bg-white px-4 pb-3 pt-4 sm:px-5 sm:pb-4 sm:pt-5">
                <Link href={`/product/${p.id}`}>
                  <h3 className="line-clamp-2 font-display text-[17px] font-medium leading-snug tracking-tight text-[var(--color-ink)]">
                    {p.name}
                  </h3>
                </Link>
                <p className="mt-1.5 font-sans text-[11px] font-semibold uppercase tracking-[0.2em] text-[var(--color-accent-gold)]">
                  {p.collection}
                </p>

                <div className="mt-4 border-t border-[var(--color-line)] pt-4">
                  <p className="font-sans text-lg font-bold tracking-tight text-[var(--color-ink)]">
                    {priceLabel}
                  </p>
                  <p className="mt-1 font-sans text-[11px] leading-relaxed text-[var(--color-muted)]">
                    MRP incl. of all taxes
                  </p>
                </div>
              </div>

              <div className="bg-white px-4 pb-4 pt-2 sm:px-5 sm:pb-5">
                {out ? (
                  <div
                    className="py-2 text-center font-sans text-xs font-semibold uppercase tracking-[0.18em] text-[var(--color-muted)]"
                    role="status"
                  >
                    Out of stock
                  </div>
                ) : (
                  <button
                    type="button"
                    onClick={() => handleMoveClick(p)}
                    className={cn(
                      "w-full rounded-full bg-[var(--color-accent-gold)] py-3.5 font-sans text-xs font-bold uppercase tracking-[0.22em] text-white",
                      "shadow-[inset_0_-1px_0_rgba(0,0,0,0.06)] transition-[opacity,transform] hover:opacity-95 active:scale-[0.99]"
                    )}
                  >
                    Move to bag
                  </button>
                )}
              </div>
            </article>
          );
        })}
      </div>
    </>
  );
}
