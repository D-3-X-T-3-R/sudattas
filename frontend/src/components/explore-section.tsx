"use client";

import { ProductCard } from "@/components/product-card";
import type { Product } from "@/lib/schemas";
import { COLLECTIONS } from "@/lib/constants";
import { Section } from "@/components/ui/section";
import { SectionHeading, Kicker } from "@/components/ui/typography";
import { ScrollReveal } from "@/components/scroll-reveal";

export interface ExploreSectionProps {
  filtered: Product[];
  collection: string;
  occasion: string;
  sort: string;
  setCollection: (c: string) => void;
  setOccasion: (o: string) => void;
  setSort: (s: string) => void;
  occasions: string[];
  collections: string[];
  moods?: { moodId: string; moodName: string }[];
  shopMoodId?: string | null;
  onMoodChange?: (moodId: string | null) => void;
  wishlist: Record<string, boolean>;
  onToggleWish: (p: Product) => void;
  onAddToCart: (p: Product) => void;
  onQuickView: (p: Product) => void;
}

export function ExploreSection({
  filtered,
  collection,
  occasion,
  sort,
  setCollection,
  setOccasion,
  setSort,
  occasions,
  collections,
  moods = [],
  shopMoodId = null,
  onMoodChange,
  wishlist,
  onToggleWish,
  onAddToCart,
  onQuickView,
}: ExploreSectionProps) {
  const collectionList = collections.length > 0 ? collections : COLLECTIONS.map((c) => c.key);
  return (
    <Section id="explore">
      <ScrollReveal>
        <div className="flex flex-col gap-6 border-y border-[var(--color-line)] py-10">
          <div className="flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
            <SectionHeading size="lg" className="font-sans uppercase tracking-[0.18em] text-[var(--color-accent-gold)]">
              Explore
            </SectionHeading>

            <div className="flex flex-wrap gap-3 sm:items-center">
              <div className="flex items-center gap-2">
                <Kicker className="text-[var(--color-muted)]">Collection</Kicker>
                <select
                  value={collection}
                  onChange={(e) => setCollection(e.target.value)}
                  className="rounded-full border border-[var(--color-line)] bg-white/70 px-4 py-2 text-sm outline-none focus:bg-white"
                >
                  <option value="All">All</option>
                  {collectionList.filter((c) => c !== "All").map((c) => (
                    <option key={c} value={c}>{c}</option>
                  ))}
                </select>
              </div>
              <div className="flex items-center gap-2">
                <Kicker className="text-[var(--color-muted)]">Occasion</Kicker>
                <select
                  value={occasion}
                  onChange={(e) => setOccasion(e.target.value)}
                  className="rounded-full border border-[var(--color-line)] bg-white/70 px-4 py-2 text-sm outline-none focus:bg-white"
                >
                  {occasions.map((o) => (
                    <option key={o} value={o}>{o}</option>
                  ))}
                </select>
              </div>
              {moods.length > 0 && onMoodChange && (
                <div className="flex items-center gap-2">
                  <Kicker className="text-[var(--color-muted)]">Mood</Kicker>
                  <select
                    value={shopMoodId ?? ""}
                    onChange={(e) => onMoodChange(e.target.value === "" ? null : e.target.value)}
                    className="max-w-[10rem] rounded-full border border-[var(--color-line)] bg-white/70 px-4 py-2 text-sm outline-none focus:bg-white sm:max-w-none"
                  >
                    <option value="">All moods</option>
                    {moods.map((m) => (
                      <option key={m.moodId} value={m.moodId}>{m.moodName}</option>
                    ))}
                  </select>
                </div>
              )}
              <div className="flex items-center gap-2">
                <Kicker className="text-[var(--color-muted)]">Sort</Kicker>
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
        <div className="mt-14 grid grid-cols-1 gap-8 sm:grid-cols-2 lg:grid-cols-4">
          {filtered.map((p, i) => (
            <ScrollReveal key={p.id} delay={i * 0.05}>
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
    </Section>
  );
}
