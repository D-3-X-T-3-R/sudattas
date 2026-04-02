"use client";

import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { SiteHeader } from "@/components/site-header";
import { ProductDetailView } from "@/components/product-detail-view";
import { useStorefront } from "@/context/storefront-context";
import type { Product } from "@/lib/schemas";

interface ProductPageClientProps {
  product: Product;
  sizes: { sizeId: string; sizeName: string }[];
}

export function ProductPageClient({ product, sizes }: ProductPageClientProps) {
  const { wishlist, toggleWish, addToCart } = useStorefront();

  return (
    <div className="min-h-screen bg-[var(--color-ivory)] text-[var(--color-ink)]">
      <SiteHeader />

      <div className="mx-auto min-w-0 max-w-[2000px] px-4 py-4">
        <Button
          variant="ghost"
          size="sm"
          className="mb-4 -ml-2 text-[var(--color-muted)] hover:text-[var(--color-ink)]"
          asChild
        >
          <Link href="/" className="flex items-center gap-2">
            <ArrowLeft className="h-4 w-4" />
            Back to shop
          </Link>
        </Button>

        <ProductDetailView
          product={product}
          sizes={sizes}
          wished={!!wishlist[product.id]}
          onToggleWish={toggleWish}
          onAddToCart={addToCart}
        />
      </div>
    </div>
  );
}
