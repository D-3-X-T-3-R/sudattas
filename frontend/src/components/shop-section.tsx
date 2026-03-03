"use client";

import { ProductCard } from "@/components/product-card";
import type { Product } from "@/lib/schemas";
import { COLLECTIONS } from "@/lib/constants";
import { ScrollReveal } from "@/components/scroll-reveal";

export interface ShopSectionProps {
  filtered: Product[];
  collection: string;
  occasion: string;
  sort: string;
  setCollection: (c: string) => void;
  setOccasion: (o: string) => void;
  setSort: (s: string) => void;
  occasions: string[];
  wishlist: Record<string, boolean>;
  onToggleWish: (p: Product) => void;
  onAddToCart: (p: Product) => void;
  onQuickView: (p: Product) => void;
}

export function ShopSection({
  filtered,
  collection,
  occasion,
  sort,
  setCollection,
  setOccasion,
  setSort,
  occasions,
  wishlist,
  onToggleWish,
  onAddToCart,
  onQuickView,
}: ShopSectionProps) {
  const featured = filtered[0];
  const rest = filtered.slice(1);

  return (
    <section id="shop" className="mx-auto max-w-7xl px-4 pb-24">
      <ScrollReveal>
        <div
          className="flex flex-col gap-6 border-y border-[var(--color-line)] py-10"
        >
          <div className="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
            <div>
              <div className="flex items-center gap-2">
                <span className="h-px w-6 bg-[var(--color-accent-gold)]" />
                <span className="text-[11px] font-semibold tracking-[0.26em] text-[var(--color-muted)]">
                  SHOP
                </span>
              </div>
              <h3 className="mt-3 font-display text-3xl font-medium tracking-tight text-[var(--color-ink)] sm:text-4xl">
                New arrivals
              </h3>
              <p className="mt-2 text-sm text-[var(--color-muted)]">
                {filtered.length} item{filtered.length === 1 ? "" : "s"} •
                Collection: {collection} • Occasion: {occasion}
              </p>
            </div>

            <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
              <div className="flex items-center gap-2">
                <span className="text-xs font-semibold tracking-[0.18em] text-[var(--color-muted)]">
                  COLLECTION
                </span>
                <select
                  value={collection}
                  onChange={(e) => setCollection(e.target.value)}
                  className="rounded-full border border-[var(--color-line)] bg-white/70 px-4 py-2 text-sm outline-none focus:bg-white"
                >
                  <option value="All">All</option>
                  {COLLECTIONS.map((c) => (
                    <option key={c.key} value={c.key}>
                      {c.key}
                    </option>
                  ))}
                </select>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-xs font-semibold tracking-[0.18em] text-[var(--color-muted)]">
                  OCCASION
                </span>
                <select
                  value={occasion}
                  onChange={(e) => setOccasion(e.target.value)}
                  className="rounded-full border border-[var(--color-line)] bg-white/70 px-4 py-2 text-sm outline-none focus:bg-white"
                >
                  {occasions.map((o) => (
                    <option key={o} value={o}>
                      {o}
                    </option>
                  ))}
                </select>
              </div>
              <div className="flex items-center gap-2">
                <span className="text-xs font-semibold tracking-[0.18em] text-[var(--color-muted)]">
                  SORT
                </span>
                <select
                  value={sort}
                  onChange={(e) => setSort(e.target.value)}
                  className="rounded-full border border-[var(--color-line)] bg-white/70 px-4 py-2 text-sm outline-none focus:bg-white"
                >
                  <option>Featured</option>
                  <option>Price: Low</option>
                  <option>Price: High</option>
                  <option>Rating</option>
                </select>
              </div>
            </div>
          </div>

          {filtered.length === 0 && (
            <div className="rounded-2xl bg-white p-6 text-sm text-[var(--color-muted)]">
              No products match your filters.
            </div>
          )}
        </div>
      </ScrollReveal>

      {filtered.length > 0 && (
        <div className="mt-12 grid gap-6 sm:grid-cols-2 lg:grid-cols-4 lg:grid-rows-2">
          {featured && (
            <ScrollReveal className="lg:col-span-2 lg:row-span-2" delay={0.05}>
              <ProductCard
                key={featured.id}
                product={featured}
                wished={!!wishlist[featured.id]}
                onToggleWish={onToggleWish}
                onAddToCart={onAddToCart}
                onQuickView={onQuickView}
                featured
              />
            </ScrollReveal>
          )}
          {rest.map((p, i) => (
            <ScrollReveal key={p.id} delay={0.1 + i * 0.05}>
              <ProductCard
                product={p}
                wished={!!wishlist[p.id]}
                onToggleWish={onToggleWish}
                onAddToCart={onAddToCart}
                onQuickView={onQuickView}
              />
            </ScrollReveal>
          ))}
        </div>
      )}
    </section>
  );
}
