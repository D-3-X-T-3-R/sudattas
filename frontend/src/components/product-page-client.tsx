"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ProductDetailView } from "@/components/product-detail-view";
import { useStorefront } from "@/context/storefront-context";
import { finishProductPageTransition, leaveProductPageWithTransition } from "@/lib/product-page-transition";

interface ProductPageClientProps {
  product: import("@/lib/schemas").Product;
  sizes: { sizeId: string; sizeName: string }[];
  relatedProducts: import("@/lib/schemas").Product[];
}

export function ProductPageClient({ product, sizes, relatedProducts }: ProductPageClientProps) {
  const router = useRouter();
  const { wishlist, toggleWish, addToCart } = useStorefront();
  const [leaving, setLeaving] = useState(false);

  useEffect(() => {
    void finishProductPageTransition(product.id);
  }, [product.id]);

  const goBack = () => {
    if (window.innerWidth >= 768) {
      router.back();
      return;
    }
    setLeaving(true);
    window.requestAnimationFrame(() => {
      void leaveProductPageWithTransition(product, router);
    });
  };

  return (
    <div className={`min-h-screen bg-[var(--background)] text-[var(--color-ink)] transition-opacity duration-300 ${leaving ? "opacity-0" : "animate-pdp-page-enter opacity-100"}`}>
      <div className="mx-auto min-w-0 max-w-[var(--container-max)] px-[var(--gutter-mobile)] py-6 md:px-[var(--gutter-tablet)] md:py-8">
        <Button
          variant="ghost"
          size="sm"
          className="mb-4 -ml-1 text-[var(--color-muted)] hover:text-[var(--color-green)]"
          onClick={goBack}
        >
          <ArrowLeft className="h-4 w-4" />
          Back to shop
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
