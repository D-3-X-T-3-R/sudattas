"use client";

import { Filter } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { AdminFilterCard } from "@/components/admin/admin-cards";

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
      <label htmlFor={id} className="mb-1.5 block text-sm font-medium text-[var(--color-muted)]">
        {label}
      </label>
      <select
        id={id}
        className={cn(
          "h-11 w-full rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px]",
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
    <div>
      <label htmlFor="products-price-min" className="mb-1.5 block text-sm font-medium text-[var(--color-muted)]">
        Price range (INR)
      </label>
      <div className="flex items-center gap-2">
        <Input
          id="products-price-min"
          type="number"
          min={0}
          step={0.01}
          placeholder="Min"
          value={min}
          onChange={(e) => onMin(e.target.value)}
          className="h-11"
        />
        <span className="text-[var(--color-muted)]">-</span>
        <Input
          id="products-price-max"
          type="number"
          min={0}
          step={0.01}
          placeholder="Max"
          value={max}
          onChange={(e) => onMax(e.target.value)}
          className="h-11"
        />
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
    <AdminFilterCard title="Filters" icon={<Filter className="h-4 w-4 text-[var(--color-green)]" />} className="mt-6">
      <form onSubmit={onApply} className="space-y-5">
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <div>
            <label htmlFor="products-name" className="mb-1.5 block text-sm font-medium text-[var(--color-muted)]">
              Name
            </label>
            <Input
              id="products-name"
              type="text"
              value={filters.searchName}
              onChange={(e) => set({ searchName: e.target.value })}
              placeholder="e.g. silk"
              className="h-11"
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
            <label htmlFor="products-status" className="mb-1.5 block text-sm font-medium text-[var(--color-muted)]">
              Status
            </label>
            <select
              id="products-status"
              className={cn(
                "h-11 w-full rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px]",
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

          <PriceRangeFilter
            min={filters.searchPriceMinRupees}
            max={filters.searchPriceMaxRupees}
            onMin={(value) => set({ searchPriceMinRupees: value })}
            onMax={(value) => set({ searchPriceMaxRupees: value })}
          />

          <div>
            <label htmlFor="products-limit" className="mb-1.5 block text-sm font-medium text-[var(--color-muted)]">
              Results per page
            </label>
            <Input
              id="products-limit"
              type="number"
              min={1}
              max={100}
              value={filters.searchLimit}
              onChange={(e) => set({ searchLimit: e.target.value })}
              className="h-11"
            />
          </div>
        </div>

        <div className="flex flex-wrap gap-2 border-t border-[var(--color-line)] pt-4">
          <Button type="submit">Apply filters</Button>
          <Button type="button" variant="outline" onClick={onClear}>Clear</Button>
          <Button type="button" variant="outline" onClick={onRefresh}>Refresh</Button>
        </div>
      </form>
    </AdminFilterCard>
  );
}
