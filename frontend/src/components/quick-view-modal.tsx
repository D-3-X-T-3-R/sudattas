"use client";

import { useMemo, useState } from "react";
import Image from "next/image";
import Link from "next/link";
import { ChevronUp, Truck } from "lucide-react";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { INR } from "@/lib/constants";
import type { Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";

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

function Accordion({ title, children, defaultOpen = false }: { title: string; children: React.ReactNode; defaultOpen?: boolean }) {
  return (
    <details className="group border-b border-[var(--color-line)]" open={defaultOpen}>
      <summary className="flex cursor-pointer list-none items-center justify-between py-4 text-sm font-semibold text-[var(--color-ink)]">
        {title}
        <ChevronUp className="h-4 w-4 shrink-0 transition-transform group-open:rotate-180" />
      </summary>
      <div className="pb-4 text-sm text-[var(--color-muted)]">{children}</div>
    </details>
  );
}

function QuickViewGallery({ product, images, selectedImageIndex, setSelectedImageIndex }: { product: Product; images: string[]; selectedImageIndex: number; setSelectedImageIndex: (idx: number) => void; }) {
  const mainImage = images[selectedImageIndex] || product.image;
  return (
    <div className="flex gap-3 bg-white p-4 md:p-6">
      <div className="flex flex-col gap-2">
        {images.map((src, i) => (
          <button key={i} type="button" onClick={() => setSelectedImageIndex(i)} aria-label={`View image ${i + 1} of ${images.length}`} className={cn("relative h-14 w-14 shrink-0 overflow-hidden rounded border-2 transition-colors md:h-16 md:w-16", selectedImageIndex === i ? "border-[var(--color-ink)]" : "border-transparent hover:border-[var(--color-line)]")}>
            <Image src={src} alt={`${product.name} view ${i + 1}`} fill className="object-cover" sizes="64px" unoptimized={isExternalImage(src)} />
          </button>
        ))}
      </div>
      <div className="relative min-h-0 flex-1 aspect-[3/4]">
        <Image src={mainImage} alt={product.imageAlt || product.name} fill className="object-cover" sizes="(max-width: 768px) 100vw, 50vw" unoptimized={isExternalImage(mainImage)} />
      </div>
    </div>
  );
}

function QuickViewActions({
  product,
  wished,
  onToggleWish,
  onAddToCart,
}: {
  product: Product;
  wished: boolean;
  onToggleWish: (p: Product) => void;
  onAddToCart: (p: Product, qty?: number, sizeName?: string | null) => void;
}) {
  const variantStock = product.variantStock ?? [];
  const sizeNames = hasSizeSelector(product)
    ? variantStock
        .map((variant) => variant.sizeName)
        .filter((sizeName) => sizeName.trim().toLowerCase() !== "free size")
    : [];
  const defaultSize = sizeNames.find((sizeName) => getStockForSize(variantStock, sizeName) > 0) ?? sizeNames[0] ?? null;
  const [selectedSize, setSelectedSize] = useState<string | null>(defaultSize);
  const [quantity, setQuantity] = useState(1);
  const [descriptionExpanded, setDescriptionExpanded] = useState(false);

  const descShort = product.description.slice(0, 160);
  const hasMoreDesc = product.description.length > 160;
  const maxQuantity = selectedSize != null
    ? getStockForSize(variantStock, selectedSize)
    : (variantStock[0]?.quantity ?? 999);
  const safeQuantity = maxQuantity < 1 ? 1 : Math.min(Math.max(1, quantity), maxQuantity);

  return (
    <div className="flex flex-col p-4 md:p-6 md:pr-8">
      <div className="text-[11px] font-semibold tracking-[0.24em] text-[var(--color-muted)]">{product.collection.toUpperCase()}</div>
      <h2 className="mt-2 font-display text-xl font-semibold tracking-tight text-[var(--color-ink)] md:text-2xl">{product.name}</h2>
      <p className="mt-4 text-sm leading-relaxed text-[var(--color-muted)]">
        {descriptionExpanded || !hasMoreDesc ? product.description : `${descShort}${hasMoreDesc ? "..." : ""}`}
        {hasMoreDesc && <button type="button" onClick={() => setDescriptionExpanded(true)} className="ml-1 font-semibold text-[var(--color-ink)] underline">Read more</button>}
      </p>
      <div className="mt-4 font-sans text-lg font-semibold text-[var(--color-accent-gold)]">MRP {product.priceFormatted ?? INR.format(product.price)}</div>

      <div className="mt-6">
        {hasSizeSelector(product) ? (
          <>
            <div className="text-sm font-semibold text-[var(--color-ink)]">
              Size{" "}
              <Link href="/size-fit-guide" className="font-normal text-[var(--color-muted)] underline">
                Size &amp; Fit Guide
              </Link>
            </div>
            <div className="mt-2 flex flex-wrap gap-2">
              {sizeNames.map((sizeName) => {
                const availableQty = getStockForSize(variantStock, sizeName);
                const outOfStock = availableQty <= 0;
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
          </>
        ) : (
          <div className="rounded-sm border border-[var(--color-line)] bg-[var(--color-line)]/10 p-3 text-sm text-[var(--color-muted)]">
            <p>This style does not use standard size variants. Fit can vary by fabric, cut, and drape.</p>
            <Link href="/size-fit-guide" className="mt-2 inline-flex font-semibold text-[var(--color-ink)] underline">
              View Size &amp; Fit Guide
            </Link>
          </div>
        )}
      </div>

      <div className="mt-4">
        <div className="text-sm font-semibold text-[var(--color-ink)]">Quantity</div>
        <div className="mt-2 flex items-center gap-2">
          <button type="button" onClick={() => setQuantity((q) => Math.max(1, q - 1))} disabled={safeQuantity <= 1} aria-label="Decrease quantity" className="flex h-10 w-10 items-center justify-center rounded border border-[var(--color-line)] text-lg font-medium hover:bg-[var(--color-line)]/20 disabled:cursor-not-allowed disabled:opacity-50">-</button>
          <span className="min-w-[2rem] text-center font-sans text-sm font-medium">{safeQuantity}</span>
          <button type="button" onClick={() => setQuantity((q) => Math.min(maxQuantity, q + 1))} disabled={safeQuantity >= maxQuantity} aria-label="Increase quantity" className="flex h-10 w-10 items-center justify-center rounded border border-[var(--color-line)] text-lg font-medium hover:bg-[var(--color-line)]/20 disabled:cursor-not-allowed disabled:opacity-50">+</button>
        </div>
      </div>

      <Button onClick={() => onAddToCart(product, safeQuantity, selectedSize)} disabled={maxQuantity < 1} className="mt-6 w-full rounded-full bg-[var(--color-accent-gold)] py-6 text-base font-semibold text-[var(--color-ink)] hover:bg-[var(--color-accent-gold)]/90 disabled:cursor-not-allowed disabled:opacity-50">ADD TO BAG</Button>
      <div className="mt-4 flex items-center gap-2 text-sm text-[var(--color-muted)]"><Truck className="h-4 w-4 shrink-0" /><span>Delivery timeline is confirmed during checkout for your pincode and selected items.</span></div>
      <Button variant="outline" onClick={() => onToggleWish(product)} className="mt-4 w-full rounded-full border-[var(--color-line)]">{wished ? "Wishlisted" : "Add to wishlist"}</Button>

      <div className="mt-8">
        <Accordion title="Product Details" defaultOpen>
          <div className="grid gap-2 text-sm">
            <div className="flex justify-between"><span>Material</span><span className="text-[var(--color-ink)]">{product.fabric}</span></div>
            <div className="flex justify-between"><span>Occasion</span><span className="text-[var(--color-ink)]">{product.occasion}</span></div>
          </div>
        </Accordion>
        <Accordion title="Delivery & Payment"><p>Shipping, COD/prepaid eligibility, and delivery timelines are confirmed during checkout for your order and location.</p></Accordion>
        <Accordion title="Returns & Exchange"><p>See our Returns &amp; Exchanges and Cancellation Policy pages for current eligibility and process details.</p></Accordion>
        <Accordion title="Care Instruction"><ul className="list-disc space-y-1 pl-5"><li>Hand wash in cold water</li><li>Dry clean recommended for Sarees</li><li>Wash inside out</li><li>Iron on low heat</li></ul></Accordion>
        <Accordion title="Customer Care"><p className="font-semibold">Have a question? We can help.</p><p className="mt-2">Mon-Sat 10:00 AM to 6:00 PM IST</p><p className="mt-1">Sujana, Survey no 14/7, Hartola Hartas, Bavdhan Pune 411011, Maharashtra</p><p className="mt-1">+91 9411 XXXXXX</p></Accordion>
      </div>
    </div>
  );
}

export interface QuickViewModalProps {
  product: Product | null;
  open: boolean;
  onClose: () => void;
  wished: boolean;
  onToggleWish: (p: Product) => void;
  onAddToCart: (p: Product, qty?: number, sizeName?: string | null) => void;
}

export function QuickViewModal({ product, open, onClose, wished, onToggleWish, onAddToCart }: QuickViewModalProps) {
  const [selectedImageIndex, setSelectedImageIndex] = useState(0);

  const images = useMemo(() => {
    if (!product) return [];
    const list = [product.image];
    if (product.hoverImage && product.hoverImage !== product.image) list.push(product.hoverImage);
    return list.filter(Boolean).length ? list : [product.image || ""];
  }, [product]);

  if (!product) return null;

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent title="" showClose onPointerDownOutside={onClose} onEscapeKeyDown={onClose} className="max-w-5xl overflow-hidden p-0" contentClassName="p-0">
        <div className="max-h-[90vh] overflow-y-auto">
          <div className="grid gap-8 md:grid-cols-2">
            <QuickViewGallery product={product} images={images} selectedImageIndex={selectedImageIndex} setSelectedImageIndex={setSelectedImageIndex} />
            <QuickViewActions key={product.id} product={product} wished={wished} onToggleWish={onToggleWish} onAddToCart={onAddToCart} />
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
