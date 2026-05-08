"use client";

import { useMemo, useState } from "react";
import Image from "next/image";
import Link from "next/link";
import { ShieldCheck, Truck, Undo2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { INR } from "@/lib/constants";
import type { Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";

const FALLBACK_SIZE_NAMES = ["Free Size", "XS", "S", "M", "L", "XL", "XXL", "3XL"];

type VariantStockRow = { sizeName: string; quantity: number };

function hasSizeSelector(product: Product): boolean {
  const stock = product.variantStock;
  if (!Array.isArray(stock) || stock.length === 0) return false;
  return !(stock.length === 1 && stock[0].sizeName.trim().toLowerCase() === "free size");
}

function getStockForSize(variantStock: VariantStockRow[], sizeName: string): number {
  return (
    variantStock.find((s) => s.sizeName.toLowerCase() === sizeName.toLowerCase())
      ?.quantity ?? 0
  );
}

function isExternalImage(src: string | undefined): boolean {
  if (!src || src.startsWith("/") || src.startsWith("data:")) return false;
  try {
    return new URL(src).hostname !== "images.unsplash.com";
  } catch {
    return false;
  }
}

function ProductGallery({
  images,
  mainImage,
  productName,
  selectedImageIndex,
  setSelectedImageIndex,
}: {
  images: string[];
  mainImage: string;
  productName: string;
  selectedImageIndex: number;
  setSelectedImageIndex: (value: number) => void;
}) {
  return (
    <div className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-3 shadow-[var(--shadow-subtle)] md:sticky md:top-24 md:p-4">
      <div className="grid gap-3 md:grid-cols-[72px_minmax(0,1fr)]">
        <div className="order-2 flex gap-2 overflow-x-auto pb-1 md:order-1 md:flex-col md:overflow-visible">
          {images.map((src, i) => (
            <button
              key={i}
              type="button"
              onClick={() => setSelectedImageIndex(i)}
              aria-label={`View image ${i + 1} of ${images.length} for ${productName}`}
              className={cn(
                "relative h-16 w-14 shrink-0 overflow-hidden rounded-sm border",
                selectedImageIndex === i
                  ? "border-[var(--color-green)]"
                  : "border-[var(--color-line)] hover:border-[var(--color-gold)]"
              )}
            >
              <Image
                src={src}
                alt={`${productName} view ${i + 1}`}
                fill
                className="object-cover"
                sizes="64px"
                unoptimized={isExternalImage(src)}
              />
            </button>
          ))}
        </div>

        <div className="order-1 relative min-h-0 overflow-hidden rounded-sm border border-[var(--color-line)] bg-white aspect-[3/4] md:order-2">
          <Image
            src={mainImage}
            alt={productName}
            fill
            className="object-cover"
            sizes="(max-width: 768px) 100vw, 50vw"
            unoptimized={isExternalImage(mainImage)}
          />
        </div>
      </div>
    </div>
  );
}

function ProductDetails({ product }: { product: Product }) {
  return (
    <div className="space-y-2">
      <details className="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] p-4" open>
        <summary className="cursor-pointer text-sm font-semibold text-[var(--color-ink)]">Details</summary>
        <div className="mt-3 grid gap-2 text-sm text-[var(--color-muted)]">
          <div className="flex justify-between gap-2">
            <span>Material</span>
            <span className="text-[var(--color-ink)]">{product.fabric || "N/A"}</span>
          </div>
          <div className="flex justify-between gap-2">
            <span>Occasion</span>
            <span className="text-[var(--color-ink)]">{product.occasion || "N/A"}</span>
          </div>
        </div>
      </details>

      <details className="rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] p-4">
        <summary className="cursor-pointer text-sm font-semibold text-[var(--color-ink)]">Care Instructions</summary>
        <ul className="mt-3 list-disc space-y-1 pl-4 text-sm text-[var(--color-muted)]">
          <li>Dry clean or gentle hand wash recommended.</li>
          <li>Do not bleach or wring aggressively.</li>
          <li>Dry in shade and iron on reverse side.</li>
          <li>Store folded in a breathable cotton cover.</li>
        </ul>
      </details>
    </div>
  );
}

export interface ProductDetailViewProps {
  product: Product;
  sizes?: { sizeId: string; sizeName: string }[];
  wished: boolean;
  onToggleWish: (p: Product) => void;
  onAddToCart: (p: Product, qty?: number, sizeName?: string | null) => void;
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

  const allSizeNames =
    sizesFromApi.length > 0
      ? sizesFromApi.map((s) => s.sizeName)
      : FALLBACK_SIZE_NAMES;

  const sizeNames = hasSizeSelector(product)
    ? allSizeNames.filter((n) => n.trim().toLowerCase() !== "free size")
    : allSizeNames;

  const defaultSize =
    sizeNames.find((name) => getStockForSize(variantStock, name) > 0) ??
    sizeNames[0] ??
    null;

  const [selectedSize, setSelectedSize] = useState<string | null>(defaultSize);
  const [quantity, setQuantity] = useState(1);

  const maxQuantity =
    selectedSize != null
      ? getStockForSize(variantStock, selectedSize)
      : variantStock[0]?.quantity ?? 999;

  const safeQuantity = maxQuantity < 1 ? 1 : Math.min(Math.max(1, quantity), maxQuantity);

  const images = useMemo(() => {
    if (product.images?.length) return product.images;
    const list = [product.image];
    if (product.hoverImage && product.hoverImage !== product.image) {
      list.push(product.hoverImage);
    }
    return list.filter(Boolean).length ? list : [product.image || ""];
  }, [product]);

  const mainImage = images[selectedImageIndex] || product.image;

  return (
    <section className="grid gap-6 md:grid-cols-[58%_42%] md:items-start">
      <ProductGallery
        images={images}
        mainImage={mainImage}
        productName={product.imageAlt || product.name}
        selectedImageIndex={selectedImageIndex}
        setSelectedImageIndex={setSelectedImageIndex}
      />

      <div className="space-y-5 rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-subtle)] md:sticky md:top-24 md:p-5">
        <div className="border-b border-[var(--color-line)] pb-4">
          <p className="text-[10px] font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">
            {product.collection}
          </p>
          <h1 className="mt-1 font-display text-[1.9rem] leading-[1.18] text-[var(--color-ink)] md:text-[2.3rem]">
            {product.name}
          </h1>
          <p className="mt-2 text-sm leading-relaxed text-[var(--color-muted)]">{product.description}</p>
          <p className="mt-3 font-sans text-2xl font-semibold text-[var(--color-ink)]">
            {product.priceFormatted ?? INR.format(product.price)}
          </p>
        </div>

        <div>
          {hasSizeSelector(product) ? (
            <>
              <div className="flex items-center justify-between gap-2">
                <p className="text-sm font-semibold text-[var(--color-ink)]">Select size</p>
                <Link href="/size-fit-guide" className="text-xs font-semibold uppercase tracking-[0.12em] text-[var(--color-green)]">
                  Size &amp; Fit Guide
                </Link>
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
                        "min-w-[2.7rem] rounded-sm border px-3 py-2 text-sm font-medium",
                        outOfStock && "cursor-not-allowed border-[var(--color-line)] bg-[var(--color-surface-soft)] text-[var(--color-muted)] line-through",
                        !outOfStock && selectedSize === sizeName && "border-[var(--color-green)] bg-[var(--color-green)] text-white",
                        !outOfStock && selectedSize !== sizeName && "border-[var(--color-line)] bg-white text-[var(--color-ink)] hover:border-[var(--color-gold)]"
                      )}
                    >
                      {sizeName}
                    </button>
                  );
                })}
              </div>
            </>
          ) : (
            <div className="rounded-md border border-[var(--color-line)] bg-[var(--color-surface-soft)] p-3 text-sm text-[var(--color-muted)]">
              This style does not use standard size variants. Free-size drape fit can vary by fabric and silhouette.
              <Link href="/size-fit-guide" className="ml-2 text-xs font-semibold uppercase tracking-[0.12em] text-[var(--color-green)]">
                View Size &amp; Fit Guide
              </Link>
            </div>
          )}
        </div>

        <div>
          <p className="text-sm font-semibold text-[var(--color-ink)]">Quantity</p>
          <div className="mt-2 inline-flex items-center rounded-md border border-[var(--color-line)] bg-white p-1">
            <button
              type="button"
              onClick={() => setQuantity((q) => Math.max(1, q - 1))}
              disabled={safeQuantity <= 1}
              className="h-8 w-8 rounded-sm text-lg text-[var(--color-ink)] disabled:opacity-40"
              aria-label="Decrease quantity"
            >
              -
            </button>
            <span className="min-w-[2.2rem] text-center text-sm font-semibold text-[var(--color-ink)]">{safeQuantity}</span>
            <button
              type="button"
              onClick={() => setQuantity((q) => Math.min(maxQuantity, q + 1))}
              disabled={safeQuantity >= maxQuantity}
              className="h-8 w-8 rounded-sm text-lg text-[var(--color-ink)] disabled:opacity-40"
              aria-label="Increase quantity"
            >
              +
            </button>
          </div>
        </div>

        <div className="grid gap-2">
          <Button
            onClick={() => onAddToCart(product, safeQuantity, selectedSize)}
            disabled={maxQuantity < 1}
            className="w-full"
          >
            ADD TO BAG
          </Button>
          <Button variant="outline" onClick={() => onToggleWish(product)} className="w-full">
            {wished ? "Wishlisted" : "Add to Wishlist"}
          </Button>
        </div>

        <div className="grid gap-2 rounded-md border border-[var(--color-line)] bg-[var(--color-surface-soft)] p-3">
          <div className="flex items-start gap-2 text-xs text-[var(--color-muted)]">
            <ShieldCheck className="mt-0.5 h-4 w-4 text-[var(--color-green)]" />
            Secure checkout and protected payments.
          </div>
          <div className="flex items-start gap-2 text-xs text-[var(--color-muted)]">
            <Truck className="mt-0.5 h-4 w-4 text-[var(--color-green)]" />
            Delivery timelines confirmed at checkout for your pincode.
          </div>
          <div className="flex items-start gap-2 text-xs text-[var(--color-muted)]">
            <Undo2 className="mt-0.5 h-4 w-4 text-[var(--color-green)]" />
            Easy return support available from your order dashboard.
          </div>
        </div>

        <ProductDetails product={product} />
      </div>
    </section>
  );
}
