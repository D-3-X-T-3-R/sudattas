"use client";

import { useMemo, useState } from "react";
import Image from "next/image";
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
  return variantStock.find((s) => s.sizeName.toLowerCase() === sizeName.toLowerCase())?.quantity ?? 0;
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
    <div className="min-w-0 overflow-hidden flex gap-3 bg-[var(--background)] p-4 md:p-6 md:self-start">
      <div className="flex shrink-0 flex-col gap-2">
        {images.map((src, i) => (
          <button
            key={i}
            type="button"
            onClick={() => setSelectedImageIndex(i)}
            className={cn(
              "relative h-14 w-14 shrink-0 overflow-hidden rounded border-2 transition-colors md:h-16 md:w-16",
              selectedImageIndex === i ? "border-[var(--color-ink)]" : "border-transparent hover:border-[var(--color-line)]"
            )}
          >
            <Image src={src} alt={`${productName} view ${i + 1}`} fill className="object-cover" sizes="64px" unoptimized={isExternalImage(src)} />
          </button>
        ))}
      </div>
      <div className="relative min-h-0 min-w-0 flex-1 overflow-hidden rounded-sm aspect-[3/4]">
        <Image src={mainImage} alt={productName} fill className="object-cover" sizes="(max-width: 768px) 100vw, 50vw" unoptimized={isExternalImage(mainImage)} />
      </div>
    </div>
  );
}

function SizeSelector({
  product,
  sizeNames,
  variantStock,
  selectedSize,
  setSelectedSize,
}: {
  product: Product;
  sizeNames: string[];
  variantStock: VariantStockRow[];
  selectedSize: string | null;
  setSelectedSize: (value: string) => void;
}) {
  if (!hasSizeSelector(product)) return null;
  return (
    <div className="mt-6">
      <div className="text-sm font-semibold text-[var(--color-ink)]">Size <button type="button" className="font-normal text-[var(--color-muted)] underline">Chart</button></div>
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
                outOfStock && "cursor-not-allowed border-[var(--color-line)] bg-[var(--color-line)]/10 text-[var(--color-muted)] line-through",
                !outOfStock && selectedSize === sizeName && "border-[var(--color-ink)] bg-[var(--color-ink)] text-white",
                !outOfStock && selectedSize !== sizeName && "border-[var(--color-line)] bg-white text-[var(--color-ink)] hover:border-[var(--color-ink)]"
              )}
            >
              {sizeName}
            </button>
          );
        })}
      </div>
    </div>
  );
}

function QuantitySelector({
  safeQuantity,
  maxQuantity,
  setQuantity,
}: {
  safeQuantity: number;
  maxQuantity: number;
  setQuantity: React.Dispatch<React.SetStateAction<number>>;
}) {
  return (
    <div className="mt-4">
      <div className="text-sm font-semibold text-[var(--color-ink)]">Quantity</div>
      <div className="mt-2 flex items-center gap-2">
        <button type="button" onClick={() => setQuantity((q) => Math.max(1, Math.min(Math.max(1, q), maxQuantity) - 1))} disabled={safeQuantity <= 1} aria-label="Decrease quantity" className="flex h-10 w-10 items-center justify-center rounded border border-[var(--color-line)] text-lg font-medium hover:bg-[var(--color-line)]/20 disabled:cursor-not-allowed disabled:opacity-50">-</button>
        <span className="min-w-[2rem] text-center font-sans text-sm font-medium">{safeQuantity}</span>
        <button type="button" onClick={() => setQuantity((q) => Math.min(maxQuantity, Math.min(Math.max(1, q), maxQuantity) + 1))} disabled={safeQuantity >= maxQuantity} aria-label="Increase quantity" className="flex h-10 w-10 items-center justify-center rounded border border-[var(--color-line)] text-lg font-medium hover:bg-[var(--color-line)]/20 disabled:cursor-not-allowed disabled:opacity-50">+</button>
      </div>
    </div>
  );
}

function ProductCareSections({ product }: { product: Product }) {
  return (
    <div className="mt-8 space-y-0 border-t border-[var(--color-line)]">
      <section className="border-b border-[var(--color-line)] py-4">
        <h3 className="text-sm font-semibold text-[var(--color-ink)]">Details</h3>
        <div className="mt-2 grid gap-2 text-sm text-[var(--color-muted)]">
          <div className="flex justify-between"><span>Material</span><span className="text-[var(--color-ink)]">{product.fabric}</span></div>
          <div className="flex justify-between"><span>Occasion</span><span className="text-[var(--color-ink)]">{product.occasion}</span></div>
        </div>
      </section>
      {product.collection.toLowerCase().includes("saree") && <section className="border-b border-[var(--color-line)] py-4"><h3 className="text-sm font-semibold text-[var(--color-ink)]">Care</h3><ul className="mt-2 list-disc space-y-1 pl-5 text-sm text-[var(--color-muted)]"><li><strong>Dry Clean Only:</strong> This saree should be professionally dry cleaned to preserve the fabric, color, and embroidery.</li><li><strong>Do Not Machine Wash:</strong> Washing in water may damage the fabric and delicate work.</li><li><strong>Avoid Direct Sunlight:</strong> When drying or airing the saree, keep it away from harsh sunlight to prevent color fading.</li><li><strong>Iron with Care:</strong> Use a low to medium heat setting. It is recommended to iron on the reverse side or place a protective cloth over the saree while ironing.</li><li><strong>Proper Storage:</strong> Store the saree in a cool, dry place. Preferably wrap it in a soft cotton or muslin cloth to allow the fabric to breathe and prevent moisture buildup.</li><li><strong>Refold Periodically:</strong> Refold the saree occasionally to avoid permanent creases along the same fold lines.</li></ul></section>}
      {product.collection.toLowerCase().includes("kurti") && <section className="border-b border-[var(--color-line)] py-4"><h3 className="text-sm font-semibold text-[var(--color-ink)]">Care</h3><ul className="mt-2 list-disc space-y-1 pl-5 text-sm text-[var(--color-muted)]"><li><strong>Gentle Wash Recommended:</strong> Hand wash separately in cold water using a mild detergent.</li><li><strong>Do Not Bleach:</strong> Avoid bleach or harsh chemicals as they may damage the fabric and fade colors.</li><li><strong>Machine Wash (If Required):</strong> Use a gentle cycle with cold water and place the kurti inside a laundry bag.</li><li><strong>Dry in Shade:</strong> Always dry the garment in shade to prevent color fading caused by direct sunlight.</li><li><strong>Iron with Care:</strong> Use a low to medium heat setting. Iron on the reverse side to protect prints, embroidery, or embellishments.</li><li><strong>Avoid Wringing:</strong> Do not twist or wring the fabric aggressively, as this may distort the shape of the garment.</li><li><strong>Proper Storage:</strong> Store in a cool, dry place and keep away from moisture to maintain fabric quality.</li></ul></section>}
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

export function ProductDetailView({ product, sizes: sizesFromApi = [], wished, onToggleWish, onAddToCart }: ProductDetailViewProps) {
  const [selectedImageIndex, setSelectedImageIndex] = useState(0);
  const variantStock = product.variantStock ?? [];
  const allSizeNames = sizesFromApi.length > 0 ? sizesFromApi.map((s) => s.sizeName) : FALLBACK_SIZE_NAMES;
  const sizeNames = hasSizeSelector(product) ? allSizeNames.filter((n) => n.trim().toLowerCase() !== "free size") : allSizeNames;
  const defaultSize = sizeNames.find((name) => getStockForSize(variantStock, name) > 0) ?? sizeNames[0] ?? null;
  const [selectedSize, setSelectedSize] = useState<string | null>(defaultSize);
  const [quantity, setQuantity] = useState(1);

  const maxQuantity = selectedSize != null ? getStockForSize(variantStock, selectedSize) : (variantStock[0]?.quantity ?? 999);
  const safeQuantity = maxQuantity < 1 ? 1 : Math.min(Math.max(1, quantity), maxQuantity);

  const images = useMemo(() => {
    if (product.images?.length) return product.images;
    const list = [product.image];
    if (product.hoverImage && product.hoverImage !== product.image) list.push(product.hoverImage);
    return list.filter(Boolean).length ? list : [product.image || ""];
  }, [product]);

  const mainImage = images[selectedImageIndex] || product.image;

  return (
    <div className="grid min-w-0 gap-8 md:grid-cols-2 md:items-start">
      <ProductGallery images={images} mainImage={mainImage} productName={product.imageAlt || product.name} selectedImageIndex={selectedImageIndex} setSelectedImageIndex={setSelectedImageIndex} />

      <div className="min-w-0 flex flex-col overflow-visible p-4 md:p-6 md:pl-6 md:pr-8">
        <h1 className="font-display text-xl font-semibold tracking-tight text-[var(--color-ink)] md:text-2xl">{product.name}</h1>
        <p className="mt-4 text-sm leading-relaxed text-[var(--color-muted)]">{product.description}</p>
        <div className="mt-4 font-sans text-2xl font-semibold text-[var(--color-accent-gold)]">{product.priceFormatted ?? INR.format(product.price)}</div>

        <SizeSelector product={product} sizeNames={sizeNames} variantStock={variantStock} selectedSize={selectedSize} setSelectedSize={setSelectedSize} />
        <QuantitySelector safeQuantity={safeQuantity} maxQuantity={maxQuantity} setQuantity={setQuantity} />

        <Button onClick={() => onAddToCart(product, safeQuantity, selectedSize)} disabled={maxQuantity < 1} className="mt-6 w-full rounded-full bg-[var(--color-accent-gold)] py-6 text-base font-semibold text-[var(--color-ink)] hover:bg-[var(--color-accent-gold)]/90 disabled:cursor-not-allowed disabled:opacity-50">ADD TO BAG</Button>
        <Button variant="outline" onClick={() => onToggleWish(product)} className="mt-4 w-full rounded-full border-[var(--color-line)]">{wished ? "Wishlisted" : "Add to wishlist"}</Button>

        <ProductCareSections product={product} />
      </div>
    </div>
  );
}
