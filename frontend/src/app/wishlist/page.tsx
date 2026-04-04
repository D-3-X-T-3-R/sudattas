"use client";

import { useEffect, useMemo, useState } from "react";
import { SiteHeader } from "@/components/site-header";
import type { CatalogSize } from "@/components/wishlist-grid";
import { WishlistGrid } from "@/components/wishlist-grid";
import { useStorefront } from "@/context/storefront-context";
import { ensureGuestSession, getGuestSessionId } from "@/lib/session";
import type { Product } from "@/lib/schemas";

export default function WishlistPage() {
  const { wishlist, toggleWish, addToCart, wishCount } = useStorefront();
  const [catalogProducts, setCatalogProducts] = useState<Product[]>([]);
  const [catalogSizes, setCatalogSizes] = useState<CatalogSize[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
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
        if (!cancelled) setCatalogProducts(list);
        if (sizesRes) {
          const sd = (await sizesRes.json()) as { sizes?: CatalogSize[] };
          if (!cancelled) setCatalogSizes(sd.sizes ?? []);
        } else if (!cancelled) {
          setCatalogSizes([]);
        }
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

      <div className="mx-auto w-full max-w-[2000px] px-4 pt-8 pb-32 md:pb-12">
        <div className="mb-6 md:mb-8">
          <h1
            id="wishlist-page-title"
            className="font-display text-lg font-medium uppercase tracking-[0.18em] text-[var(--color-accent-gold)]"
          >
            My Wishlist
          </h1>
          <p className="mt-1 text-sm text-[var(--color-muted)]">({titleCount})</p>
        </div>

        <div>
          {loading ? (
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 sm:gap-5 lg:grid-cols-3 lg:gap-6 xl:grid-cols-4">
              {Array.from({ length: 4 }).map((_, i) => (
                <div
                  key={i}
                  className="animate-pulse overflow-hidden rounded-xl border border-[var(--color-line)] bg-[var(--color-warm-white)] shadow-[0_8px_28px_-6px_rgba(26,24,20,0.08)]"
                >
                  <div className="aspect-[3/4] bg-[var(--color-line)]/70" />
                  <div className="space-y-2 p-4 sm:p-5">
                    <div className="h-5 w-3/4 rounded bg-[var(--color-line)]/90" />
                    <div className="h-3 w-1/2 rounded bg-[var(--color-line)]/70" />
                    <div className="mt-3 h-14 rounded-lg bg-[var(--color-line)]/50" />
                  </div>
                  <div className="h-12 border-t border-[var(--color-line)] bg-[var(--color-line)]/25" />
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
      </div>
    </div>
  );
}
