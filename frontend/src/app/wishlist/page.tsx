"use client";

import { useEffect, useMemo, useState } from "react";
import { SiteHeader } from "@/components/site-header";
import type { CatalogSize } from "@/components/wishlist-grid";
import { WishlistGrid } from "@/components/wishlist-grid";
import { PageShell, SectionHeader } from "@/components/ui/page-shell";
import { useStorefront } from "@/context/storefront-context";
import { ensureGuestSession, getGuestSessionId } from "@/lib/session";
import type { Product } from "@/lib/schemas";

const WISHLIST_CATALOG_CACHE_KEY = "sudattas_wishlist_catalog_v1";
const WISHLIST_CATALOG_CACHE_TTL_MS = 5 * 60 * 1000;

type WishlistCatalogCache = {
  savedAt: number;
  products: Product[];
  sizes: CatalogSize[];
};

function readWishlistCatalogCache(): WishlistCatalogCache | null {
  if (typeof window === "undefined") return null;
  try {
    const raw = window.sessionStorage.getItem(WISHLIST_CATALOG_CACHE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as WishlistCatalogCache;
    if (!parsed || typeof parsed.savedAt !== "number") return null;
    if (Date.now() - parsed.savedAt > WISHLIST_CATALOG_CACHE_TTL_MS) return null;
    return {
      savedAt: parsed.savedAt,
      products: Array.isArray(parsed.products) ? parsed.products : [],
      sizes: Array.isArray(parsed.sizes) ? parsed.sizes : [],
    };
  } catch {
    return null;
  }
}

function writeWishlistCatalogCache(products: Product[], sizes: CatalogSize[]) {
  if (typeof window === "undefined") return;
  try {
    const payload: WishlistCatalogCache = { savedAt: Date.now(), products, sizes };
    window.sessionStorage.setItem(WISHLIST_CATALOG_CACHE_KEY, JSON.stringify(payload));
  } catch {
    // Ignore cache write failures (private mode/quota).
  }
}

export default function WishlistPage() {
  const { wishlist, toggleWish, addToCart, wishCount } = useStorefront();
  const [catalogProducts, setCatalogProducts] = useState<Product[]>([]);
  const [catalogSizes, setCatalogSizes] = useState<CatalogSize[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    const cached = readWishlistCatalogCache();
    if (cached) {
      setCatalogProducts(cached.products);
      setCatalogSizes(cached.sizes);
      setLoading(false);
    }
    void (async () => {
      await ensureGuestSession();
      const sid = getGuestSessionId();
      const headers: Record<string, string> = {};
      if (sid) headers["X-Session-Id"] = sid;
      try {
        const [productsRes, sizesRes] = await Promise.all([
          fetch("/api/products", { headers }),
          sid
            ? fetch("/api/sizes", { headers: { "x-session-id": sid } })
            : Promise.resolve(null as Response | null),
        ]);
        const data = (await productsRes.json()) as { products?: Product[] } | Product[];
        const list = Array.isArray(data) ? data : data.products ?? [];
        let nextSizes: CatalogSize[] = [];
        if (sizesRes) {
          const sd = (await sizesRes.json()) as { sizes?: CatalogSize[] };
          nextSizes = sd.sizes ?? [];
        }
        writeWishlistCatalogCache(list, nextSizes);
        if (!cancelled) setCatalogProducts(list);
        if (!cancelled) setCatalogSizes(nextSizes);
      } catch {
        if (!cancelled) {
          setCatalogProducts([]);
          setCatalogSizes([]);
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const wishedProducts = useMemo(
    () => catalogProducts.filter((p) => wishlist[p.id]),
    [catalogProducts, wishlist]
  );

  const titleN = !loading ? wishedProducts.length : wishCount;
  const titleCount = titleN === 1 ? "1 item" : `${titleN} items`;

  return (
    <div className="min-h-screen w-full min-w-0 bg-[var(--background)] text-[var(--foreground)]">
      <SiteHeader />
      <PageShell containerClassName="pt-8 pb-20 md:pt-10 md:pb-14">
        <SectionHeader
          label="Wishlist"
          title="Saved Favourites"
          description={`Review your shortlisted styles before adding them to your bag. (${titleCount})`}
        />

        <div className="mt-8">
          {loading ? (
            <div className="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4 md:gap-5">
              {Array.from({ length: 4 }).map((_, i) => (
                <div
                  key={i}
                  className="animate-pulse overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] shadow-[var(--shadow-subtle)]"
                >
                  <div className="aspect-[3/4] bg-[var(--color-line)]/60" />
                  <div className="space-y-2 p-4">
                    <div className="h-5 w-3/4 rounded bg-[var(--color-line)]/80" />
                    <div className="h-4 w-1/2 rounded bg-[var(--color-line)]/60" />
                    <div className="h-10 rounded bg-[var(--color-line)]/40" />
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <WishlistGrid
              products={wishedProducts}
              catalogSizes={catalogSizes}
              onRemove={toggleWish}
              onAddToBag={(p, sizeName) => void addToCart(p, 1, sizeName)}
            />
          )}
        </div>
      </PageShell>
    </div>
  );
}
