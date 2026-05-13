"use client";

import { X } from "lucide-react";
import { ProductCard } from "@/components/product-card";
import type { Product } from "@/lib/schemas";
import { Section } from "@/components/ui/section";
import { ScrollReveal } from "@/components/scroll-reveal";
import { SectionHeader, EmptyState } from "@/components/ui/page-shell";
import {
  EXPLORE_SORT_OPTIONS,
  type AvailabilityValue,
  type BlouseValue,
  type ExploreSortOption,
  type StorefrontCatalogController,
} from "@/domains/storefront/hooks/use-storefront-catalog";
import {
  CheckboxGroup,
  PriceRangeSlider,
  SelectFilter,
} from "@/components/storefront-filter-controls";
import { cn } from "@/lib/utils";

export interface ExploreSectionProps {
  catalog: StorefrontCatalogController;
  wishlist: Record<string, boolean>;
  onToggleWish: (p: Product) => void;
  onQuickView: (p: Product) => void;
}

function PriceRange({
  catalog,
  idPrefix,
}: {
  catalog: StorefrontCatalogController;
  idPrefix: string;
}) {
  return (
    <PriceRangeSlider
      id={`${idPrefix}-price`}
      minPaise={catalog.priceBounds.min}
      maxPaise={catalog.priceBounds.max}
      selectedMinPaise={catalog.selectedMinPricePaise}
      selectedMaxPaise={catalog.selectedMaxPricePaise}
      onChange={catalog.setPriceRangePaise}
    />
  );
}

function SortSelect({
  value,
  onChange,
  className,
}: {
  value: ExploreSortOption;
  onChange: (value: ExploreSortOption) => void;
  className?: string;
}) {
  return (
    <label className={cn("block", className)}>
      <span className="mb-1 block text-[10px] font-semibold uppercase tracking-[0.16em] text-[var(--color-muted)]">
        Sort
      </span>
      <select
        value={value}
        onChange={(event) => onChange(event.target.value as ExploreSortOption)}
        className="h-10 w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 text-sm text-[var(--color-ink)] outline-none focus:border-[var(--color-green)] focus:ring-2 focus:ring-[var(--color-focus)]"
      >
        {EXPLORE_SORT_OPTIONS.map((option) => (
          <option key={option} value={option}>
            {option}
          </option>
        ))}
      </select>
    </label>
  );
}

function FilterGroups({
  catalog,
  idPrefix,
}: {
  catalog: StorefrontCatalogController;
  idPrefix: string;
}) {
  return (
    <div className="mt-4 space-y-4">
      {catalog.filterVisibility.category ? (
        <SelectFilter
          title="Category"
          id={`${idPrefix}-category-filter`}
          options={catalog.categoryOptions}
          value={catalog.selectedCategories[0] ?? ""}
          onChange={(value) => catalog.setSelectedCategories(value ? [value] : [])}
        />
      ) : null}
      {catalog.filterVisibility.price ? (
        <PriceRange catalog={catalog} idPrefix={idPrefix} />
      ) : null}
      {catalog.filterVisibility.fabric ? (
        <SelectFilter
          title="Fabric"
          id={`${idPrefix}-fabric-filter`}
          options={catalog.fabricOptions}
          value={catalog.selectedFabrics[0] ?? ""}
          onChange={(value) => catalog.setSelectedFabrics(value ? [value] : [])}
        />
      ) : null}
      {catalog.filterVisibility.occasion ? (
        <SelectFilter
          title="Occasion"
          id={`${idPrefix}-occasion-filter`}
          options={catalog.occasionFilterOptions}
          value={catalog.selectedOccasions[0] ?? ""}
          onChange={(value) => catalog.setSelectedOccasions(value ? [value] : [])}
        />
      ) : null}
      {catalog.filterVisibility.size ? (
        <CheckboxGroup
          title="Size"
          options={catalog.sizeOptions}
          selected={catalog.selectedSizes}
          onToggle={(value) =>
            catalog.setSelectedSizes((current) => catalog.toggleValue(current, value))
          }
        />
      ) : null}
      {catalog.filterVisibility.craft ? (
        <SelectFilter
          title="Craft"
          id={`${idPrefix}-craft-filter`}
          options={catalog.craftOptions}
          value={catalog.selectedCrafts[0] ?? ""}
          onChange={(value) => catalog.setSelectedCrafts(value ? [value] : [])}
        />
      ) : null}
      {catalog.filterVisibility.blouse ? (
        <CheckboxGroup
          title="Blouse Option"
          options={catalog.blouseOptions}
          selected={catalog.selectedBlouse}
          onToggle={(value) =>
            catalog.setSelectedBlouse((current) =>
              catalog.toggleValue(current, value as BlouseValue)
            )
          }
        />
      ) : null}
      {catalog.filterVisibility.availability ? (
        <CheckboxGroup
          title="Availability"
          options={catalog.availabilityOptions}
          selected={catalog.selectedAvailability}
          onToggle={(value) =>
            catalog.setSelectedAvailability((current) =>
              catalog.toggleValue(current, value as AvailabilityValue)
            )
          }
        />
      ) : null}
    </div>
  );
}

function FiltersPanel({
  catalog,
  idPrefix,
}: {
  catalog: StorefrontCatalogController;
  idPrefix: string;
}) {
  const hasVisibleFilters = Object.values(catalog.filterVisibility).some(Boolean);

  return (
    <div className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-subtle)]">
      <div className="flex min-h-9 items-center justify-between gap-3">
        <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[var(--color-muted)]">
          Filter By
        </p>
        <button
          type="button"
          onClick={catalog.hasActiveFilters ? catalog.resetFilters : undefined}
          disabled={!catalog.hasActiveFilters}
          aria-hidden={!catalog.hasActiveFilters}
          tabIndex={catalog.hasActiveFilters ? undefined : -1}
          className={cn(
            "inline-flex min-h-9 shrink-0 items-center gap-1 rounded-md border border-[var(--color-line)] px-2.5 text-[10px] font-semibold uppercase tracking-[0.14em] text-[var(--color-green)] hover:border-[var(--color-gold)]",
            !catalog.hasActiveFilters && "invisible pointer-events-none"
          )}
        >
          <X size={13} />
          Reset
        </button>
      </div>
      {hasVisibleFilters ? (
        <FilterGroups catalog={catalog} idPrefix={idPrefix} />
      ) : (
        <p className="mt-3 text-sm text-[var(--color-muted)]">
          No additional filters are available for these products yet.
        </p>
      )}
    </div>
  );
}

function ProductResults({
  products,
  wishlist,
  onToggleWish,
  onQuickView,
  onReset,
}: {
  products: Product[];
  wishlist: Record<string, boolean>;
  onToggleWish: (p: Product) => void;
  onQuickView: (p: Product) => void;
  onReset: () => void;
}) {
  if (products.length === 0) {
    return (
      <EmptyState
        title="No products match these filters"
        description="Clear the filters or adjust your range to see more pieces."
        action={
          <button
            type="button"
            onClick={onReset}
            className="inline-flex h-10 items-center justify-center rounded-md border border-[var(--color-green)] bg-[var(--color-green)] px-4 text-[11px] font-semibold uppercase tracking-[0.14em] text-white hover:border-[var(--color-green-2)] hover:bg-[var(--color-green-2)]"
          >
            Reset filters
          </button>
        }
      />
    );
  }

  return (
    <div className="grid grid-cols-2 gap-4 md:gap-5 lg:grid-cols-3">
      {products.map((product, index) => (
        <ScrollReveal key={product.id} delay={index * 0.03}>
          <ProductCard
            product={product}
            wished={!!wishlist[product.id]}
            onToggleWish={onToggleWish}
            onQuickView={onQuickView}
          />
        </ScrollReveal>
      ))}
    </div>
  );
}

export function ExploreSection({
  catalog,
  wishlist,
  onToggleWish,
  onQuickView,
}: ExploreSectionProps) {
  return (
    <Section id="explore">
      <ScrollReveal>
        <SectionHeader
          label="Catalog"
          title="Explore"
          description="Browse curated sarees, kurtis, and occasion-ready pieces from Sudatta's."
          action={
            <div className="flex flex-wrap items-end gap-3">
              <span className="pb-2 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-muted)]">
                {catalog.filtered.length} items
              </span>
              <SortSelect
                value={catalog.sort}
                onChange={catalog.setSort}
                className="min-w-[12rem]"
              />
            </div>
          }
        />
      </ScrollReveal>

      <div className="mt-6 md:hidden">
        <details className="rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-subtle)]">
          <summary className="cursor-pointer text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-green)]">
            Filters
          </summary>
          <div className="mt-4">
            <FiltersPanel catalog={catalog} idPrefix="explore-mobile" />
          </div>
        </details>
      </div>

      <div className="mt-8 grid items-start gap-6 md:grid-cols-[260px_minmax(0,1fr)]">
        <aside className="hidden md:block">
          <FiltersPanel catalog={catalog} idPrefix="explore-desktop" />
        </aside>
        <div>
          <ProductResults
            products={catalog.filtered}
            wishlist={wishlist}
            onToggleWish={onToggleWish}
            onQuickView={onQuickView}
            onReset={catalog.resetFilters}
          />
        </div>
      </div>
    </Section>
  );
}
