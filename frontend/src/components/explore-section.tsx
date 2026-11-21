"use client";

import { useState } from "react";
import { Dialog, DialogContent, DialogTrigger } from "@/components/ui/dialog";
import { SlidersHorizontal, X } from "lucide-react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { ProductCard } from "@/components/product-card";
import type { ProductTransitionOrigin } from "@/components/product-card";
import type { Product } from "@/lib/schemas";
import { Section } from "@/components/ui/section";
import { ScrollReveal } from "@/components/scroll-reveal";
import { EmptyState } from "@/components/ui/page-shell";
import {
  EXPLORE_SORT_OPTIONS,
  useExploreFilters,
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
  onQuickView: (p: Product, origin: ProductTransitionOrigin) => void;
}

function PriceRange({
  catalog,
  idPrefix,
}: {
  catalog: ReturnType<typeof useExploreFilters>;
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
    <div className={className}>
      <Select value={value} onValueChange={(nextValue) => onChange(nextValue as ExploreSortOption)}>
        <SelectTrigger aria-label="Sort products" className="h-10 gap-3 [&>svg]:shrink-0 [&>svg]:transition-transform [&>svg]:duration-200 data-[state=open]:[&>svg]:rotate-180 motion-reduce:[&>svg]:transition-none">
          <SelectValue />
        </SelectTrigger>
        <SelectContent className="explore-sort-menu min-w-[var(--radix-select-trigger-width)]" align="end">
          {EXPLORE_SORT_OPTIONS.map((option) => (
            <SelectItem key={option} value={option}>{option}</SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}

function FilterGroups({
  catalog,
  idPrefix,
}: {
  catalog: ReturnType<typeof useExploreFilters>;
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
          singleRow
          showCounts={false}
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
          openUpward
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
  flat = false,
}: {
  catalog: ReturnType<typeof useExploreFilters>;
  idPrefix: string;
  flat?: boolean;
}) {
  const hasVisibleFilters = Object.values(catalog.filterVisibility).some(Boolean);

  return (
    <div
      className={cn(
        "rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-subtle)]",
        flat && "rounded-none border-0 bg-transparent p-0 shadow-none"
      )}
    >
      {!flat && <div className="flex min-h-9 items-center justify-between gap-3">
        <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[var(--color-muted)]">
          Filter By
        </p>
        <button
          type="button"
          onClick={catalog.hasActiveFilters ? catalog.resetFilters : undefined}
          disabled={!catalog.hasActiveFilters}
          tabIndex={catalog.hasActiveFilters ? undefined : -1}
          className={cn(
            "inline-flex min-h-9 shrink-0 items-center gap-1 rounded-md border border-[var(--color-line)] px-2.5 text-[10px] font-semibold uppercase tracking-[0.14em] text-[var(--color-green)] hover:border-[var(--color-gold)]",
            !catalog.hasActiveFilters && "invisible"
          )}
        >
          <X size={13} />
          Reset
        </button>
      </div>}
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
  onQuickView: (p: Product, origin: ProductTransitionOrigin) => void;
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
            transitionScope="explore"
          />
        </ScrollReveal>
      ))}
    </div>
  );
}

const ignoreMoodChange = async () => {};

export function ExploreSection({
  catalog,
  wishlist,
  onToggleWish,
  onQuickView,
}: ExploreSectionProps) {
  const [filtersOpen, setFiltersOpen] = useState(false);
  const draft = useExploreFilters(catalog.products, null, ignoreMoodChange);
  const changeFiltersOpen = (open: boolean) => {
    if (open) {
    draft.setSelectedCategories(catalog.selectedCategories);
    draft.setSelectedFabrics(catalog.selectedFabrics);
    draft.setSelectedOccasions(catalog.selectedOccasions);
    draft.setSelectedSizes(catalog.selectedSizes);
    draft.setSelectedCrafts(catalog.selectedCrafts);
    draft.setSelectedBlouse(catalog.selectedBlouse);
    draft.setSelectedAvailability(catalog.selectedAvailability);
    draft.setMinPrice(catalog.minPrice);
    draft.setMaxPrice(catalog.maxPrice);
    draft.setQuery(catalog.query);
    }
    setFiltersOpen(open);
  };
  const applyFilters = () => {
    catalog.setSelectedCategories(draft.selectedCategories);
    catalog.setSelectedFabrics(draft.selectedFabrics);
    catalog.setSelectedOccasions(draft.selectedOccasions);
    catalog.setSelectedSizes(draft.selectedSizes);
    catalog.setSelectedCrafts(draft.selectedCrafts);
    catalog.setSelectedBlouse(draft.selectedBlouse);
    catalog.setSelectedAvailability(draft.selectedAvailability);
    catalog.setMinPrice(draft.minPrice);
    catalog.setMaxPrice(draft.maxPrice);
    catalog.setQuery(draft.query);
    setFiltersOpen(false);
  };
  return (
    <Dialog open={filtersOpen} onOpenChange={changeFiltersOpen}>
    <Section id="explore">
      <ScrollReveal>
        <header className="border-b border-[var(--color-line)] pb-6 md:pb-8">
          <div className="flex flex-col gap-4 md:flex-row md:items-center">
            <div className="hidden flex-1 md:block" />
            <h2 className="font-display text-[2rem] leading-tight text-[var(--color-ink)] md:text-center md:text-[2.5rem]">
              Explore
            </h2>
            <div className="flex items-center justify-between gap-3 md:flex-1 md:items-end md:justify-end">
              <div className="flex w-full items-center justify-between gap-2 md:w-auto">

                <DialogTrigger asChild>
                <button
                  type="button"
                  aria-label="Filters"
                  className={cn(
                    "inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] text-[var(--color-green)] md:hidden",
                    (filtersOpen || catalog.hasActiveFilters) && "border-[var(--color-green)] bg-[var(--color-surface-soft)]"
                  )}
                >
                  <SlidersHorizontal aria-hidden="true" className="h-5 w-5" />
                </button>
                </DialogTrigger>
                <SortSelect
                value={catalog.sort}
                onChange={catalog.setSort}
                className="min-w-[9.5rem] md:min-w-[12rem]"
                />
              </div>
            </div>
          </div>
        </header>
      </ScrollReveal>

      <DialogContent title="Filters" titleClassName="sr-only" showClose={false} headerClassName="sr-only" overlayClassName="filter-popup-overlay" className="filter-popup max-h-[90dvh] max-w-md rounded-2xl" contentClassName="max-h-[90dvh] overflow-y-auto overscroll-contain p-4">
        <FiltersPanel catalog={draft} idPrefix="explore-mobile" flat />
        <div className="mt-5 grid grid-cols-2 gap-3">
          <button
            type="button"
            onClick={draft.resetFilters}
            disabled={!draft.hasActiveFilters}
            className="min-h-11 rounded-md border border-[var(--color-green)] px-4 text-sm font-semibold text-[var(--color-green)] transition-colors hover:bg-[var(--color-surface-soft)] disabled:cursor-not-allowed disabled:border-[var(--color-line)] disabled:text-[var(--color-muted)] disabled:opacity-50"
          >
            RESET
          </button>
          <button type="button" onClick={applyFilters} className="min-h-11 rounded-md bg-[var(--color-green)] px-4 text-sm font-semibold text-[var(--color-on-deep)]">
            APPLY
          </button>
        </div>
      </DialogContent>

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
    </Dialog>
  );
}
