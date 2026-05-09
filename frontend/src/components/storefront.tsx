"use client";

import { useEffect } from "react";
import dynamic from "next/dynamic";
import { usePathname, useRouter } from "next/navigation";
import { useReducedMotion } from "framer-motion";
import { useStorefront } from "@/context/storefront-context";
import { useStorefrontCatalog } from "@/domains/storefront/hooks/use-storefront-catalog";
import { useStorefrontNavigationEffects } from "@/domains/storefront/hooks/use-storefront-navigation-effects";
import { useActiveSection } from "@/hooks/use-active-section";
import { ensureGuestSession } from "@/lib/session";
import { consumePendingHomeCollection, goTo, PENDING_HOME_COLLECTION_EVENT } from "@/hooks/use-scroll-to";
import { HeroSection } from "@/components/hero-section";
import { BlockPrintStorySection } from "@/components/block-print-story-section";
import { CollectionsSection } from "@/components/collections-section";
import { CategoriesSection } from "@/components/categories-section";
import { ShopSection } from "@/components/shop-section";
import { ExploreSection } from "@/components/explore-section";
import { MobileBottomBar } from "@/components/mobile-bottom-bar";
import { Section } from "@/components/ui/section";
import { TrustStrip } from "@/components/trust-strip";
import { useToast } from "@/components/ui/toast";
import { Spinner } from "@/components/ui/loading";

const EditorialBlock = dynamic(() => import("@/components/editorial-block").then((m) => m.EditorialBlock), {
  loading: () => (
    <Section>
      <div className="flex justify-center py-16">
        <Spinner />
      </div>
    </Section>
  ),
});

const StorySection = dynamic(() => import("@/components/story-section").then((m) => m.StorySection), {
  loading: () => (
    <Section id="story">
      <div className="flex justify-center py-12">
        <Spinner />
      </div>
    </Section>
  ),
});

const Footer = dynamic(() => import("@/components/footer").then((m) => m.Footer), {
  loading: () => (
    <footer className="border-t border-[var(--color-line)] py-10">
      <div className="mx-auto max-w-[var(--container-max)] px-[var(--gutter-mobile)] md:px-[var(--gutter-tablet)]">
        <div className="h-16 animate-pulse rounded bg-[var(--color-line)]/40" />
      </div>
    </footer>
  ),
});

export function Storefront() {
  const pathname = usePathname();
  const router = useRouter();
  const reduceMotion = !!useReducedMotion();
  const { showToast } = useToast();
  const { wishlist, toggleWish, cartCount, wishCount } = useStorefront();

  const {
    collection,
    setCollection,
    occasion,
    setOccasion,
    sort,
    setSort,
    products,
    productsError,
    productsBannerDismissed,
    setProductsBannerDismissed,
    categories,
    moods,
    shopMoodId,
    loadingProducts,
    filtered,
    collectionOptions,
    occasionOptions,
    applyShopMoodFilter,
  } = useStorefrontCatalog({ showToast });

  const activeSection = useActiveSection(["top", "collections", "shop", "story"]);
  useStorefrontNavigationEffects({ pathname, reduceMotion, loadingProducts });

  useEffect(() => {
    void ensureGuestSession();
  }, []);

  useEffect(() => {
    const applyPendingCollection = () => {
      const pendingCollection = consumePendingHomeCollection();
      if (pendingCollection) setCollection(pendingCollection);
    };

    applyPendingCollection();

    const onPendingCollection = (event: Event) => {
      if (event instanceof CustomEvent && typeof event.detail === "string") {
        consumePendingHomeCollection();
        setCollection(event.detail);
        return;
      }
      applyPendingCollection();
    };

    window.addEventListener(PENDING_HOME_COLLECTION_EVENT, onPendingCollection);
    return () => window.removeEventListener(PENDING_HOME_COLLECTION_EVENT, onPendingCollection);
  }, [setCollection]);

  const goToProduct = (id: string) => router.push(`/product/${id}`);
  const goToWithMotion = (id: string, instant?: boolean) => goTo(id, instant ?? reduceMotion);

  return (
    <div id="top" className="min-h-screen bg-[var(--background)] text-[var(--foreground)]">
      <HeroSection />
      <Section compact className="pt-5">
        <TrustStrip />
      </Section>

      {productsError && !productsBannerDismissed && (
        <div className="border-b border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900">
          <div className="mx-auto flex max-w-[var(--container-max)] items-start justify-between gap-3 px-[var(--gutter-mobile)] md:px-[var(--gutter-tablet)]">
            <p className="min-w-0">
              <strong>Catalog temporarily unavailable.</strong> We could not load products right now. {productsError}
            </p>
            <button
              type="button"
              onClick={() => setProductsBannerDismissed(true)}
              className="shrink-0 rounded p-1 hover:bg-amber-200/50"
              aria-label="Dismiss"
            >
              x
            </button>
          </div>
        </div>
      )}

      <CollectionsSection
        setCollection={setCollection}
        moods={moods}
        onPickMood={(mood) => {
          void applyShopMoodFilter(mood.moodId);
        }}
        reduceMotion={reduceMotion}
      />

      <CategoriesSection
        categories={categories.filter((category) => collectionOptions.includes(category.name))}
        onPickCategory={(name) => setCollection(name)}
        reduceMotion={reduceMotion}
      />

      {loadingProducts ? (
        <>
          <Section id="shop">
            <div className="flex justify-center py-16">
              <Spinner />
            </div>
          </Section>
          <Section id="explore">
            <div className="flex justify-center py-16">
              <Spinner />
            </div>
          </Section>
        </>
      ) : (
        <>
          <ShopSection
            products={products}
            wishlist={wishlist}
            onToggleWish={toggleWish}
            onQuickView={(product) => goToProduct(product.id)}
            onViewAll={() => {
              setSort("Latest");
              goTo("explore", reduceMotion);
            }}
          />

          <ExploreSection
            filtered={filtered}
            collection={collection}
            occasion={occasion}
            sort={sort}
            setCollection={setCollection}
            setOccasion={setOccasion}
            setSort={setSort}
            occasions={occasionOptions}
            collections={collectionOptions}
            moods={moods}
            shopMoodId={shopMoodId}
            onMoodChange={(id) => {
              void applyShopMoodFilter(id);
            }}
            wishlist={wishlist}
            onToggleWish={toggleWish}
            onQuickView={(product) => goToProduct(product.id)}
          />
        </>
      )}

      <EditorialBlock />
      <BlockPrintStorySection />
      <StorySection />
      <Footer goTo={goToWithMotion} />

      <MobileBottomBar
        activeSection={activeSection}
        wishCount={wishCount}
        cartCount={cartCount}
        onCartOpen={() => router.push("/bag")}
        reduceMotion={reduceMotion}
      />

      <div className="h-16 md:hidden" />
    </div>
  );
}
