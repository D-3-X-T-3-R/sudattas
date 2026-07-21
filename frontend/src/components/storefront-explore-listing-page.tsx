"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { useStorefront } from "@/context/storefront-context";
import { useStorefrontCatalog } from "@/domains/storefront/hooks/use-storefront-catalog";
import { ensureGuestSession } from "@/lib/session";
import { ExploreSection } from "@/components/explore-section";
import { Footer } from "@/components/footer";
import { Section } from "@/components/ui/section";
import { Spinner } from "@/components/ui/loading";
import { useToast } from "@/components/ui/toast";

export function StorefrontExploreListingPage() {
  const router = useRouter();
  const { wishlist, toggleWish } = useStorefront();
  const { showToast } = useToast();
  const catalog = useStorefrontCatalog({ showToast });

  useEffect(() => {
    void ensureGuestSession();
  }, []);

  return (
    <div className="min-h-screen bg-[var(--background)] text-[var(--foreground)]">
      {catalog.productsError && !catalog.productsBannerDismissed ? (
        <div className="border-b border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900">
          <div className="mx-auto flex max-w-[var(--container-max)] items-start justify-between gap-3 px-[var(--gutter-mobile)] md:px-[var(--gutter-tablet)]">
            <p className="min-w-0">
              <strong>Catalog temporarily unavailable.</strong> We could not load products right
              now. {catalog.productsError}
            </p>
            <button
              type="button"
              onClick={() => catalog.setProductsBannerDismissed(true)}
              className="shrink-0 rounded p-1 hover:bg-amber-200/50"
              aria-label="Dismiss"
            >
              x
            </button>
          </div>
        </div>
      ) : null}

      {catalog.loadingProducts ? (
        <Section id="explore">
          <div className="flex justify-center py-16">
            <Spinner />
          </div>
        </Section>
      ) : (
        <ExploreSection
          catalog={catalog}
          wishlist={wishlist}
          onToggleWish={toggleWish}
          onQuickView={(product) => router.push(`/product/${product.id}`)}
        />
      )}

      <Footer />
    </div>
  );
}
