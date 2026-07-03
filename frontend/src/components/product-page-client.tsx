"use client";

import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ProductDetailView } from "@/components/product-detail-view";
import { useStorefront } from "@/context/storefront-context";

interface ProductPageClientProps {
  product: import("@/lib/schemas").Product;
  sizes: { sizeId: string; sizeName: string }[];
  relatedProducts: import("@/lib/schemas").Product[];
}

export function ProductPageClient({ product, sizes, relatedProducts }: ProductPageClientProps) {
  const { wishlist, toggleWish, addToCart } = useStorefront();

  return (
    <div className="min-h-screen bg-[var(--background)] text-[var(--color-ink)]">
      <div className="mx-auto min-w-0 max-w-[var(--container-max)] px-[var(--gutter-mobile)] py-6 md:px-[var(--gutter-tablet)] md:py-8">
        <Button
          variant="ghost"
          size="sm"
          className="mb-4 -ml-1 text-[var(--color-muted)] hover:text-[var(--color-green)]"
          asChild
        >
          <Link href="/" className="flex items-center gap-2">
            <ArrowLeft className="h-4 w-4" />
            Back to shop
          </Link>
        </Button>

        <ProductDetailView
          key={product.id}
          product={product}
          sizes={sizes}
          wished={!!wishlist[product.id]}
          onToggleWish={toggleWish}
          onAddToCart={addToCart}
          relatedProducts={relatedProducts}
          relatedWishlist={wishlist}
        />
      </div>
    </div>
  );
}
