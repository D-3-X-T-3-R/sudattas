"use client";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { Download, Filter } from "lucide-react";
import { AdminFilterCard } from "@/components/admin/admin-cards";

type CustomersFiltersCardProps = {
  searchQuery: string;
  setSearchQuery: (value: string) => void;
  filterAuth: string;
  setFilterAuth: (value: string) => void;
  pageSize: number;
  setPageSize: (value: number) => void;
  setPage: (value: number) => void;
  onRefresh: () => void;
  onExportCsv: () => void;
  exportDisabled: boolean;
};

export function CustomersFiltersCard({
  searchQuery,
  setSearchQuery,
  filterAuth,
  setFilterAuth,
  pageSize,
  setPageSize,
  setPage,
  onRefresh,
  onExportCsv,
  exportDisabled,
}: CustomersFiltersCardProps) {
  return (
    <AdminFilterCard title="Filters" icon={<Filter className="h-4 w-4 text-[var(--color-green)]" />}>
      <div className="flex flex-wrap items-end gap-3">
        <div>
          <label htmlFor="customers-search" className="mb-1 block text-xs text-[var(--color-muted)]">
            Search
          </label>
          <Input
            id="customers-search"
            type="text"
            placeholder="Email, name, or user ID"
            value={searchQuery}
            onChange={(e) => {
              setPage(1);
              setSearchQuery(e.target.value);
            }}
            className="h-10 w-64"
          />
        </div>

        <div>
          <label htmlFor="customers-auth" className="mb-1 block text-xs text-[var(--color-muted)]">
            Auth
          </label>
          <select
            id="customers-auth"
            className={cn(
              "h-10 min-w-[8rem] rounded-md border border-[var(--color-line)] bg-white px-3 text-sm",
              "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
            )}
            value={filterAuth}
            onChange={(e) => {
              setPage(1);
              setFilterAuth(e.target.value);
            }}
          >
            <option value="">All</option>
            <option value="email">Email</option>
            <option value="google">Google</option>
          </select>
        </div>

        <div>
          <label htmlFor="customers-page-size" className="mb-1 block text-xs text-[var(--color-muted)]">
            Per page
          </label>
          <select
            id="customers-page-size"
            className={cn(
              "h-10 min-w-[6rem] rounded-md border border-[var(--color-line)] bg-white px-3 text-sm",
              "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
            )}
            value={String(pageSize)}
            onChange={(e) => {
              setPage(1);
              setPageSize(Number(e.target.value));
            }}
          >
            <option value="20">20</option>
            <option value="50">50</option>
            <option value="100">100</option>
          </select>
        </div>

        <Button type="button" variant="outline" size="sm" onClick={onRefresh}>
          Refresh
        </Button>

        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={onExportCsv}
          disabled={exportDisabled}
        >
          <Download className="mr-1.5 h-4 w-4" />
          Export CSV
        </Button>
      </div>
    </AdminFilterCard>
  );
}
