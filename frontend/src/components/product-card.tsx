"use client";

import Image from "next/image";
import { Heart } from "lucide-react";
import { Button } from "@/components/ui/button";
import { INR } from "@/lib/constants";
import type { Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";

const PLACEHOLDER_IMAGE =
  "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='400' height='500' viewBox='0 0 400 500'%3E%3Crect fill='%23f0ede8' width='400' height='500'/%3E%3Ctext fill='%23999' font-family='sans-serif' font-size='14' x='50%25' y='50%25' dominant-baseline='middle' text-anchor='middle'%3ENo image%3C/text%3E%3C/svg%3E";

function isExternalProductImage(src: string | undefined): boolean {
  if (!src || src.startsWith("/") || src.startsWith("data:")) return false;
  try {
    const host = new URL(src).hostname;
    return host !== "images.unsplash.com";
  } catch {
    return false;
  }
}

export interface ProductCardProps {
  product: Product;
  wished: boolean;
  onToggleWish: (p: Product) => void;
  onQuickView: (p: Product) => void;
  featured?: boolean;
}

export function ProductCard({
  product,
  wished,
  onToggleWish,
  onQuickView,
  featured = false,
}: ProductCardProps) {
  return (
    <article className={cn("group rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] shadow-[var(--shadow-subtle)]", featured && "h-full")}>
      <div className="relative overflow-hidden rounded-t-lg border-b border-[var(--color-line)]">
        <button
          type="button"
          onClick={() => onQuickView(product)}
          className={cn("relative block w-full text-left", featured ? "aspect-[4/5]" : "aspect-[3/4]")}
          aria-label={`Quick view ${product.name}`}
        >
          <Image
            src={product.image || PLACEHOLDER_IMAGE}
            alt={product.imageAlt || product.name}
            fill
            className="object-cover transition-transform duration-500 ease-out group-hover:scale-[1.03]"
            sizes="(max-width: 768px) 50vw, 25vw"
            unoptimized={isExternalProductImage(product.image)}
          />
        </button>

        <Button
          variant="outline"
          size="icon"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            onToggleWish(product);
          }}
          className="absolute right-2.5 top-2.5 h-8 w-8 rounded-sm border-[var(--color-line)] bg-white/95"
          aria-label={wished ? "Remove from wishlist" : "Add to wishlist"}
        >
          <Heart
            className={cn(
              "h-4 w-4",
              wished && "fill-[var(--color-gold)] text-[var(--color-gold)]"
            )}
          />
        </Button>
      </div>

      <div className="p-3 sm:p-4">
        <h3 className="line-clamp-2 font-display text-lg leading-tight text-[var(--color-ink)] sm:text-[1.35rem]">
          {product.name}
        </h3>
        <p className="mt-2 font-sans text-lg font-semibold text-[var(--color-ink)]">
          {product.priceFormatted ?? INR.format(product.price)}
        </p>
        <p className="mt-1 text-[10px] font-semibold uppercase tracking-[0.2em] text-[var(--color-gold)]">
          New
        </p>
      </div>
    </article>
  );
}
