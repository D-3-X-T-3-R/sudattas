"use client";

import { useState, useMemo, useEffect } from "react";
import Image from "next/image";
import { ChevronUp } from "lucide-react";
import { Button } from "@/components/ui/button";
import { INR } from "@/lib/constants";
import type { Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";

/** Fallback when sizes not provided from API (e.g. legacy callers). */
const FALLBACK_SIZE_NAMES = [
  "Free Size",
  "XS",
  "S",
  "M",
  "L",
  "XL",
  "XXL",
  "3XL",
];

/** Free size = no size selector. Hide sizes section when variantStock is empty or only "Free Size". */
function hasSizeSelector(product: Product): boolean {
  const stock = product.variantStock;
  if (!Array.isArray(stock) || stock.length === 0) return false;
  const onlyFreeSize =
    stock.length === 1 &&
    stock[0].sizeName.trim().toLowerCase() === "free size";
  return !onlyFreeSize;
}

function getStockForSize(
  variantStock: { sizeName: string; quantity: number }[],
  sizeName: string
): number {
  const entry = variantStock.find(
    (s) => s.sizeName.toLowerCase() === sizeName.toLowerCase()
  );
  return entry?.quantity ?? 0;
}

function isExternalImage(src: string | undefined): boolean {
  if (!src || src.startsWith("/") || src.startsWith("data:")) return false;
  try {
    const host = new URL(src).hostname;
    return host !== "images.unsplash.com";
  } catch {
    return false;
  }
}

function Accordion({
  title,
  children,
  defaultOpen = false,
}: {
  title: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
}) {
  return (
    <details
      className="group border-b border-[var(--color-line)]"
      open={defaultOpen}
    >
      <summary className="flex cursor-pointer list-none items-center justify-between py-4 text-sm font-semibold text-[var(--color-ink)]">
        {title}
        <ChevronUp className="h-4 w-4 shrink-0 transition-transform group-open:rotate-180" />
      </summary>
      <div className="pb-4 text-sm text-[var(--color-muted)]">{children}</div>
    </details>
  );
}

export interface ProductDetailViewProps {
  product: Product;
  /** All sizes from DB (display order). When provided, used for the size list; missing or 0-qty shown struck through. */
  sizes?: { sizeId: string; sizeName: string }[];
  wished: boolean;
  onToggleWish: (p: Product) => void;
  onAddToCart: (p: Product, qty?: number) => void;
}

export function ProductDetailView({
  product,
  sizes: sizesFromApi = [],
  wished,
  onToggleWish,
  onAddToCart,
}: ProductDetailViewProps) {
  const [selectedImageIndex, setSelectedImageIndex] = useState(0);
  const variantStock = product.variantStock ?? [];
  const allSizeNames = sizesFromApi.length > 0 ? sizesFromApi.map((s) => s.sizeName) : FALLBACK_SIZE_NAMES;
  // For sized products, don't show "Free Size" in the list
  const sizeNames = hasSizeSelector(product)
    ? allSizeNames.filter((n) => n.trim().toLowerCase() !== "free size")
    : allSizeNames;
  const defaultSize =
    sizeNames.find((name) => getStockForSize(variantStock, name) > 0) ?? sizeNames[0] ?? null;
  const [selectedSize, setSelectedSize] = useState<string | null>(defaultSize);
  const [quantity, setQuantity] = useState(1);
  const [descriptionExpanded, setDescriptionExpanded] = useState(false);

  const maxQuantity =
    selectedSize != null
      ? getStockForSize(variantStock, selectedSize)
      : (variantStock[0]?.quantity ?? 999);

  useEffect(() => {
    setSelectedSize(
      sizeNames.find((name) => getStockForSize(variantStock, name) > 0) ?? sizeNames[0] ?? null
    );
  }, [product.id]);

  useEffect(() => {
    setQuantity((q) =>
      maxQuantity < 1 ? 1 : Math.min(Math.max(1, q), maxQuantity)
    );
  }, [selectedSize, maxQuantity]);

  const images = useMemo(() => {
    if (product.images?.length) return product.images;
    const list = [product.image];
    if (product.hoverImage && product.hoverImage !== product.image) {
      list.push(product.hoverImage);
    }
    return list.filter(Boolean).length ? list : [product.image || ""];
  }, [product]);

  const mainImage = images[selectedImageIndex] || product.image;
  const descShort = product.description.slice(0, 160);
  const hasMoreDesc = product.description.length > 160;

  return (
    <div className="grid min-w-0 gap-8 md:grid-cols-2 md:items-start">
      {/* Left: image gallery — match page background so image border blends in */}
      <div className="min-w-0 overflow-hidden flex gap-3 bg-[var(--background)] p-4 md:p-6 md:self-start">
        <div className="flex shrink-0 flex-col gap-2">
          {images.map((src, i) => (
            <button
              key={i}
              type="button"
              onClick={() => setSelectedImageIndex(i)}
              className={cn(
                "relative h-14 w-14 shrink-0 overflow-hidden rounded border-2 transition-colors md:h-16 md:w-16",
                selectedImageIndex === i
                  ? "border-[var(--color-ink)]"
                  : "border-transparent hover:border-[var(--color-line)]"
              )}
            >
              <Image
                src={src}
                alt={`${product.name} view ${i + 1}`}
                fill
                className="object-cover"
                sizes="64px"
                unoptimized={isExternalImage(src)}
              />
            </button>
          ))}
        </div>
        <div className="relative min-h-0 min-w-0 flex-1 overflow-hidden rounded-sm aspect-[3/4]">
          <Image
            src={mainImage}
            alt={product.imageAlt || product.name}
            fill
            className="object-cover"
            sizes="(max-width: 768px) 100vw, 50vw"
            unoptimized={isExternalImage(mainImage)}
          />
        </div>
      </div>

      {/* Right: details + actions + accordions */}
      <div className="min-w-0 flex flex-col overflow-visible p-4 md:p-6 md:pl-6 md:pr-8">
        <div className="text-[11px] font-semibold tracking-[0.24em] text-[var(--color-muted)]">
          {product.collection.toUpperCase()}
        </div>
        <div className="mt-2 text-2xl font-semibold text-[var(--color-ink)]">
          {product.priceFormatted ?? INR.format(product.price)} INR
        </div>
        <h1 className="mt-3 font-display text-xl font-semibold tracking-tight text-[var(--color-ink)] md:text-2xl">
          {product.name}
        </h1>

        <p className="mt-4 text-sm leading-relaxed text-[var(--color-muted)]">
          {descriptionExpanded || !hasMoreDesc
            ? product.description
            : `${descShort}${hasMoreDesc ? "…" : ""}`}
          {hasMoreDesc && (
            <button
              type="button"
              onClick={() => setDescriptionExpanded(true)}
              className="ml-1 font-semibold text-[var(--color-ink)] underline"
            >
              Read more
            </button>
          )}
        </p>

        {hasSizeSelector(product) && (
          <div className="mt-6">
            <div className="text-sm font-semibold text-[var(--color-ink)]">
              Size{" "}
              <button
                type="button"
                className="font-normal text-[var(--color-muted)] underline"
              >
                Chart
              </button>
            </div>
            <div className="mt-2 flex flex-wrap gap-2">
              {sizeNames.map((sizeName) => {
                const qty = getStockForSize(variantStock, sizeName);
                const outOfStock = qty <= 0;
                return (
                  <button
                    key={sizeName}
                    type="button"
                    disabled={outOfStock}
                    onClick={() => !outOfStock && setSelectedSize(sizeName)}
                    className={cn(
                      "min-w-[2.5rem] rounded border px-3 py-2 text-sm font-medium transition-colors",
                      outOfStock &&
                        "cursor-not-allowed border-[var(--color-line)] bg-[var(--color-line)]/10 text-[var(--color-muted)] line-through",
                      !outOfStock &&
                        selectedSize === sizeName &&
                        "border-[var(--color-ink)] bg-[var(--color-ink)] text-white",
                      !outOfStock &&
                        selectedSize !== sizeName &&
                        "border-[var(--color-line)] bg-white text-[var(--color-ink)] hover:border-[var(--color-ink)]"
                    )}
                  >
                    {sizeName}
                  </button>
                );
              })}
            </div>
          </div>
        )}

        <div className="mt-4">
          <div className="text-sm font-semibold text-[var(--color-ink)]">
            Quantity
          </div>
          <div className="mt-2 flex items-center gap-2">
            <button
              type="button"
              onClick={() => setQuantity((q) => Math.max(1, q - 1))}
              disabled={quantity <= 1}
              className="flex h-10 w-10 items-center justify-center rounded border border-[var(--color-line)] text-lg font-medium hover:bg-[var(--color-line)]/20 disabled:cursor-not-allowed disabled:opacity-50"
            >
              −
            </button>
            <span className="min-w-[2rem] text-center text-sm font-medium">
              {quantity}
            </span>
            <button
              type="button"
              onClick={() => setQuantity((q) => Math.min(maxQuantity, q + 1))}
              disabled={quantity >= maxQuantity}
              className="flex h-10 w-10 items-center justify-center rounded border border-[var(--color-line)] text-lg font-medium hover:bg-[var(--color-line)]/20 disabled:cursor-not-allowed disabled:opacity-50"
            >
              +
            </button>
          </div>
        </div>

        <Button
          onClick={() => onAddToCart(product, quantity)}
          disabled={maxQuantity < 1}
          className="mt-6 w-full rounded-full bg-[var(--color-ink)] py-6 text-base font-semibold hover:bg-[var(--color-ink)]/90 disabled:cursor-not-allowed disabled:opacity-50"
        >
          ADD TO CART
        </Button>

        <Button
          variant="outline"
          onClick={() => onToggleWish(product)}
          className="mt-4 w-full rounded-full border-[var(--color-line)]"
        >
          {wished ? "Wishlisted" : "Add to wishlist"}
        </Button>

        <div className="mt-8">
          <Accordion title="Details" defaultOpen>
            <div className="grid gap-2 text-sm">
              <div className="flex justify-between">
                <span>Material</span>
                <span className="text-[var(--color-ink)]">{product.fabric}</span>
              </div>
              <div className="flex justify-between">
                <span>Occasion</span>
                <span className="text-[var(--color-ink)]">{product.occasion}</span>
              </div>
            </div>
          </Accordion>
          <Accordion title="Payment">
            <p>
              Order is made to order. Dispatch in 7–10 working days; transit
              4–5 days. Cash on Delivery available.
            </p>
          </Accordion>
          <Accordion title="Exchange">
            <ul className="list-disc space-y-1 pl-5">
              <li>3 days easy return & exchange</li>
              <li>Free pickup for returns & exchanges</li>
              <li>Returns/exchanges allowed only for unused products with tags</li>
            </ul>
          </Accordion>
          <Accordion title="Care">
            <ul className="list-disc space-y-1 pl-5">
              <li>Hand wash in cold water</li>
              <li>Dry clean recommended for Sarees</li>
              <li>Wash inside out</li>
              <li>Iron on low heat</li>
            </ul>
          </Accordion>
        </div>
      </div>
    </div>
  );
}
