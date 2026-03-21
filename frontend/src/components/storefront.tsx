"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";
import { useReducedMotion } from "framer-motion";
import { ensureGuestSession, getGuestSessionId, clearGuestSession } from "@/lib/session";
import { PRODUCTS_SEED } from "@/lib/seed-data";
import type { Product, CartLine } from "@/lib/schemas";
import { useStorefront } from "@/context/storefront-context";

type ProductsResponse = { products: Product[]; error: string | null };

async function fetchStorefrontProducts(
  sessionId: string | null,
  moodId?: string | null
): Promise<{
  products: Product[];
  error: string | null;
}> {
  try {
    const headers: Record<string, string> = {};
    if (sessionId) headers["X-Session-Id"] = sessionId;
    const qs =
      moodId && moodId.trim() !== ""
        ? `?moodId=${encodeURIComponent(moodId.trim())}`
        : "";
    const res = await fetch(`/api/products${qs}`, { headers });
    const data = (await res.json()) as ProductsResponse | Product[];
    if (Array.isArray(data)) {
      return { products: data, error: null };
    }
    return {
      products: data.products ?? [],
      error: data.error ?? (res.ok ? null : "Request failed"),
    };
  } catch {
    return { products: [], error: "Network error" };
  }
}

type StorefrontFiltersResponse = {
  categories: { categoryId: string; name: string }[];
  occasions: { occasionId: string; occasionName: string }[];
  moods: { moodId: string; moodName: string; thumbnailUrl?: string }[];
  error: string | null;
};

async function fetchStorefrontFilters(sessionId: string | null): Promise<StorefrontFiltersResponse> {
  try {
    const headers: Record<string, string> = {};
    if (sessionId) headers["X-Session-Id"] = sessionId;
    console.log("[DEBUG] fetchStorefrontFilters: fetching with sessionId =", sessionId);
    const res = await fetch("/api/storefront-filters", { headers });
    const raw = await res.text();
    console.log("[DEBUG] fetchStorefrontFilters: HTTP", res.status, "| raw body:", raw);
    const data = JSON.parse(raw) as StorefrontFiltersResponse;
    return {
      categories: data.categories ?? [],
      occasions: data.occasions ?? [],
      moods: data.moods ?? [],
      error: data.error ?? null,
    };
  } catch (e) {
    console.error("[DEBUG] fetchStorefrontFilters: caught", e);
    return { categories: [], occasions: [], moods: [], error: "Network error" };
  }
}
import { useActiveSection } from "@/hooks/use-active-section";
import { useLockBodyScroll } from "@/hooks/use-lock-body-scroll";
import { useRazorpayTest } from "@/hooks/use-razorpay-test";
import { goTo } from "@/hooks/use-scroll-to";
import { Header } from "@/components/header";
import { HeroSection } from "@/components/hero-section";
import { CollectionsSection } from "@/components/collections-section";
import { EditorialBlock } from "@/components/editorial-block";
import { ShopSection } from "@/components/shop-section";
import { ExploreSection } from "@/components/explore-section";
import { StorySection } from "@/components/story-section";
import { Footer } from "@/components/footer";
import { MenuDrawer } from "@/components/menu-drawer";
import { WishlistDrawer } from "@/components/wishlist-drawer";
import { MobileBottomBar } from "@/components/mobile-bottom-bar";
import { useToast } from "@/components/ui/toast";
import { Spinner } from "@/components/ui/loading";

export function Storefront() {
  const router = useRouter();
  const reduceMotion = useReducedMotion();
  const { showToast } = useToast();
  const {
    wishlist,
    toggleWish,
    cart,
    addToCart,
    decCart,
    incCart,
    wishOpen,
    setWishOpen,
    cartLines,
    cartCount,
    cartSubtotal,
    wishCount,
  } = useStorefront();
  const [menuOpen, setMenuOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [collection, setCollection] = useState("All");
  const [occasion, setOccasion] = useState("All");
  const [sort, setSort] = useState("Featured");
  const [products, setProducts] = useState<Product[]>(PRODUCTS_SEED);
  const [productsError, setProductsError] = useState<string | null>(null);
  const [productsBannerDismissed, setProductsBannerDismissed] = useState(false);
  const [categories, setCategories] = useState<{ categoryId: string; name: string }[]>([]);
  const [occasions, setOccasions] = useState<{ occasionId: string; occasionName: string }[]>([]);
  const [moods, setMoods] = useState<{ moodId: string; moodName: string; thumbnailUrl?: string }[]>([]);
  /** When set, product list is loaded from GraphQL searchProduct with this moodId. */
  const [shopMoodId, setShopMoodId] = useState<string | null>(null);
  const [loadingProducts, setLoadingProducts] = useState(false);

  const { paymentMessage, paymentLoading, runTest } = useRazorpayTest();
  const activeSection = useActiveSection(["top", "collections", "shop", "story"]);
  useLockBodyScroll(menuOpen || wishOpen);

  useEffect(() => {
    ensureGuestSession();
  }, []);

  useEffect(() => {
    // Guest session → Next /api/products + /api/storefront-filters → GraphQL (see route files).
    async function loadProducts() {
      setLoadingProducts(true);
      console.log("[DEBUG] loadProducts: start");
      await ensureGuestSession();
      let sessionId = getGuestSessionId();
      console.log("[DEBUG] loadProducts: sessionId after ensureGuestSession =", sessionId);

      async function loadCatalog(sid: string | null, mood: string | null) {
        console.log("[DEBUG] loadCatalog: calling with sid =", sid);
        const [pr, fr] = await Promise.all([
          fetchStorefrontProducts(sid, mood),
          fetchStorefrontFilters(sid),
        ]);
        console.log("[DEBUG] loadCatalog: products.length =", pr.products.length, "products.error =", pr.error);
        console.log("[DEBUG] loadCatalog: moods.length =", fr.moods.length, "filters.error =", fr.error, "moods =", fr.moods);
        return { pr, fr };
      }

      let { pr, fr } = await loadCatalog(sessionId, null);

      const looksLikeBadSession = (msg: string | null) =>
        !!msg &&
        (msg.includes("Session invalid") ||
          msg.includes("Session not found") ||
          msg.includes("expired") ||
          msg.includes("Unauthorized"));

      const hasError = !!(pr.error || fr.error);
      console.log("[DEBUG] hasError =", hasError, "| pr.error =", pr.error, "| fr.error =", fr.error);
      if (hasError && (looksLikeBadSession(pr.error) || looksLikeBadSession(fr.error) || !sessionId)) {
        console.log("[DEBUG] BAD SESSION detected — clearing and retrying");
        clearGuestSession();
        await ensureGuestSession();
        sessionId = getGuestSessionId();
        console.log("[DEBUG] retry: new sessionId =", sessionId);
        if (sessionId) {
          ({ pr, fr } = await loadCatalog(sessionId, null));
        } else {
          console.error("[DEBUG] retry: ensureGuestSession still returned null — backend unreachable?");
        }
      }

      const { products: list, error } = pr;
      console.log("[DEBUG] final products.length =", list.length, "| final moods =", fr.moods);
      if (list.length > 0) {
        setProducts(list);
        setProductsError(null);
      } else if (error) {
        setProductsError(error ?? "No products from backend");
        showToast({
          title: "Catalog",
          description:
            "Having trouble connecting. Your bag is saved on this device.",
        });
      } else {
        setProducts([]);
        setProductsError(null);
      }

      setCategories(fr.categories);
      setOccasions(fr.occasions);
      setMoods(fr.moods);
      console.log("[DEBUG] setMoods called with", fr.moods.length, "moods:", fr.moods);
      if (process.env.NODE_ENV === "development" && fr.error) {
        console.warn("[storefront-filters]", fr.error);
      }
      setLoadingProducts(false);
    }
    loadProducts();
  }, []);

  /** Re-fetch catalog with GraphQL searchProduct + moodId (or full list when null). */
  const applyShopMoodFilter = useCallback(
    async (nextMoodId: string | null) => {
      setShopMoodId(nextMoodId);
      setLoadingProducts(true);
      await ensureGuestSession();
      let sid = getGuestSessionId();
      let pr = await fetchStorefrontProducts(sid, nextMoodId);
      const badSession = (msg: string | null) =>
        !!msg &&
        (msg.includes("Session invalid") ||
          msg.includes("Session not found") ||
          msg.includes("expired"));
      if (sid && badSession(pr.error)) {
        clearGuestSession();
        await ensureGuestSession();
        sid = getGuestSessionId();
        pr = await fetchStorefrontProducts(sid, nextMoodId);
      }
      if (pr.products.length > 0) {
        setProducts(pr.products);
        setProductsError(null);
      } else if (pr.error) {
        setProductsError(pr.error);
        showToast({
          title: "Catalog",
          description:
            "Having trouble connecting. Your bag is saved on this device.",
        });
      } else {
        setProducts([]);
        setProductsError(null);
      }
      setLoadingProducts(false);
    },
    [showToast]
  );

  const occasionOptions = useMemo(() => {
    const fromProducts = new Set(products.map((p) => p.occasion).filter(Boolean));
    if (occasions.length > 0) {
      const matched = occasions
        .map((o) => o.occasionName)
        .filter((name) => fromProducts.has(name));
      return ["All", ...matched];
    }
    return ["All", ...Array.from(fromProducts)];
  }, [occasions, products]);

  const collectionOptions = useMemo(() => {
    const fromProducts = new Set(products.map((p) => p.collection).filter(Boolean));
    if (categories.length > 0) {
      const matched = categories
        .map((c) => c.name)
        .filter((name) => fromProducts.has(name));
      return ["All", ...matched];
    }
    return ["All", ...Array.from(fromProducts)];
  }, [categories, products]);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    let xs = products.filter((p) => {
      const matchesQuery =
        !q ||
        [p.name, p.collection, p.fabric, p.occasion]
          .join(" ")
          .toLowerCase()
          .includes(q);
      const matchesCollection =
        collection === "All" || p.collection === collection;
      const matchesOccasion = occasion === "All" || p.occasion === occasion;
      return matchesQuery && matchesCollection && matchesOccasion;
    });
    if (sort === "Price: Low") xs = [...xs].sort((a, b) => a.price - b.price);
    if (sort === "Price: High") xs = [...xs].sort((a, b) => b.price - a.price);
    if (sort === "Rating") xs = [...xs].sort((a, b) => b.rating - a.rating);
    return xs;
  }, [products, query, collection, occasion, sort]);

  const wishedProducts = useMemo(
    () => products.filter((p) => wishlist[p.id]),
    [products, wishlist]
  );

  const goToProduct = (p: Product) => router.push(`/product/${p.id}`);
  const goToWithMotion = (id: string, instant?: boolean) =>
    goTo(id, instant ?? !!reduceMotion);

  return (
    <div id="top" className="min-h-screen bg-[var(--background)] text-[var(--foreground)]">
      <Header
        query={query}
        setQuery={setQuery}
        cartCount={cartCount}
        wishCount={wishCount}
        setMenuOpen={setMenuOpen}
        setCartOpen={() => {}}
        setWishOpen={setWishOpen}
        goTo={goToWithMotion}
      />

      <HeroSection />

      {productsError && !productsBannerDismissed && (
        <div className="border-b border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900">
          <div className="mx-auto flex max-w-[2000px] items-start justify-between gap-3">
            <p className="min-w-0">
              <strong>Sample products only.</strong> Catalog could not be loaded ({productsError}). To show your DB products: in <code className="rounded bg-amber-100 px-1">.env.local</code> set <code className="rounded bg-amber-100 px-1">NEXT_PUBLIC_GRAPHQL_URL</code> (e.g. http://localhost:8080/v2) and <code className="rounded bg-amber-100 px-1">NEXT_PUBLIC_ADMIN_API_KEY</code> to match the backend <code className="rounded bg-amber-100 px-1">ADMIN_API_KEY</code>, then restart the dev server.
            </p>
            <button
              type="button"
              onClick={() => setProductsBannerDismissed(true)}
              className="shrink-0 rounded p-1 hover:bg-amber-200/50"
              aria-label="Dismiss"
            >
              ×
            </button>
          </div>
        </div>
      )}

      <CollectionsSection
        setCollection={setCollection}
        moods={moods}
        onPickMood={(m) => {
          void applyShopMoodFilter(m.moodId);
        }}
        reduceMotion={!!reduceMotion}
      />

      {loadingProducts ? (
        <div className="flex justify-center py-16">
          <Spinner />
        </div>
      ) : (
        <>
          <ShopSection
            products={products}
            wishlist={wishlist}
            onToggleWish={toggleWish}
            onAddToCart={addToCart}
            onQuickView={goToProduct}
            reduceMotion={!!reduceMotion}
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
            onMoodChange={(id) => void applyShopMoodFilter(id)}
            wishlist={wishlist}
            onToggleWish={toggleWish}
            onAddToCart={addToCart}
            onQuickView={goToProduct}
          />
        </>
      )}

      <EditorialBlock />

      <StorySection />

      <Footer goTo={goToWithMotion} />

      <MenuDrawer
        open={menuOpen}
        onClose={() => setMenuOpen(false)}
        setCollection={setCollection}
        reduceMotion={!!reduceMotion}
      />

      <WishlistDrawer
        open={wishOpen}
        onClose={() => setWishOpen(false)}
        wishCount={wishCount}
        wishedProducts={wishedProducts}
        onQuickView={goToProduct}
        onAddToCart={addToCart}
        onToggleWish={toggleWish}
      />

      <MobileBottomBar
        activeSection={activeSection}
        wishCount={wishCount}
        cartCount={cartCount}
        onWishOpen={() => setWishOpen(true)}
        onCartOpen={() => router.push("/bag")}
        reduceMotion={!!reduceMotion}
      />


      <div className="h-16 md:hidden" />
    </div>
  );
}
