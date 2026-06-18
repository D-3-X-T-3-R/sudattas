"use client";

import Image from "next/image";
import Link from "next/link";
import { useState } from "react";
import { X } from "lucide-react";
import { INR } from "@/lib/constants";
import type { Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { EmptyState } from "@/components/ui/page-shell";

const PLACEHOLDER_IMAGE =
  "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='400' height='500' viewBox='0 0 400 500'%3E%3Crect fill='%23f0ede8' width='400' height='500'/%3E%3Ctext fill='%23999' font-family='sans-serif' font-size='14' x='50%25' y='50%25' dominant-baseline='middle' text-anchor='middle'%3ENo image%3C/text%3E%3C/svg%3E";

export type CatalogSize = { sizeId: string; sizeName: string };

function isExternalProductImage(src: string | undefined): boolean {
  if (!src || src.startsWith("/") || src.startsWith("data:")) return false;
  try {
    return new URL(src).hostname !== "images.unsplash.com";
  } catch {
    return false;
  }
}

function isProductOutOfStock(p: Product): boolean {
  const vs = p.variantStock ?? [];
  if (vs.length === 0) return false;
  return !vs.some((v) => v.quantity > 0);
}

function hasSizeSelector(product: Product): boolean {
  const stock = product.variantStock;
  if (!Array.isArray(stock) || stock.length === 0) return false;
  return !(stock.length === 1 && stock[0].sizeName.trim().toLowerCase() === "free size");
}

function buildSizeOptions(product: Product, catalog: CatalogSize[]): { sizeId: string; sizeName: string }[] {
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
    .map((v) => ({ sizeId: v.sizeId, sizeName: v.sizeName }));
}

function sizeOptionsForWishlist(product: Product, catalog: CatalogSize[]) {
  const opts = buildSizeOptions(product, catalog);
  if (opts.length > 0) return opts;
  const vs = product.variantStock ?? [];
  return vs
    .filter((v) => v.quantity > 0 && v.sizeName.toLowerCase() !== "free size")
    .map((v) => ({ sizeId: v.sizeId, sizeName: v.sizeName }));
}

function SizePickerDialog({
  product,
  options,
  selectedSizeName,
  setSelectedSizeName,
  onClose,
  onConfirm,
}: {
  product: Product | null;
  options: { sizeId: string; sizeName: string }[];
  selectedSizeName: string | null;
  setSelectedSizeName: (value: string | null) => void;
  onClose: () => void;
  onConfirm: () => void;
}) {
  return (
    <Dialog open={!!product} onOpenChange={(open) => !open && onClose()}>
      <DialogContent
        title="Select size"
        titleClassName="font-display text-lg text-[var(--color-ink)]"
        className="max-w-md"
        contentClassName="space-y-4"
      >
        {product ? (
          <>
            <p className="text-sm text-[var(--color-muted)]">
              {product.name}
            </p>
            {options.length === 0 ? (
              <p className="text-sm text-[var(--color-muted)]">No sizes available.</p>
            ) : (
              <div className="space-y-2" role="listbox" aria-label="Size">
                {options.map((o) => {
                  const selected = selectedSizeName === o.sizeName;
                  return (
                    <button
                      key={o.sizeId}
                      type="button"
                      role="option"
                      aria-selected={selected}
                      onClick={() => setSelectedSizeName(o.sizeName)}
                      className={cn(
                        "flex w-full items-center rounded-md border px-4 py-3 text-left text-sm font-semibold uppercase tracking-[0.12em]",
                        selected
                          ? "border-[var(--color-green)] bg-[var(--color-green)] text-white"
                          : "border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-ink)] hover:border-[var(--color-gold)]"
                      )}
                    >
                      {o.sizeName}
                    </button>
                  );
                })}
              </div>
            )}
            <div className="grid grid-cols-2 gap-2">
              <Button type="button" variant="outline" onClick={onClose}>
                Cancel
              </Button>
              <Button type="button" disabled={!selectedSizeName || options.length === 0} onClick={onConfirm}>
                Add To Bag
              </Button>
            </div>
          </>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}

function WishlistCard({
  p,
  onRemove,
  onMoveToBag,
}: {
  p: Product;
  onRemove: (p: Product) => void;
  onMoveToBag: (p: Product) => void;
}) {
  const out = isProductOutOfStock(p);
  const priceLabel = p.priceFormatted ?? INR.format(p.price);

  return (
    <article className="group flex flex-col">
      <div className="relative aspect-[3/4] overflow-hidden rounded-[var(--radius-md)] border border-[var(--color-line)] bg-[var(--color-surface-soft)]">
        <Link href={`/product/${p.id}`} className="block h-full w-full" aria-label={`View ${p.name}`}>
          <Image
            src={p.image || PLACEHOLDER_IMAGE}
            alt={p.imageAlt || p.name}
            fill
            className="object-cover transition-transform duration-500 ease-out group-hover:scale-[1.04]"
            sizes="(max-width: 768px) 50vw, 25vw"
            unoptimized={isExternalProductImage(p.image)}
          />
        </Link>
        <button
          type="button"
          onClick={() => onRemove(p)}
          className="absolute right-3 top-3 z-10 flex h-8 w-8 items-center justify-center rounded-full border border-[var(--color-line)] bg-[var(--color-surface)]/90 text-[var(--color-ink)] backdrop-blur-sm transition-colors hover:text-[var(--color-green)]"
          aria-label={`Remove ${p.name} from wishlist`}
        >
          <X className="h-4 w-4" strokeWidth={2.2} />
        </button>
      </div>

      <div className="mt-3 flex items-start justify-between gap-3 sm:mt-4">
        <Link href={`/product/${p.id}`} className="min-w-0">
          <h3 className="line-clamp-2 break-words font-display text-[1.05rem] leading-snug text-[var(--color-ink)] sm:text-xl">{p.name}</h3>
        </Link>
        <p className="whitespace-nowrap pt-0.5 font-sans text-sm font-semibold text-[var(--color-ink)] sm:text-base">{priceLabel}</p>
      </div>

      <div className="mt-3">
        {out ? (
          <div className="rounded-[var(--radius-md)] border border-[var(--color-line)] bg-[var(--color-surface-soft)] py-2.5 text-center text-xs font-semibold uppercase tracking-[0.16em] text-[var(--color-muted)]" role="status">
            Out Of Stock
          </div>
        ) : (
          <Button type="button" onClick={() => onMoveToBag(p)} className="w-full">
            Move To Bag
          </Button>
        )}
      </div>
    </article>
  );
}

export interface WishlistGridProps {
  products: Product[];
  catalogSizes: CatalogSize[];
  onRemove: (p: Product) => void;
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
      <EmptyState
        title="Your wishlist is empty"
        description="Save pieces you love and they will appear here."
        action={
          <Link
            href="/"
            className="inline-flex h-11 items-center justify-center rounded-md border border-[var(--color-green)] bg-[var(--color-green)] px-6 text-[11px] font-semibold uppercase tracking-[0.16em] text-white"
          >
            Continue Shopping
          </Link>
        }
      />
    );
  }

  return (
    <>
      <SizePickerDialog
        product={sizeDialogProduct}
        options={sizeDialogOptions}
        selectedSizeName={selectedSizeName}
        setSelectedSizeName={setSelectedSizeName}
        onClose={() => {
          setSizeDialogProduct(null);
          setSelectedSizeName(null);
        }}
        onConfirm={confirmSizeAndAdd}
      />

      <div className="grid grid-cols-2 gap-4 md:gap-5 lg:grid-cols-4">
        {products.map((p) => (
          <WishlistCard key={p.id} p={p} onRemove={onRemove} onMoveToBag={handleMoveClick} />
        ))}
      </div>
    </>
  );
}
