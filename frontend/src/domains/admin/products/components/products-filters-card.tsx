"use client";

import { Filter } from "lucide-react";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

export interface ProductsFiltersState {
  searchName: string;
  searchCategoryId: string;
  searchProductStatusId: string;
  searchMoodId: string;
  searchPriceMinRupees: string;
  searchPriceMaxRupees: string;
  searchLimit: string;
  searchFabric: string;
  searchWeave: string;
  searchOccasion: string;
}

export interface ProductsFilterOption {
  id: string;
  name: string;
}

interface ProductsFiltersCardProps {
  filters: ProductsFiltersState;
  categories: ProductsFilterOption[];
  moods: ProductsFilterOption[];
  fabrics: ProductsFilterOption[];
  weaves: ProductsFilterOption[];
  occasions: ProductsFilterOption[];
  onFiltersChange: (next: ProductsFiltersState) => void;
  onApply: React.FormEventHandler<HTMLFormElement>;
  onClear: () => void;
  onRefresh: () => void;
}

function FilterSelect({
  id,
  label,
  value,
  onChange,
  options,
  allLabel,
  getOptionValue,
}: {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: ProductsFilterOption[];
  allLabel: string;
  getOptionValue?: (option: ProductsFilterOption) => string;
}) {
  return (
    <div>
      <label htmlFor={id} className="mb-1 block text-xs text-[var(--color-muted)]">
        {label}
      </label>
      <select
        id={id}
        className={cn(
          "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
          "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
        )}
        value={value}
        onChange={(e) => onChange(e.target.value)}
      >
        <option value="">{allLabel}</option>
        {options.map((o) => (
          <option key={o.id} value={getOptionValue ? getOptionValue(o) : o.id}>
            {o.name}
          </option>
        ))}
      </select>
    </div>
  );
}

function PriceRangeFilter({
  min,
  max,
  onMin,
  onMax,
}: {
  min: string;
  max: string;
  onMin: (value: string) => void;
  onMax: (value: string) => void;
}) {
  return (
    <div className="min-w-[14rem]">
      <label htmlFor="products-price-min" className="mb-1 block text-xs text-[var(--color-muted)]">
        Price range (INR)
      </label>
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-2">
          <Input
            id="products-price-min"
            type="number"
            min={0}
            step={0.01}
            placeholder="Min"
            value={min}
            onChange={(e) => onMin(e.target.value)}
            className="h-9 w-24 rounded-md"
          />
          <span className="text-xs text-[var(--color-muted)]">-</span>
          <Input
            id="products-price-max"
            type="number"
            min={0}
            step={0.01}
            placeholder="Max"
            value={max}
            onChange={(e) => onMax(e.target.value)}
            className="h-9 w-24 rounded-md"
          />
        </div>
        <div className="flex items-center gap-2">
          <input
            type="range"
            min={0}
            max={50000}
            step={100}
            value={Math.min(Number(min) || 0, 50000)}
            onChange={(e) => onMin(e.target.value)}
            className="h-2 w-24 flex-1 cursor-pointer appearance-none rounded-lg bg-[var(--color-line)] accent-[var(--color-accent-brown)]"
            aria-label="Min price (INR)"
          />
          <input
            type="range"
            min={0}
            max={50000}
            step={100}
            value={max === "" ? 50000 : Math.min(Number(max), 50000)}
            onChange={(e) => onMax(e.target.value)}
            className="h-2 w-24 flex-1 cursor-pointer appearance-none rounded-lg bg-[var(--color-line)] accent-[var(--color-accent-brown)]"
            aria-label="Max price (INR)"
          />
        </div>
      </div>
    </div>
  );
}

export function ProductsFiltersCard({
  filters,
  categories,
  moods,
  fabrics,
  weaves,
  occasions,
  onFiltersChange,
  onApply,
  onClear,
  onRefresh,
}: ProductsFiltersCardProps) {
  const set = (patch: Partial<ProductsFiltersState>) => onFiltersChange({ ...filters, ...patch });

  return (
    <Card className="mt-6 rounded-xl border-[var(--color-line)] border-l-4 border-l-blue-500 bg-white shadow-[var(--admin-card-shadow)]">
      <CardTitle className="flex items-center gap-2 text-[var(--color-muted)]">
        <Filter className="h-4 w-4 text-blue-500" />
        Filters
      </CardTitle>
      <CardContent className="mt-3">
        <form onSubmit={onApply} className="flex flex-wrap items-end gap-3">
          <div>
            <label htmlFor="products-name" className="mb-1 block text-xs text-[var(--color-muted)]">
              Name
            </label>
            <Input
              id="products-name"
              type="text"
              value={filters.searchName}
              onChange={(e) => set({ searchName: e.target.value })}
              placeholder="e.g. silk"
              className="h-9 w-40 rounded-md"
            />
          </div>

          <FilterSelect
            id="products-category"
            label="Category"
            value={filters.searchCategoryId}
            onChange={(value) => set({ searchCategoryId: value })}
            options={categories}
            allLabel="All categories"
          />

          <div>
            <label htmlFor="products-status" className="mb-1 block text-xs text-[var(--color-muted)]">
              Status
            </label>
            <select
              id="products-status"
              className={cn(
                "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              )}
              value={filters.searchProductStatusId}
              onChange={(e) => set({ searchProductStatusId: e.target.value })}
            >
              <option value="">All statuses</option>
              <option value="1">Draft</option>
              <option value="2">Active</option>
              <option value="3">Archived</option>
            </select>
          </div>

          <FilterSelect
            id="products-mood"
            label="Mood"
            value={filters.searchMoodId}
            onChange={(value) => set({ searchMoodId: value })}
            options={moods}
            allLabel="All moods"
          />

          <PriceRangeFilter
            min={filters.searchPriceMinRupees}
            max={filters.searchPriceMaxRupees}
            onMin={(value) => set({ searchPriceMinRupees: value })}
            onMax={(value) => set({ searchPriceMaxRupees: value })}
          />

          <div>
            <label htmlFor="products-limit" className="mb-1 block text-xs text-[var(--color-muted)]">
              Limit
            </label>
            <Input
              id="products-limit"
              type="number"
              min={1}
              max={100}
              value={filters.searchLimit}
              onChange={(e) => set({ searchLimit: e.target.value })}
              className="h-9 w-20 rounded-md"
            />
          </div>

          <FilterSelect
            id="products-fabric"
            label="Fabric"
            value={filters.searchFabric}
            onChange={(value) => set({ searchFabric: value })}
            options={fabrics}
            allLabel="All fabrics"
            getOptionValue={(o) => o.name}
          />

          <FilterSelect
            id="products-weave"
            label="Weave"
            value={filters.searchWeave}
            onChange={(value) => set({ searchWeave: value })}
            options={weaves}
            allLabel="All weaves"
            getOptionValue={(o) => o.name}
          />

          <FilterSelect
            id="products-occasion"
            label="Occasion"
            value={filters.searchOccasion}
            onChange={(value) => set({ searchOccasion: value })}
            options={occasions}
            allLabel="All occasions"
            getOptionValue={(o) => o.name}
          />

          <div className="flex gap-2">
            <Button type="submit" size="sm">Apply</Button>
            <Button type="button" variant="outline" size="sm" onClick={onClear}>Clear</Button>
            <Button type="button" variant="outline" size="sm" onClick={onRefresh}>Refresh</Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
