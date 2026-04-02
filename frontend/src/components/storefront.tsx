"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import Link from "next/link";
import dynamic from "next/dynamic";
import { usePathname, useRouter } from "next/navigation";
import { useReducedMotion } from "framer-motion";
import { User } from "lucide-react";
import { useSession } from "next-auth/react";
import { ensureGuestSession, getGuestSessionId, clearGuestSession } from "@/lib/session";
import { toRouteFailureUi } from "@/lib/route-state";
import { PRODUCTS_SEED } from "@/lib/seed-data";
import type { Product } from "@/lib/schemas";
import { useStorefront } from "@/context/storefront-context";
import { useStorefrontLogin } from "@/context/storefront-login-context";

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
  categories: { categoryId: string; name: string; thumbnailUrl?: string }[];
  occasions: { occasionId: string; occasionName: string }[];
  moods: { moodId: string; moodName: string; thumbnailUrl?: string }[];
  error: string | null;
};

async function fetchStorefrontFilters(sessionId: string | null): Promise<StorefrontFiltersResponse> {
  try {
    const headers: Record<string, string> = {};
    if (sessionId) headers["X-Session-Id"] = sessionId;
    const res = await fetch("/api/storefront-filters", { headers });
    const data = (await res.json()) as StorefrontFiltersResponse;
    return {
      categories: data.categories ?? [],
      occasions: data.occasions ?? [],
      moods: data.moods ?? [],
      error: data.error ?? null,
    };
  } catch {
    return { categories: [], occasions: [], moods: [], error: "Network error" };
  }
}
import { useActiveSection } from "@/hooks/use-active-section";
import { useLockBodyScroll } from "@/hooks/use-lock-body-scroll";
import { clearPendingHomeSection, goTo, peekPendingHomeSection } from "@/hooks/use-scroll-to";
import { Header } from "@/components/header";
import { HeroSection } from "@/components/hero-section";
import { CollectionsSection } from "@/components/collections-section";
import { CategoriesSection } from "@/components/categories-section";
import { ShopSection } from "@/components/shop-section";
import { ExploreSection } from "@/components/explore-section";
import { MenuDrawer } from "@/components/menu-drawer";
import { MobileBottomBar } from "@/components/mobile-bottom-bar";
import { Section } from "@/components/ui/section";
import { useToast } from "@/components/ui/toast";
import { Spinner } from "@/components/ui/loading";

const EditorialBlock = dynamic(
  () => import("@/components/editorial-block").then((m) => m.EditorialBlock),
  {
    loading: () => (
      <Section>
        <div className="flex justify-center py-16">
          <Spinner />
        </div>
      </Section>
    ),
  }
);
const StorySection = dynamic(
  () => import("@/components/story-section").then((m) => m.StorySection),
  {
    loading: () => (
      <Section id="story">
        <div className="flex justify-center py-12">
          <Spinner />
        </div>
      </Section>
    ),
  }
);
const Footer = dynamic(
  () => import("@/components/footer").then((m) => m.Footer),
  {
    loading: () => (
      <footer className="border-t border-[var(--color-line)] py-10">
        <div className="mx-auto max-w-[2000px] px-4">
          <div className="h-16 animate-pulse rounded bg-[var(--color-line)]/40" />
        </div>
      </footer>
    ),
  }
);

export function Storefront() {
  const pathname = usePathname();
  const router = useRouter();
  const { status, data: session } = useSession();
  const { openLogin } = useStorefrontLogin();
  const reduceMotion = useReducedMotion();
  const { showToast } = useToast();
  const {
    wishlist,
    toggleWish,
    cartCount,
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
  const [categories, setCategories] = useState<{ categoryId: string; name: string; thumbnailUrl?: string }[]>([]);
  const [occasions, setOccasions] = useState<{ occasionId: string; occasionName: string }[]>([]);
  const [moods, setMoods] = useState<{ moodId: string; moodName: string; thumbnailUrl?: string }[]>([]);
  /** When set, product list is loaded from GraphQL searchProduct with this moodId. */
  const [shopMoodId, setShopMoodId] = useState<string | null>(null);
  /** Start true so #shop / #explore are never missing during the first catalog fetch. */
  const [loadingProducts, setLoadingProducts] = useState(true);
  const didInitialLoadRef = useRef(false);

  const activeSection = useActiveSection(["top", "collections", "shop", "story"]);
  useLockBodyScroll(menuOpen);

  useEffect(() => {
    ensureGuestSession();
  }, [showToast]);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const prev = window.history.scrollRestoration;
    window.history.scrollRestoration = "manual";
    return () => {
      window.history.scrollRestoration = prev;
    };
  }, [showToast]);

  /**
   * Cross-route nav: sessionStorage + optional hash; scroll after target exists in DOM.
   * Moods/Collections (`#collections`, `#category-collections`) are always mounted above the
   * catalog gate. `#shop` / `#explore` only exist as placeholders while `loadingProducts` is
   * true, then swap to real sections — we must not clear pending until catalog is ready, or the
   * effect won't run again to scroll to the final layout.
   */
  useEffect(() => {
    if (pathname !== "/") return;
    const hashId = typeof window !== "undefined" ? window.location.hash.slice(1) : "";
    const { id: pendingId, fromOtherPage } = peekPendingHomeSection();
    const id = hashId || pendingId;
    if (!id) return;

    /* From other routes: top first, then same as Header on landing — `goTo(id, false)`. */
    const scrollViaTop = Boolean(pendingId) && fromOtherPage && !hashId;

    let cancelled = false;
    let attempts = 0;
    const finish = () => {
      if (!pendingId) return;
      const waitForCatalog =
        loadingProducts && (id === "shop" || id === "explore");
      if (waitForCatalog) return;
      clearPendingHomeSection();
    };
    const tryScroll = () => {
      if (cancelled) return;
      if (document.getElementById(id)) {
        if (scrollViaTop) {
          window.scrollTo({ top: 0, behavior: "auto" });
          requestAnimationFrame(() => {
            requestAnimationFrame(() => {
              if (!cancelled) {
                goTo(id, false);
                finish();
              }
            });
          });
        } else {
          goTo(id, !!reduceMotion);
          finish();
        }
        return;
      }
      attempts += 1;
      if (attempts < 120) window.setTimeout(tryScroll, 50);
    };
    tryScroll();
    return () => {
      cancelled = true;
    };
  }, [pathname, reduceMotion, loadingProducts]);

  useEffect(() => {
    const onHashChange = () => {
      if (window.location.pathname !== "/") return;
      const id = window.location.hash.slice(1);
      if (!id) return;
      goTo(id, !!reduceMotion);
    };
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, [reduceMotion]);

  useEffect(() => {
    if (didInitialLoadRef.current) return;
    didInitialLoadRef.current = true;

    // Guest session → Next /api/products + /api/storefront-filters → GraphQL (see route files).
    async function loadProducts() {
      setLoadingProducts(true);
      await ensureGuestSession();
      let sessionId = getGuestSessionId();

      async function loadCatalog(sid: string | null, mood: string | null) {
        const [pr, fr] = await Promise.all([
          fetchStorefrontProducts(sid, mood),
          fetchStorefrontFilters(sid),
        ]);
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
      if (hasError && (looksLikeBadSession(pr.error) || looksLikeBadSession(fr.error) || !sessionId)) {
        clearGuestSession();
        await ensureGuestSession();
        sessionId = getGuestSessionId();
        if (sessionId) {
          ({ pr, fr } = await loadCatalog(sessionId, null));
        }
      }

      const { products: list, error } = pr;
      if (list.length > 0) {
        setProducts(list);
        setProductsError(null);
      } else if (error) {
        setProductsError(toRouteFailureUi("public", new Error(error)).message);
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
      setLoadingProducts(false);
    }
    loadProducts();
  }, [showToast]);

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
        setProductsError(toRouteFailureUi("public", new Error(pr.error)).message);
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
    if (sort === "Latest") xs = [...xs].sort((a, b) => parseInt(b.id) - parseInt(a.id) || b.id.localeCompare(a.id));
    return xs;
  }, [products, query, collection, occasion, sort]);

  const goToProduct = (p: Product) => router.push(`/product/${p.id}`);
  const goToWithMotion = (id: string, instant?: boolean) =>
    goTo(id, instant ?? !!reduceMotion);
  const rawName = session?.user?.name?.trim() ?? "";
  const looksLikePhone = /^\+?\d{10,15}$/.test(rawName);
  const firstName = !rawName || looksLikePhone ? "Profile" : rawName.split(/\s+/)[0];

  return (
    <div id="top" className="min-h-screen bg-[var(--background)] text-[var(--foreground)]">
      <Header
        query={query}
        setQuery={setQuery}
        cartCount={cartCount}
        wishCount={wishCount}
        setMenuOpen={setMenuOpen}
        goTo={goToWithMotion}
        authEnabled
        authButtons={
          status === "authenticated" ? (
            <Link
              href="/profile"
              className="inline-flex h-10 items-center gap-2 rounded-full border border-[var(--color-line)] bg-white px-4 text-xs font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)] transition-colors hover:border-[var(--color-accent-gold)] hover:text-[var(--color-accent-gold)]"
              aria-label="Open profile"
            >
              <User size={14} />
              {firstName}
            </Link>
          ) : (
            <button
              type="button"
              onClick={() => openLogin()}
              className="inline-flex h-10 items-center gap-2 rounded-full border border-[var(--color-line)] bg-white px-4 text-xs font-semibold uppercase tracking-[0.16em] text-[var(--color-ink)] transition-colors hover:border-[var(--color-accent-gold)] hover:text-[var(--color-accent-gold)]"
              aria-label="Sign in"
            >
              <User size={14} />
              Sign In
            </button>
          )
        }
      />

      <HeroSection />

      {productsError && !productsBannerDismissed && (
        <div className="border-b border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900">
          <div className="mx-auto flex max-w-[2000px] items-start justify-between gap-3">
            <p className="min-w-0">
              <strong>Catalog temporarily unavailable.</strong> We could not load
              products right now. {productsError}
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

      <CategoriesSection
        categories={categories.filter((c) =>
          collectionOptions.includes(c.name)
        )}
        onPickCategory={(name) => setCollection(name)}
        reduceMotion={!!reduceMotion}
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
            onQuickView={goToProduct}
            onViewAll={() => {
              setSort("Latest");
              goTo("explore", !!reduceMotion);
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
            onMoodChange={(id) => void applyShopMoodFilter(id)}
            wishlist={wishlist}
            onToggleWish={toggleWish}
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

      <MobileBottomBar
        activeSection={activeSection}
        wishCount={wishCount}
        cartCount={cartCount}
        onCartOpen={() => router.push("/bag")}
        reduceMotion={!!reduceMotion}
      />


      <div className="h-16 md:hidden" />
    </div>
  );
}
