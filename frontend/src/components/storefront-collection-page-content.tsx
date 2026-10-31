"use client";

import { useMemo, useState } from "react";
import Image from "next/image";
import Link from "next/link";
import { X } from "lucide-react";
import { Footer } from "@/components/footer";
import type {
  CollectionCardProduct,
  StorefrontCollectionPageData,
} from "@/lib/storefront-collection-page";
import { PageShell, SectionHeader, EmptyState } from "@/components/ui/page-shell";
import {
  optionalRupeesInputToPaise,
  paiseToRupeesInput,
} from "@/lib/money";
import { cn } from "@/lib/utils";
import {
  CheckboxGroup,
  PriceRangeSlider,
  SelectFilter,
} from "@/components/storefront-filter-controls";

const PLACEHOLDER_IMAGE =
  "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='400' height='500' viewBox='0 0 400 500'%3E%3Crect fill='%23f0ede8' width='400' height='500'/%3E%3Ctext fill='%23999' font-family='sans-serif' font-size='14' x='50%25' y='50%25' dominant-baseline='middle' text-anchor='middle'%3ENo image%3C/text%3E%3C/svg%3E";

const SORT_OPTIONS = [
  { value: "featured", label: "Featured" },
  { value: "price-asc", label: "Price: Low to High" },
  { value: "price-desc", label: "Price: High to Low" },
  { value: "name-asc", label: "Name: A-Z" },
] as const;

type SortOption = (typeof SORT_OPTIONS)[number]["value"];
type FilterOption = { value: string; label: string; count: number };
type AvailabilityValue = "in-stock" | "out-of-stock";
type BlouseValue = "included" | "not-included";

type FilterSelection = {
  selectedCategories: string[];
  selectedFabrics: string[];
  selectedOccasions: string[];
  selectedCrafts: string[];
  selectedSizes: string[];
  selectedBlouse: BlouseValue[];
  selectedAvailability: AvailabilityValue[];
  minPricePaise: number | null;
  maxPricePaise: number | null;
};

type FilterVisibility = {
  category: boolean;
  price: boolean;
  fabric: boolean;
  occasion: boolean;
  craft: boolean;
  size: boolean;
  blouse: boolean;
  availability: boolean;
};

const SIZE_ORDER = ["XS", "S", "M", "L", "XL", "XXL", "3XL"] as const;

function normalizeSizeName(name: string): string {
  const normalized = name.trim().toUpperCase().replace(/\s+/g, "");
  return normalized === "XXXL" ? "3XL" : normalized;
}

function isExternalProductImage(src: string | undefined): boolean {
  if (!src || src.startsWith("/") || src.startsWith("data:")) return false;
  try {
    return new URL(src).hostname !== "images.unsplash.com";
  } catch {
    return false;
  }
}

function buildOptions(
  products: CollectionCardProduct[],
  getValue: (product: CollectionCardProduct) => string | null | undefined
): FilterOption[] {
  const counts = new Map<string, number>();
  for (const product of products) {
    const value = (getValue(product) ?? "").trim();
    if (!value) continue;
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }
  return Array.from(counts.entries())
    .map(([value, count]) => ({ value, label: value, count }))
    .sort((a, b) => a.label.localeCompare(b.label));
}

function buildBlouseOptions(products: CollectionCardProduct[]): FilterOption[] {
  let included = 0;
  let notIncluded = 0;
  for (const product of products) {
    if (product.hasBlousePiece == null) continue;
    if (product.hasBlousePiece) included += 1;
    else notIncluded += 1;
  }
  return [
    ...(included > 0
      ? [{ value: "included", label: "Blouse piece included", count: included }]
      : []),
    ...(notIncluded > 0
      ? [{ value: "not-included", label: "No blouse piece", count: notIncluded }]
      : []),
  ];
}

function productKnownStock(product: CollectionCardProduct): number | null {
  if (product.variantStock.length > 0) {
    return product.variantStock.reduce((sum, row) => sum + Math.max(0, row.quantity), 0);
  }
  const stock = product.stockQuantity?.trim();
  if (!stock) return null;
  const parsed = Number.parseInt(stock, 10);
  return Number.isFinite(parsed) ? Math.max(0, parsed) : null;
}

function buildAvailabilityOptions(products: CollectionCardProduct[]): FilterOption[] {
  let inStock = 0;
  let outOfStock = 0;
  for (const product of products) {
    const stock = productKnownStock(product);
    if (stock == null) continue;
    if (stock > 0) inStock += 1;
    else outOfStock += 1;
  }
  return [
    ...(inStock > 0 ? [{ value: "in-stock", label: "In stock", count: inStock }] : []),
    ...(outOfStock > 0
      ? [{ value: "out-of-stock", label: "Out of stock", count: outOfStock }]
      : []),
  ];
}

function buildSizeOptions(products: CollectionCardProduct[]): FilterOption[] {
  const counts = new Map<string, number>();
  for (const product of products) {
    const productSizes = new Set(
      product.variantStock
        .map((row) => normalizeSizeName(row.sizeName))
        .filter((name) => name && name !== "FREESIZE")
    );
    for (const size of productSizes) {
      counts.set(size, (counts.get(size) ?? 0) + 1);
    }
  }
  return SIZE_ORDER.filter((size) => counts.has(size)).map((size) => ({
    value: size,
    label: size,
    count: counts.get(size) ?? 0,
  }));
}

function hasUsefulOptions(options: FilterOption[], totalProducts: number): boolean {
  if (options.length > 1) return true;
  return options.length === 1 && options[0].count < totalProducts;
}

function toggleValue<T extends string>(values: T[], value: T): T[] {
  return values.includes(value)
    ? values.filter((entry) => entry !== value)
    : [...values, value];
}

function filterMatches(product: CollectionCardProduct, selection: FilterSelection): boolean {
  if (
    selection.selectedCategories.length > 0 &&
    !selection.selectedCategories.includes(product.categoryName)
  ) {
    return false;
  }
  if (selection.selectedFabrics.length > 0 && !selection.selectedFabrics.includes(product.fabric)) {
    return false;
  }
  if (
    selection.selectedOccasions.length > 0 &&
    !selection.selectedOccasions.includes(product.occasion)
  ) {
    return false;
  }
  if (selection.selectedCrafts.length > 0 && !selection.selectedCrafts.includes(product.weave)) {
    return false;
  }
  if (selection.selectedSizes.length > 0) {
    const productSizes = product.variantStock
      .map((row) => normalizeSizeName(row.sizeName))
      .filter((name) => name && name !== "FREESIZE");
    if (!productSizes.some((size) => selection.selectedSizes.includes(size))) return false;
  }
  if (selection.selectedBlouse.length > 0) {
    const blouseValue =
      product.hasBlousePiece == null ? null : product.hasBlousePiece ? "included" : "not-included";
    if (!blouseValue || !selection.selectedBlouse.includes(blouseValue)) return false;
  }
  if (selection.selectedAvailability.length > 0) {
    const stock = productKnownStock(product);
    const value = stock == null ? null : stock > 0 ? "in-stock" : "out-of-stock";
    if (!value || !selection.selectedAvailability.includes(value)) return false;
  }
  if (selection.minPricePaise != null && product.pricePaise < selection.minPricePaise) return false;
  if (selection.maxPricePaise != null && product.pricePaise > selection.maxPricePaise) return false;
  return true;
}

function sortProducts(products: CollectionCardProduct[], sortBy: SortOption): CollectionCardProduct[] {
  if (sortBy === "price-asc") {
    return [...products].sort((a, b) => a.pricePaise - b.pricePaise || a.name.localeCompare(b.name));
  }
  if (sortBy === "price-desc") {
    return [...products].sort((a, b) => b.pricePaise - a.pricePaise || a.name.localeCompare(b.name));
  }
  if (sortBy === "name-asc") {
    return [...products].sort((a, b) => a.name.localeCompare(b.name));
  }
  return products;
}

function useCollectionFilters(data: StorefrontCollectionPageData) {
  const [sortBy, setSortBy] = useState<SortOption>("featured");
  const [selectedCategories, setSelectedCategories] = useState<string[]>([]);
  const [selectedFabrics, setSelectedFabrics] = useState<string[]>([]);
  const [selectedOccasions, setSelectedOccasions] = useState<string[]>([]);
  const [selectedCrafts, setSelectedCrafts] = useState<string[]>([]);
  const [selectedSizes, setSelectedSizes] = useState<string[]>([]);
  const [selectedBlouse, setSelectedBlouse] = useState<BlouseValue[]>([]);
  const [selectedAvailability, setSelectedAvailability] = useState<AvailabilityValue[]>([]);
  const [minPrice, setMinPrice] = useState("");
  const [maxPrice, setMaxPrice] = useState("");

  const categoryOptions = useMemo(
    () => buildOptions(data.products, (product) => product.categoryName),
    [data.products]
  );
  const fabricOptions = useMemo(
    () => buildOptions(data.products, (product) => product.fabric),
    [data.products]
  );
  const occasionOptions = useMemo(
    () => buildOptions(data.products, (product) => product.occasion),
    [data.products]
  );
  const craftOptions = useMemo(
    () => buildOptions(data.products, (product) => product.weave),
    [data.products]
  );
  const blouseOptions = useMemo(() => buildBlouseOptions(data.products), [data.products]);
  const availabilityOptions = useMemo(
    () => buildAvailabilityOptions(data.products),
    [data.products]
  );
  const sizeOptions = useMemo(() => buildSizeOptions(data.products), [data.products]);
  const priceBounds = useMemo(() => {
    const prices = data.products.map((product) => product.pricePaise).filter((price) => price >= 0);
    return {
      min: prices.length > 0 ? Math.min(...prices) : 0,
      max: prices.length > 0 ? Math.max(...prices) : 0,
    };
  }, [data.products]);

  const minPricePaise = optionalRupeesInputToPaise(minPrice);
  const maxPricePaise = optionalRupeesInputToPaise(maxPrice);
  const selectedMinPricePaise = minPricePaise ?? priceBounds.min;
  const selectedMaxPricePaise = maxPricePaise ?? priceBounds.max;
  const hasActivePriceFilter =
    (minPricePaise != null && minPricePaise > priceBounds.min) ||
    (maxPricePaise != null && maxPricePaise < priceBounds.max);
  const setPriceRangePaise = (nextMinPaise: number, nextMaxPaise: number) => {
    const boundedMin = Math.max(priceBounds.min, Math.min(nextMinPaise, priceBounds.max));
    const boundedMax = Math.max(priceBounds.min, Math.min(nextMaxPaise, priceBounds.max));
    const clampedMin = Math.min(boundedMin, boundedMax);
    const clampedMax = Math.max(boundedMin, boundedMax);
    setMinPrice(clampedMin <= priceBounds.min ? "" : paiseToRupeesInput(clampedMin));
    setMaxPrice(clampedMax >= priceBounds.max ? "" : paiseToRupeesInput(clampedMax));
  };
  const selection: FilterSelection = useMemo(
    () => ({
      selectedCategories,
      selectedFabrics,
      selectedOccasions,
      selectedCrafts,
      selectedSizes,
      selectedBlouse,
      selectedAvailability,
      minPricePaise,
      maxPricePaise,
    }),
    [
      maxPricePaise,
      minPricePaise,
      selectedAvailability,
      selectedBlouse,
      selectedCategories,
      selectedCrafts,
      selectedFabrics,
      selectedOccasions,
      selectedSizes,
    ]
  );

  const filteredProducts = useMemo(
    () => sortProducts(data.products.filter((product) => filterMatches(product, selection)), sortBy),
    [data.products, selection, sortBy]
  );
  const visibility: FilterVisibility = {
    category: hasUsefulOptions(categoryOptions, data.products.length),
    price: data.products.length > 0,
    fabric: hasUsefulOptions(fabricOptions, data.products.length),
    occasion: hasUsefulOptions(occasionOptions, data.products.length),
    craft: hasUsefulOptions(craftOptions, data.products.length),
    size: hasUsefulOptions(sizeOptions, data.products.length),
    blouse:
      data.categoryName.toLowerCase().includes("saree") &&
      hasUsefulOptions(blouseOptions, data.products.length),
    availability: hasUsefulOptions(availabilityOptions, data.products.length),
  };
  const hasActiveFilters =
    selectedCategories.length > 0 ||
    selectedFabrics.length > 0 ||
    selectedOccasions.length > 0 ||
    selectedCrafts.length > 0 ||
    selectedSizes.length > 0 ||
    selectedBlouse.length > 0 ||
    selectedAvailability.length > 0 ||
    hasActivePriceFilter;

  const resetFilters = () => {
    setSelectedCategories([]);
    setSelectedFabrics([]);
    setSelectedOccasions([]);
    setSelectedCrafts([]);
    setSelectedSizes([]);
    setSelectedBlouse([]);
    setSelectedAvailability([]);
    setMinPrice("");
    setMaxPrice("");
  };

  return {
    sortBy,
    setSortBy,
    selectedCategories,
    setSelectedCategories,
    selectedFabrics,
    setSelectedFabrics,
    selectedOccasions,
    setSelectedOccasions,
    selectedCrafts,
    setSelectedCrafts,
    selectedSizes,
    setSelectedSizes,
    selectedBlouse,
    setSelectedBlouse,
    selectedAvailability,
    setSelectedAvailability,
    minPrice,
    setMinPrice,
    maxPrice,
    setMaxPrice,
    minPricePaise,
    maxPricePaise,
    selectedMinPricePaise,
    selectedMaxPricePaise,
    hasActivePriceFilter,
    setPriceRangePaise,
    categoryOptions,
    fabricOptions,
    occasionOptions,
    craftOptions,
    sizeOptions,
    blouseOptions,
    availabilityOptions,
    priceBounds,
    filteredProducts,
    visibility,
    hasActiveFilters,
    resetFilters,
  };
}

function PriceRange({
  controls,
  idPrefix,
}: {
  controls: ReturnType<typeof useCollectionFilters>;
  idPrefix: string;
}) {
  return (
    <PriceRangeSlider
      id={`${idPrefix}-price`}
      minPaise={controls.priceBounds.min}
      maxPaise={controls.priceBounds.max}
      selectedMinPaise={controls.selectedMinPricePaise}
      selectedMaxPaise={controls.selectedMaxPricePaise}
      onChange={controls.setPriceRangePaise}
    />
  );
}

function SortSelect({
  value,
  onChange,
  className,
}: {
  value: SortOption;
  onChange: (value: SortOption) => void;
  className?: string;
}) {
  return (
    <label className={cn("block", className)}>
      <span className="mb-1 block text-[10px] font-semibold uppercase tracking-[0.16em] text-[var(--color-muted)]">
        Sort
      </span>
      <select
        value={value}
        onChange={(event) => onChange(event.target.value as SortOption)}
        className="h-10 w-full rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 text-sm text-[var(--color-ink)] outline-none focus:border-[var(--color-green)] focus:ring-2 focus:ring-[var(--color-focus)]"
      >
        {SORT_OPTIONS.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

function CollectionFiltersPanel({
  controls,
  idPrefix,
  flat = false,
}: {
  controls: ReturnType<typeof useCollectionFilters>;
  idPrefix: string;
  flat?: boolean;
}) {
  const hasVisibleFilters = Object.values(controls.visibility).some(Boolean);

  return (
    <div
      className={cn(
        "rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-subtle)]",
        flat && "rounded-none border-0 bg-transparent p-0 shadow-none"
      )}
    >
      <div className="flex min-h-9 items-center justify-between gap-3">
        <p className="text-[10px] font-semibold uppercase tracking-[0.22em] text-[var(--color-muted)]">
          Filter By
        </p>
        <button
          type="button"
          onClick={controls.hasActiveFilters ? controls.resetFilters : undefined}
          disabled={!controls.hasActiveFilters}
          aria-hidden={!controls.hasActiveFilters}
          tabIndex={controls.hasActiveFilters ? undefined : -1}
          className={cn(
            "inline-flex min-h-9 shrink-0 items-center gap-1 rounded-md border border-[var(--color-line)] px-2.5 text-[10px] font-semibold uppercase tracking-[0.14em] text-[var(--color-green)] hover:border-[var(--color-gold)]",
            !controls.hasActiveFilters && "invisible pointer-events-none"
          )}
        >
          <X size={13} />
          Reset
        </button>
      </div>
      {hasVisibleFilters ? (
        <FilterGroups controls={controls} idPrefix={idPrefix} />
      ) : (
        <p className="mt-3 text-sm text-[var(--color-muted)]">
          No additional filters are available for this collection yet.
        </p>
      )}
    </div>
  );
}

function FilterGroups({
  controls,
  idPrefix,
}: {
  controls: ReturnType<typeof useCollectionFilters>;
  idPrefix: string;
}) {
  return (
    <div className="mt-4 space-y-4">
      {controls.visibility.category ? (
        <SelectFilter
          title="Category"
          id={`${idPrefix}-category-filter`}
          options={controls.categoryOptions}
          value={controls.selectedCategories[0] ?? ""}
          onChange={(value) => controls.setSelectedCategories(value ? [value] : [])}
        />
      ) : null}
      {controls.visibility.price ? (
        <PriceRange controls={controls} idPrefix={idPrefix} />
      ) : null}
      {controls.visibility.fabric ? (
        <SelectFilter
          title="Fabric"
          id={`${idPrefix}-fabric-filter`}
          options={controls.fabricOptions}
          value={controls.selectedFabrics[0] ?? ""}
          onChange={(value) => controls.setSelectedFabrics(value ? [value] : [])}
        />
      ) : null}
      {controls.visibility.occasion ? (
        <SelectFilter
          title="Occasion"
          id={`${idPrefix}-occasion-filter`}
          options={controls.occasionOptions}
          value={controls.selectedOccasions[0] ?? ""}
          onChange={(value) => controls.setSelectedOccasions(value ? [value] : [])}
        />
      ) : null}
      {controls.visibility.craft ? (
        <SelectFilter
          title="Craft"
          id={`${idPrefix}-craft-filter`}
          options={controls.craftOptions}
          value={controls.selectedCrafts[0] ?? ""}
          onChange={(value) => controls.setSelectedCrafts(value ? [value] : [])}
        />
      ) : null}
      {controls.visibility.size ? (
        <CheckboxGroup
          title="Size"
          options={controls.sizeOptions}
          selected={controls.selectedSizes}
          onToggle={(value) => controls.setSelectedSizes((current) => toggleValue(current, value))}
        />
      ) : null}
      {controls.visibility.blouse ? (
        <CheckboxGroup
          title="Blouse Option"
          options={controls.blouseOptions}
          selected={controls.selectedBlouse}
          onToggle={(value) =>
            controls.setSelectedBlouse((current) => toggleValue(current, value as BlouseValue))
          }
        />
      ) : null}
      {controls.visibility.availability ? (
        <CheckboxGroup
          title="Availability"
          options={controls.availabilityOptions}
          selected={controls.selectedAvailability}
          onToggle={(value) =>
            controls.setSelectedAvailability((current) =>
              toggleValue(current, value as AvailabilityValue)
            )
          }
        />
      ) : null}
    </div>
  );
}

function ProductGrid({ products }: { products: CollectionCardProduct[] }) {
  return (
    <div className="grid grid-cols-2 gap-4 md:gap-5 lg:grid-cols-3">
      {products.map((product) => (
        <Link
          key={product.id}
          href={`/product/${encodeURIComponent(product.id)}`}
          className="group flex flex-col"
        >
          <div className="relative aspect-[2/3] overflow-hidden rounded-[var(--radius-md)] border border-[var(--color-line)] bg-[var(--color-surface-soft)]">
            <Image
              src={product.imageUrl || PLACEHOLDER_IMAGE}
              alt={product.name}
              fill
              className="object-cover transition-transform duration-500 ease-out group-hover:scale-[1.04]"
              sizes="(max-width: 768px) 50vw, 25vw"
              unoptimized={isExternalProductImage(product.imageUrl)}
            />
            <span className="pointer-events-none absolute left-3 top-3 rounded-full border border-[var(--color-line)] bg-[var(--color-surface)]/90 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.2em] text-[var(--color-green)] backdrop-blur-sm">
              New
            </span>
            {/* Extension point: once CollectionCardProduct carries hoverImage/description, swap this for the full ProductCard Quick Add overlay. */}
            <span className="pointer-events-none absolute inset-x-0 bottom-0 translate-y-full bg-[var(--color-green)] py-2.5 text-center text-[11px] font-semibold uppercase tracking-[0.18em] text-white transition-transform duration-300 ease-out group-hover:translate-y-0">
              View Details
            </span>
          </div>
          <div className="mt-3 flex items-start justify-between gap-3 sm:mt-4">
            <h2 className="line-clamp-2 break-words font-display text-[1.05rem] leading-snug text-[var(--color-ink)] sm:text-xl">
              {product.name}
            </h2>
            <p className="whitespace-nowrap pt-0.5 font-sans text-sm font-semibold text-[var(--color-ink)] sm:text-base">
              {product.priceLabel}
            </p>
          </div>
        </Link>
      ))}
    </div>
  );
}

function ProductResults({
  products,
  onReset,
}: {
  products: CollectionCardProduct[];
  onReset: () => void;
}) {
  if (products.length > 0) return <ProductGrid products={products} />;

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

export function StorefrontCollectionPageContent({
  data,
}: {
  data: StorefrontCollectionPageData;
}) {
  const controls = useCollectionFilters(data);

  return (
    <div className="min-h-screen bg-[var(--background)] text-[var(--foreground)]">
      <PageShell wide containerClassName="py-8 md:py-10">
        <SectionHeader
          label="Collection"
          title={data.categoryName}
          description="Thoughtfully designed pieces with premium fabrics and timeless silhouettes."
          action={
            <div className="flex flex-wrap items-end gap-3">
              <span className="pb-2 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-muted)]">
                {controls.filteredProducts.length} items
              </span>
              <SortSelect
                value={controls.sortBy}
                onChange={controls.setSortBy}
                className="min-w-[12rem]"
              />
            </div>
          }
        />

        {data.products.length === 0 ? (
          <div className="mt-8">
            <EmptyState
              title="No products available"
              description="This collection is currently being refreshed. Please check back shortly."
            />
          </div>
        ) : (
          <>
            <div className="mt-6 md:hidden">
              <details className="overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)] shadow-[var(--shadow-subtle)]">
                <summary className="flex min-h-12 cursor-pointer list-none items-center justify-between px-4 text-[11px] font-semibold uppercase tracking-[0.16em] text-[var(--color-green)] marker:hidden">
                  <span>Filters</span>
                  <span className="text-[10px] tracking-[0.14em] text-[var(--color-muted)]">
                    Refine
                  </span>
                </summary>
                <div className="border-t border-[var(--color-line)] p-4">
                  <CollectionFiltersPanel
                    controls={controls}
                    idPrefix="collection-mobile"
                    flat
                  />
                </div>
              </details>
            </div>

            <section className="mt-8 grid items-start gap-6 md:grid-cols-[260px_minmax(0,1fr)]">
              <aside className="hidden md:block">
                <CollectionFiltersPanel
                  controls={controls}
                  idPrefix="collection-desktop"
                />
              </aside>
              <div>
                <ProductResults
                  products={controls.filteredProducts}
                  onReset={controls.resetFilters}
                />
              </div>
            </section>
          </>
        )}
      </PageShell>
      <Footer />
    </div>
  );
}
