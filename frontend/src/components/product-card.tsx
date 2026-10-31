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
  const hasHoverImage = !!product.hoverImage && product.hoverImage !== product.image;

  return (
    <article className={cn("group flex flex-col", featured && "h-full")}>
      <div className="relative overflow-hidden rounded-[var(--radius-md)] border border-[var(--color-line)] bg-[var(--color-surface-soft)]">
        <button
          type="button"
          onClick={() => onQuickView(product)}
          className="relative block w-full text-left aspect-[2/3]"
          aria-label={`View ${product.name}`}
        >
          <Image
            src={product.image || PLACEHOLDER_IMAGE}
            alt={product.imageAlt || product.name}
            fill
            className={cn(
              "object-cover transition-[transform,opacity] duration-500 ease-out",
              hasHoverImage ? "group-hover:opacity-0" : "group-hover:scale-[1.04]"
            )}
            sizes="(max-width: 768px) 50vw, 25vw"
            unoptimized={isExternalProductImage(product.image)}
          />
          {hasHoverImage ? (
            <Image
              src={product.hoverImage!}
              alt={product.imageAlt || product.name}
              fill
              className="object-cover opacity-0 transition-opacity duration-500 ease-out group-hover:opacity-100"
              sizes="(max-width: 768px) 50vw, 25vw"
              unoptimized={isExternalProductImage(product.hoverImage)}
            />
          ) : null}
        </button>

        <span className="pointer-events-none absolute left-3 top-3 rounded-full border border-[var(--color-line)] bg-[var(--color-surface)]/90 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.2em] text-[var(--color-green)] backdrop-blur-sm">
          New
        </span>

        <Button
          variant="outline"
          size="icon"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            onToggleWish(product);
          }}
          className="absolute right-3 top-3 h-9 w-9 rounded-full border-white/50 bg-[var(--color-surface)]/85 backdrop-blur-sm hover:border-[var(--color-gold)]"
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

      <div className="mt-3 flex items-start justify-between gap-3 sm:mt-4">
        <h3 className="line-clamp-2 break-words font-display text-[1.05rem] leading-snug text-[var(--color-ink)] sm:text-xl">
          {product.name}
        </h3>
        <p className="whitespace-nowrap pt-0.5 font-sans text-sm font-semibold text-[var(--color-ink)] sm:text-base">
          {product.priceFormatted ?? INR.format(product.price)}
        </p>
      </div>
    </article>
  );
}
