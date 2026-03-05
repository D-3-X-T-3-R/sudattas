"use client";

import { useState } from "react";
import { motion, useReducedMotion } from "framer-motion";
import Image from "next/image";
import { Heart } from "lucide-react";
import { Button } from "@/components/ui/button";
import { INR } from "@/lib/constants";
import type { Product } from "@/lib/schemas";
import { cn } from "@/lib/utils";

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
  const [hover, setHover] = useState(false);
  const reduceMotion = useReducedMotion();
  const img =
    hover && product.hoverImage ? product.hoverImage : product.image;

  const cardHoverY = reduceMotion ? undefined : 3;
  const duration = 0.4;

  return (
    <motion.div
      whileHover={cardHoverY !== undefined ? { y: -cardHoverY } : undefined}
      transition={{ duration, ease: "easeOut" }}
      className={cn("group", featured && "flex h-full flex-col")}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
    >
      <div
        className={cn(
          "relative overflow-hidden rounded-sm bg-white shadow-[0_1px_3px_rgba(26,24,20,0.06)] transition-shadow duration-300 group-hover:shadow-[0_6px_20px_rgba(26,24,20,0.08)]",
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
            src={img}
            alt={product.imageAlt || product.name}
            fill
            className="object-cover transition duration-500 ease-out group-hover:scale-[1.02]"
            sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 33vw"
          />
        </button>

        {/* Hover-only: wishlist (desktop) */}
        <Button
          variant="outline"
          size="icon"
          onClick={() => onToggleWish(product)}
          className="absolute right-3 top-3 h-10 w-10 rounded-full border-[var(--color-line)] bg-white/90 backdrop-blur opacity-0 transition-opacity duration-300 group-hover:opacity-100 md:opacity-0 md:group-hover:opacity-100"
          aria-label={wished ? "Remove from wishlist" : "Add to wishlist"}
        >
          <Heart
            className={cn("h-5 w-5", wished && "fill-[var(--color-ink)]")}
          />
        </Button>

        {/* Hover-only: quick view + add (desktop); always visible on touch for accessibility, but we keep them in quick view flow on mobile */}
        <div className="pointer-events-none absolute inset-x-0 bottom-0 h-24 bg-gradient-to-t from-black/40 to-transparent opacity-0 transition duration-300 group-hover:opacity-100 md:opacity-0 md:group-hover:opacity-100" />
        <div className="absolute inset-x-3 bottom-3 flex gap-2 opacity-0 transition duration-300 group-hover:opacity-100 md:opacity-0 md:group-hover:opacity-100">
          <Button
            variant="outline"
            size="sm"
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              onQuickView(product);
            }}
            className="pointer-events-auto flex-1 rounded-full border-white/60 bg-white/95 backdrop-blur hover:bg-white"
          >
            Quick view
          </Button>
          <Button
            size="sm"
            onClick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              onAddToCart(product);
            }}
            className="pointer-events-auto rounded-full bg-[var(--color-ink)] hover:bg-[var(--color-ink)]/90"
          >
            Add
          </Button>
        </div>
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
          {INR.format(product.price)}
        </div>
      </div>
    </motion.div>
  );
}
