"use client";

import { useState } from "react";
import { motion, useReducedMotion } from "framer-motion";
import Image from "next/image";
import { Heart } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Rating } from "@/components/rating";
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

  return (
    <motion.div
      whileHover={reduceMotion ? undefined : { y: -4 }}
      transition={{ duration: 0.28, ease: "easeOut" }}
      className={cn("group", featured && "flex h-full flex-col")}
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
    >
      <div className={cn(
        "relative overflow-hidden rounded-sm bg-white shadow-[0_1px_3px_rgba(26,24,20,0.06)] transition-shadow duration-300 group-hover:shadow-[0_8px_24px_rgba(26,24,20,0.08)]",
        featured && "flex flex-1 flex-col"
      )}>
        <div className={cn(
          "w-full relative",
          featured ? "min-h-[280px] flex-1 basis-0" : "aspect-[3/4]"
        )}>
          <Image
            src={img}
            alt={product.imageAlt || product.name}
            fill
            className="object-cover transition duration-700 ease-out group-hover:scale-[1.03]"
            sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 25vw"
          />
        </div>
        <Button
          variant="outline"
          size="icon"
          onClick={() => onToggleWish(product)}
          className="absolute right-3 top-3 h-10 w-10 rounded-full border-[var(--color-line)] bg-white/90 backdrop-blur hover:bg-white"
          aria-label={wished ? "Remove from wishlist" : "Add to wishlist"}
        >
          <Heart
            className={cn("h-5 w-5", wished && "fill-[var(--color-ink)]")}
          />
        </Button>

        <div className="pointer-events-none absolute inset-x-0 bottom-0 h-28 bg-gradient-to-t from-black/40 to-transparent opacity-0 transition duration-300 group-hover:opacity-100" />
        <div className="absolute inset-x-3 bottom-3 flex gap-2 opacity-0 transition duration-300 group-hover:opacity-100">
          <Button
            variant="outline"
            size="sm"
            onClick={() => onQuickView(product)}
            className="pointer-events-auto flex-1 rounded-full border-white/60 bg-white/95 backdrop-blur hover:bg-white"
          >
            Quick view
          </Button>
          <Button
            size="sm"
            onClick={() => onAddToCart(product)}
            className="pointer-events-auto rounded-full bg-[var(--color-ink)] hover:bg-[var(--color-ink)]/90"
          >
            Add
          </Button>
        </div>
      </div>

      <div className={cn("mt-5", featured && "mt-6 flex-none")}>
        <div className="flex items-center gap-2">
          <span className="h-px w-4 bg-[var(--color-accent-gold)]" />
          <span className="text-[11px] tracking-[0.2em] text-[var(--color-muted)]">
            {product.collection.toUpperCase()}
          </span>
        </div>
        <div className={cn(
          "mt-2 line-clamp-2 font-display font-medium tracking-tight text-[var(--color-ink)]",
          featured ? "text-lg md:text-xl" : "text-base"
        )}>
          {product.name}
        </div>
        <div className="mt-3 flex items-center justify-between">
          <div className={cn("font-semibold text-[var(--color-ink)]", featured && "text-base")}>
            {INR.format(product.price)}
          </div>
          <Rating value={product.rating} />
        </div>
      </div>
    </motion.div>
  );
}
