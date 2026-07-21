"use client";

import { useEffect } from "react";
import dynamic from "next/dynamic";
import { usePathname, useRouter } from "next/navigation";
import { useReducedMotion } from "framer-motion";
import { useStorefront } from "@/context/storefront-context";
import { useStorefrontCatalog } from "@/domains/storefront/hooks/use-storefront-catalog";
import { useStorefrontNavigationEffects } from "@/domains/storefront/hooks/use-storefront-navigation-effects";
import { ensureGuestSession } from "@/lib/session";
import { consumePendingHomeCollection, goTo, PENDING_HOME_COLLECTION_EVENT } from "@/hooks/use-scroll-to";
import { HeroSection } from "@/components/hero-section";
import { CollectionsSection } from "@/components/collections-section";
import { CategoriesSection } from "@/components/categories-section";
import { ShopSection } from "@/components/shop-section";
import { ExploreSection } from "@/components/explore-section";
import { FullBleedVideoSection } from "@/components/full-bleed-video-section";
import { JournalTeaserSection } from "@/components/journal-teaser-section";
import { Section } from "@/components/ui/section";
import { TrustStrip } from "@/components/trust-strip";
import { useToast } from "@/components/ui/toast";
import { Spinner } from "@/components/ui/loading";

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
    <footer className="bg-deep-feature py-14 md:py-20">
      <div className="mx-auto max-w-[var(--container-max)] px-[var(--gutter-mobile)] md:px-[var(--gutter-tablet)]">
        <div className="h-16 animate-pulse rounded bg-white/10" />
      </div>
    </footer>
  ),
});

export function Storefront() {
  const pathname = usePathname();
  const router = useRouter();
  const reduceMotion = !!useReducedMotion();
  const { showToast } = useToast();
  const { wishlist, toggleWish } = useStorefront();

  const catalog = useStorefrontCatalog({ showToast });
  const {
    setCollection,
    products,
    productsError,
    productsBannerDismissed,
    setProductsBannerDismissed,
    categories,
    moods,
    loadingProducts,
    collectionOptions,
    applyShopMoodFilter,
  } = catalog;

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
    <div id="top" className="min-h-screen text-[var(--foreground)]">
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

      <CategoriesSection
        categories={categories.filter((category) => collectionOptions.includes(category.name))}
        onPickCategory={(name) => setCollection(name)}
        reduceMotion={reduceMotion}
      />

      <FullBleedVideoSection
        id="block-print-story"
        src="/videos/block_print_story_new.mp4"
        mobileSrc="/videos/block_print_story_original.MP4"
        kicker="The Block Print Story"
        heading="Print, stitch, rhythm, and patience."
        body="Every print begins with touch, pressure, rhythm, and care — chosen for textile character, quiet irregularities, and boutique finish."
        align="right"
      />

      {loadingProducts ? (
        <Section id="shop" className="relative z-0 bg-[var(--background)]">
          <div className="flex justify-center py-16">
            <Spinner />
          </div>
        </Section>
      ) : (
        <ShopSection
          products={products}
          wishlist={wishlist}
          onToggleWish={toggleWish}
          onQuickView={(product) => goToProduct(product.id)}
          onViewAll={() => {
            catalog.setSort("Featured");
            goTo("explore", reduceMotion);
          }}
        />
      )}

      <FullBleedVideoSection
        src="/videos/Woman_posing_in_saree_202606140847.mp4"
        mobilePlaylist={["/videos/IMG_6700.MP4", "/videos/IMG_6701.MP4", "/videos/IMG_6704.MP4"]}
        kicker="Sudatta's Atelier"
        heading="Made to be worn, kept, and passed on."
        body="From loom to wardrobe — every piece is finished by hand, in small batches, with time-honoured techniques."
      />

      <CollectionsSection
        setCollection={setCollection}
        moods={moods}
        onPickMood={(mood) => {
          void applyShopMoodFilter(mood.moodId);
        }}
        reduceMotion={reduceMotion}
      />

      {loadingProducts ? (
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
          onQuickView={(product) => goToProduct(product.id)}
        />
      )}

      <JournalTeaserSection />
      <StorySection />
      <Footer goTo={goToWithMotion} />

    </div>
  );
}
