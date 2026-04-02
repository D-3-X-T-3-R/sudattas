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
  const set = (patch: Partial<ProductsFiltersState>) =>
    onFiltersChange({ ...filters, ...patch });

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
          <div>
            <label htmlFor="products-category" className="mb-1 block text-xs text-[var(--color-muted)]">
              Category
            </label>
            <select
              id="products-category"
              className={cn(
                "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              )}
              value={filters.searchCategoryId}
              onChange={(e) => set({ searchCategoryId: e.target.value })}
            >
              <option value="">All categories</option>
              {categories.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name}
                </option>
              ))}
            </select>
          </div>
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
          <div>
            <label htmlFor="products-mood" className="mb-1 block text-xs text-[var(--color-muted)]">
              Mood
            </label>
            <select
              id="products-mood"
              className={cn(
                "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              )}
              value={filters.searchMoodId}
              onChange={(e) => set({ searchMoodId: e.target.value })}
            >
              <option value="">All moods</option>
              {moods.map((m) => (
                <option key={m.id} value={m.id}>
                  {m.name}
                </option>
              ))}
            </select>
          </div>
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
                  value={filters.searchPriceMinRupees}
                  onChange={(e) => set({ searchPriceMinRupees: e.target.value })}
                  className="h-9 w-24 rounded-md"
                />
                <span className="text-xs text-[var(--color-muted)]">-</span>
                <Input
                  id="products-price-max"
                  type="number"
                  min={0}
                  step={0.01}
                  placeholder="Max"
                  value={filters.searchPriceMaxRupees}
                  onChange={(e) => set({ searchPriceMaxRupees: e.target.value })}
                  className="h-9 w-24 rounded-md"
                />
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="range"
                  min={0}
                  max={50000}
                  step={100}
                  value={Math.min(Number(filters.searchPriceMinRupees) || 0, 50000)}
                  onChange={(e) => set({ searchPriceMinRupees: e.target.value })}
                  className="h-2 w-24 flex-1 cursor-pointer appearance-none rounded-lg bg-[var(--color-line)] accent-[var(--color-accent-brown)]"
                  aria-label="Min price (INR)"
                />
                <input
                  type="range"
                  min={0}
                  max={50000}
                  step={100}
                  value={filters.searchPriceMaxRupees === "" ? 50000 : Math.min(Number(filters.searchPriceMaxRupees), 50000)}
                  onChange={(e) => set({ searchPriceMaxRupees: e.target.value })}
                  className="h-2 w-24 flex-1 cursor-pointer appearance-none rounded-lg bg-[var(--color-line)] accent-[var(--color-accent-brown)]"
                  aria-label="Max price (INR)"
                />
              </div>
            </div>
          </div>
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
          <div>
            <label htmlFor="products-fabric" className="mb-1 block text-xs text-[var(--color-muted)]">
              Fabric
            </label>
            <select
              id="products-fabric"
              className={cn(
                "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              )}
              value={filters.searchFabric}
              onChange={(e) => set({ searchFabric: e.target.value })}
            >
              <option value="">All fabrics</option>
              {fabrics.map((f) => (
                <option key={f.id} value={f.name}>
                  {f.name}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label htmlFor="products-weave" className="mb-1 block text-xs text-[var(--color-muted)]">
              Weave
            </label>
            <select
              id="products-weave"
              className={cn(
                "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              )}
              value={filters.searchWeave}
              onChange={(e) => set({ searchWeave: e.target.value })}
            >
              <option value="">All weaves</option>
              {weaves.map((w) => (
                <option key={w.id} value={w.name}>
                  {w.name}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label htmlFor="products-occasion" className="mb-1 block text-xs text-[var(--color-muted)]">
              Occasion
            </label>
            <select
              id="products-occasion"
              className={cn(
                "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              )}
              value={filters.searchOccasion}
              onChange={(e) => set({ searchOccasion: e.target.value })}
            >
              <option value="">All occasions</option>
              {occasions.map((o) => (
                <option key={o.id} value={o.name}>
                  {o.name}
                </option>
              ))}
            </select>
          </div>
          <div className="flex gap-2">
            <Button type="submit" size="sm">
              Apply
            </Button>
            <Button type="button" variant="outline" size="sm" onClick={onClear}>
              Clear
            </Button>
            <Button type="button" variant="outline" size="sm" onClick={onRefresh}>
              Refresh
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}

