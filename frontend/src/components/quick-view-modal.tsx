"use client";

import { useState, useMemo } from "react";
import Image from "next/image";
import { ChevronUp, Truck } from "lucide-react";
import {
  Dialog,
  DialogContent,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { INR } from "@/lib/constants";
import type { Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";

const SIZES = ["XS", "S", "M", "L", "XL", "XXL"];

function isExternalImage(src: string | undefined): boolean {
  if (!src || src.startsWith("/") || src.startsWith("data:")) return false;
  try {
    const host = new URL(src).hostname;
    return host !== "images.unsplash.com";
  } catch {
    return false;
  }
}

export interface QuickViewModalProps {
  product: Product | null;
  open: boolean;
  onClose: () => void;
  wished: boolean;
  onToggleWish: (p: Product) => void;
  onAddToCart: (p: Product, qty?: number, sizeName?: string | null) => void;
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

export function QuickViewModal({
  product,
  open,
  onClose,
  wished,
  onToggleWish,
  onAddToCart,
}: QuickViewModalProps) {
  const [selectedImageIndex, setSelectedImageIndex] = useState(0);
  const [selectedSize, setSelectedSize] = useState<string | null>("M");
  const [quantity, setQuantity] = useState(1);
  const [descriptionExpanded, setDescriptionExpanded] = useState(false);

  const images = useMemo(() => {
    if (!product) return [];
    const list = [product.image];
    if (product.hoverImage && product.hoverImage !== product.image) {
      list.push(product.hoverImage);
    }
    return list.filter(Boolean).length ? list : [product.image || ""];
  }, [product]);

  if (!product) return null;

  const mainImage = images[selectedImageIndex] || product.image;
  const descShort = product.description.slice(0, 160);
  const hasMoreDesc = product.description.length > 160;

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent
        title=""
        showClose
        onPointerDownOutside={onClose}
        onEscapeKeyDown={onClose}
        className="max-w-5xl overflow-hidden p-0"
        contentClassName="p-0"
      >
        <div className="max-h-[90vh] overflow-y-auto">
          <div className="grid gap-8 md:grid-cols-2">
            {/* Left: image gallery */}
            <div className="flex gap-3 bg-white p-4 md:p-6">
              <div className="flex flex-col gap-2">
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
              <div className="relative min-h-0 flex-1 aspect-[3/4]">
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
            <div className="flex flex-col p-4 md:p-6 md:pr-8">
              <div className="text-[11px] font-semibold tracking-[0.24em] text-[var(--color-muted)]">
                {product.collection.toUpperCase()}
              </div>
              <h2 className="mt-2 font-display text-xl font-semibold tracking-tight text-[var(--color-ink)] md:text-2xl">
                {product.name}
              </h2>

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

              <div className="mt-4 font-sans text-lg font-semibold text-[var(--color-accent-gold)]">
                MRP {product.priceFormatted ?? INR.format(product.price)}
              </div>

              {/* Size */}
              <div className="mt-6">
                <div className="text-sm font-semibold text-[var(--color-ink)]">
                  Size{" "}
                  <button
                    type="button"
                    className="font-normal text-[var(--color-muted)] underline"
                  >
                    Size Chart
                  </button>
                </div>
                <div className="mt-2 flex flex-wrap gap-2">
                  {SIZES.map((size) => (
                    <button
                      key={size}
                      type="button"
                      onClick={() => setSelectedSize(size)}
                      className={cn(
                        "min-w-[2.5rem] rounded border px-3 py-2 text-sm font-medium transition-colors",
                        selectedSize === size
                          ? "border-[var(--color-ink)] bg-[var(--color-ink)] text-white"
                          : "border-[var(--color-line)] bg-white text-[var(--color-ink)] hover:border-[var(--color-ink)]"
                      )}
                    >
                      {size}
                    </button>
                  ))}
                </div>
              </div>

              {/* Quantity */}
              <div className="mt-4">
                <div className="text-sm font-semibold text-[var(--color-ink)]">
                  Quantity
                </div>
                <div className="mt-2 flex items-center gap-2">
                  <button
                    type="button"
                    onClick={() => setQuantity((q) => Math.max(1, q - 1))}
                    className="flex h-10 w-10 items-center justify-center rounded border border-[var(--color-line)] text-lg font-medium hover:bg-[var(--color-line)]/20"
                  >
                    −
                  </button>
                  <span className="min-w-[2rem] text-center font-sans text-sm font-medium">
                    {quantity}
                  </span>
                  <button
                    type="button"
                    onClick={() => setQuantity((q) => q + 1)}
                    className="flex h-10 w-10 items-center justify-center rounded border border-[var(--color-line)] text-lg font-medium hover:bg-[var(--color-line)]/20"
                  >
                    +
                  </button>
                </div>
              </div>

              <Button
                onClick={() => onAddToCart(product, quantity, selectedSize)}
                className="mt-6 w-full rounded-full bg-[var(--color-accent-gold)] py-6 text-base font-semibold text-[var(--color-ink)] hover:bg-[var(--color-accent-gold)]/90"
              >
                ADD TO BAG
              </Button>

              <div className="mt-4 flex items-center gap-2 text-sm text-[var(--color-muted)]">
                <Truck className="h-4 w-4 shrink-0" />
                <span>Estimated Delivery by: Wednesday, Mar 16</span>
              </div>

              <Button
                variant="outline"
                onClick={() => onToggleWish(product)}
                className="mt-4 w-full rounded-full border-[var(--color-line)]"
              >
                {wished ? "Wishlisted" : "Add to wishlist"}
              </Button>

              {/* Accordions */}
              <div className="mt-8">
                <Accordion title="Product Details" defaultOpen>
                  <div className="grid gap-2 text-sm">
                    <div className="flex justify-between">
                      <span>Material</span>
                      <span className="text-[var(--color-ink)]">
                        {product.fabric}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span>Occasion</span>
                      <span className="text-[var(--color-ink)]">
                        {product.occasion}
                      </span>
                    </div>
                  </div>
                </Accordion>
                <Accordion title="Delivery & Payment">
                  <p>
                    Order is made to order. Dispatch in 7–10 working days;
                    transit 4–5 days. Cash on Delivery available.
                  </p>
                </Accordion>
                <Accordion title="Returns & Exchange">
                  <ul className="list-disc space-y-1 pl-5">
                    <li>3 days easy return & exchange</li>
                    <li>Free pickup for returns & exchanges</li>
                    <li>Returns/exchanges allowed only for unused products with tags</li>
                  </ul>
                </Accordion>
                <Accordion title="Care Instruction">
                  <ul className="list-disc space-y-1 pl-5">
                    <li>Hand wash in cold water</li>
                    <li>Dry clean recommended for Sarees</li>
                    <li>Wash inside out</li>
                    <li>Iron on low heat</li>
                  </ul>
                </Accordion>
                <Accordion title="Customer Care">
                  <p className="font-semibold">Have a question? We can help.</p>
                  <p className="mt-2">Mon–Sat 10:00 AM to 6:00 PM IST</p>
                  <p className="mt-1">
                    Sujana, Survey no 14/7, Hartola Hartas, Bavdhan Pune 411011,
                    Maharashtra
                  </p>
                  <p className="mt-1">+91 9411 XXXXXX</p>
                </Accordion>
              </div>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
