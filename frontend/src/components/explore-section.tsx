"use client";

import { ProductCard } from "@/components/product-card";
import type { Product } from "@/lib/schemas";
import { COLLECTIONS } from "@/lib/constants";
import { Section } from "@/components/ui/section";
import { ScrollReveal } from "@/components/scroll-reveal";
import { SectionHeader, EmptyState } from "@/components/ui/page-shell";
import { cn } from "@/lib/utils";

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
  onQuickView: (p: Product) => void;
}

function FilterField({
  label,
  id,
  value,
  onChange,
  children,
  className,
}: {
  label: string;
  id: string;
  value: string;
  onChange: (value: string) => void;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <label className={cn("block", className)} htmlFor={id}>
      <span className="mb-1 block text-[10px] font-semibold uppercase tracking-[0.18em] text-[var(--color-muted)]">
        {label}
      </span>
      <select
        id={id}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="h-10 w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 text-sm text-[var(--color-ink)] outline-none focus:border-[var(--color-green)] focus:ring-2 focus:ring-[var(--color-focus)]"
      >
        {children}
      </select>
    </label>
  );
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
  onQuickView,
}: ExploreSectionProps) {
  const collectionList =
    collections.length > 0 ? collections : COLLECTIONS.map((c) => c.key);

  const filterPanel = (
    <div className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-subtle)]">
      <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[var(--color-muted)]">Filter By</p>
      <div className="mt-4 space-y-3">
        <FilterField
          label="Category"
          id="explore-collection"
          value={collection}
          onChange={setCollection}
        >
          <option value="All">All</option>
          {collectionList
            .filter((c) => c !== "All")
            .map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
        </FilterField>

        <FilterField
          label="Occasion"
          id="explore-occasion"
          value={occasion}
          onChange={setOccasion}
        >
          {occasions.map((o) => (
            <option key={o} value={o}>
              {o}
            </option>
          ))}
        </FilterField>

        {moods.length > 0 && onMoodChange ? (
          <FilterField
            label="Mood"
            id="explore-mood"
            value={shopMoodId ?? ""}
            onChange={(value) => onMoodChange(value === "" ? null : value)}
          >
            <option value="">All Moods</option>
            {moods.map((m) => (
              <option key={m.moodId} value={m.moodId}>
                {m.moodName}
              </option>
            ))}
          </FilterField>
        ) : null}
      </div>
    </div>
  );

  return (
    <Section id="explore">
      <ScrollReveal>
        <SectionHeader
          label="Catalog"
          title="New Arrivals"
          description="Thoughtfully designed pieces for every celebration and everyday elegance."
          action={
            <div className="flex items-center gap-3">
              <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-muted)]">
                {filtered.length} items
              </span>
              <FilterField
                label="Sort"
                id="explore-sort"
                value={sort}
                onChange={setSort}
                className="min-w-[9.5rem]"
              >
                <option>Featured</option>
                <option>Latest</option>
                <option>Price: Low</option>
                <option>Price: High</option>
                <option>Rating</option>
              </FilterField>
            </div>
          }
        />
      </ScrollReveal>

      <div className="mt-6 md:hidden">
        <details className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-subtle)]">
          <summary className="cursor-pointer text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-green)]">
            Filter & Sort
          </summary>
          <div className="mt-4 space-y-3">
            {filterPanel}
            <FilterField label="Sort" id="explore-sort-mobile" value={sort} onChange={setSort}>
              <option>Featured</option>
              <option>Latest</option>
              <option>Price: Low</option>
              <option>Price: High</option>
              <option>Rating</option>
            </FilterField>
          </div>
        </details>
      </div>

      <div className="mt-8 grid items-start gap-6 md:grid-cols-[260px_minmax(0,1fr)]">
        <aside className="hidden md:block">{filterPanel}</aside>
        <div>
          {filtered.length === 0 ? (
            <EmptyState
              title="No products found"
              description="Try adjusting your filters to discover more options."
            />
          ) : (
            <div className="grid grid-cols-2 gap-4 md:grid-cols-3 lg:grid-cols-4 md:gap-5">
              {filtered.map((p, i) => (
                <ScrollReveal key={p.id} delay={i * 0.03}>
                  <ProductCard
                    product={p}
                    wished={!!wishlist[p.id]}
                    onToggleWish={onToggleWish}
                    onQuickView={onQuickView}
                  />
                </ScrollReveal>
              ))}
            </div>
          )}
        </div>
      </div>
    </Section>
  );
}
