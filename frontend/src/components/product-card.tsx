"use client";

import Image from "next/image";
import { Heart } from "lucide-react";
import { Button } from "@/components/ui/button";
import { INR } from "@/lib/constants";
import type { Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";

const PLACEHOLDER_IMAGE =
  "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='400' height='500' viewBox='0 0 400 500'%3E%3Crect fill='%23f0ede8' width='400' height='500'/%3E%3Ctext fill='%23999' font-family='sans-serif' font-size='14' x='50%25' y='50%25' dominant-baseline='middle' text-anchor='middle'%3ENo image%3C/text%3E%3C/svg%3E";

/** True if URL is from a host not in Next.js remotePatterns (e.g. your R2/CDN). Use unoptimized so the image still loads. */
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
  onAddToCart: (p: Product) => void;
  onQuickView: (p: Product) => void;
  featured?: boolean;
}

export function ProductCard({
  product,
  wished,
  onToggleWish,
  onAddToCart,
  onQuickView,
  featured = false,
}: ProductCardProps) {
  return (
    <div className={cn("group", featured && "flex h-full flex-col")}>
      <div
        className={cn(
          "relative overflow-hidden rounded-sm bg-[var(--background)]",
          featured && "flex flex-1 flex-col"
        )}
      >
        <button
          type="button"
          onClick={() => onQuickView(product)}
          className={cn(
            "relative w-full cursor-pointer text-left",
            featured ? "min-h-[280px] flex-1 basis-0 aspect-[4/5]" : "aspect-[4/5]"
          )}
          aria-label={`Quick view ${product.name}`}
        >
          <Image
            src={product.image || PLACEHOLDER_IMAGE}
            alt={product.imageAlt || product.name}
            fill
            className="object-cover"
            sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 33vw"
            unoptimized={isExternalProductImage(product.image)}
          />
        </button>

        {/* Hover-only: wishlist */}
        <Button
          variant="outline"
          size="icon"
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            onToggleWish(product);
          }}
          className="absolute right-3 top-3 z-10 h-10 w-10 rounded-full border-[var(--color-line)] bg-white/90 backdrop-blur opacity-0 transition-opacity duration-300 group-hover:opacity-100 md:opacity-0 md:group-hover:opacity-100"
          aria-label={wished ? "Remove from wishlist" : "Add to wishlist"}
        >
          <Heart
            className={cn("h-5 w-5", wished && "fill-[var(--color-ink)]")}
          />
        </Button>
      </div>

      <div className={cn("mt-4", featured && "mt-6 flex-none")}>
        <div className="flex items-center gap-2">
          <span className="text-xs uppercase tracking-[0.18em] text-[var(--color-muted)]">
            {product.collection}
          </span>
        </div>
        <div
          className={cn(
            "mt-1.5 line-clamp-2 font-display font-medium tracking-tight text-[var(--color-ink)]",
            featured ? "text-lg md:text-xl" : "text-base"
          )}
        >
          {product.name}
        </div>
        <div className="mt-2 font-semibold text-[var(--color-ink)]">
          {product.priceFormatted ?? INR.format(product.price)}
        </div>
      </div>
    </div>
  );
}
