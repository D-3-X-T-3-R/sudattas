"use client";

import Link from "next/link";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { CustomerListRow } from "@/lib/admin-queries";
import { ExternalLink, Users } from "lucide-react";
import { formatCreateDate, formatCurrency } from "@/domains/admin/customers/utils";

type SortKey = "name" | "email" | "created" | "orders" | "spent";

type CustomersTableCardProps = {
  isLoading: boolean;
  isError: boolean;
  errorTitle?: string;
  errorMessage?: string;
  onRetry: () => void;
  customersLength: number;
  filteredLength: number;
  pagedCustomers: CustomerListRow[];
  orderStats: Map<string, { count: number; totalPaise: number }>;
  selectedCustomer: CustomerListRow | null;
  setSelectedCustomer: (customer: CustomerListRow | null) => void;
  handleSort: (key: SortKey) => void;
  sortIconFor: (column: SortKey) => React.ReactNode;
  sortKey: SortKey;
  sortDir: "asc" | "desc";
  page: number;
  pageSize: number;
  setPage: (updater: (prev: number) => number) => void;
};

function SortHeader({
  label,
  column,
  onSort,
  icon,
}: {
  label: string;
  column: SortKey;
  onSort: (key: SortKey) => void;
  icon: React.ReactNode;
}) {
  return (
    <button
      type="button"
      className="flex items-center font-medium hover:text-[var(--color-ink)]"
      onClick={() => onSort(column)}
    >
      {label}
      {icon}
    </button>
  );
}

function CustomerRow({
  customer,
  selectedCustomer,
  setSelectedCustomer,
  stats,
}: {
  customer: CustomerListRow;
  selectedCustomer: CustomerListRow | null;
  setSelectedCustomer: (customer: CustomerListRow | null) => void;
  stats?: { count: number; totalPaise: number };
}) {
  return (
    <tr
      className={cn(
        "border-b border-[var(--color-line)] last:border-0 hover:bg-[var(--color-surface)]",
        selectedCustomer?.userId === customer.userId && "bg-[var(--color-line)]/20"
      )}
    >
      <td className="py-3 pr-4 text-[var(--color-ink)]">
        {customer.fullName ?? customer.username ?? "-"}
      </td>
      <td className="py-3 pr-4 text-[var(--color-ink)]">{customer.email}</td>
      <td className="py-3 pr-4 font-mono text-[var(--color-ink)]">{customer.userId}</td>
      <td className="py-3 pr-4 text-[var(--color-muted)]">{customer.authProvider}</td>
      <td className="py-3 pr-4 text-[var(--color-ink)]">{stats?.count ?? 0}</td>
      <td className="py-3 pr-4 text-[var(--color-ink)]">
        {stats ? formatCurrency(stats.totalPaise) : "-"}
      </td>
      <td className="py-3 pr-4 text-[var(--color-muted)]">
        {formatCreateDate(customer.createDate)}
      </td>
      <td className="py-3">
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="mr-2"
          onClick={() => setSelectedCustomer(customer)}
          aria-label={`View profile for ${customer.fullName ?? customer.username ?? customer.email}`}
        >
          View
        </Button>
        <Link
          href={`/imtheboss/orders?userId=${encodeURIComponent(customer.userId)}`}
          className="inline-flex items-center gap-1 text-sm text-[var(--color-accent-brown)] hover:underline"
          aria-label={`Open orders for ${customer.fullName ?? customer.username ?? customer.email}`}
        >
          Orders
          <ExternalLink className="h-3.5 w-3" />
        </Link>
      </td>
    </tr>
  );
}

export function CustomersTableCard({
  isLoading,
  isError,
  errorTitle,
  errorMessage,
  onRetry,
  customersLength,
  filteredLength,
  pagedCustomers,
  orderStats,
  selectedCustomer,
  setSelectedCustomer,
  handleSort,
  sortIconFor,
  sortKey,
  sortDir,
  page,
  pageSize,
  setPage,
}: CustomersTableCardProps) {
  const ariaSortFor = (column: SortKey): "ascending" | "descending" | "none" =>
    sortKey !== column ? "none" : sortDir === "asc" ? "ascending" : "descending";

  return (
    <Card className="mt-6 rounded-xl border-[var(--color-line)] border-l-4 border-l-amber-500 bg-white shadow-[var(--admin-card-shadow)]">
      <CardTitle className="flex items-center gap-2 text-[var(--color-muted)]">
        <Users className="h-4 w-4 text-amber-500" />
        Customers
      </CardTitle>
      <CardContent className="mt-3">
        {isLoading && <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading customers...</p>}
        {isError && (
          <div className="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
            <p className="font-medium">{errorTitle ?? "Could not load customers."}</p>
            <p className="mt-1 text-xs">{errorMessage ?? "Please try again."}</p>
            <Button variant="outline" size="sm" className="mt-2" onClick={onRetry}>
              Try again
            </Button>
          </div>
        )}
        {!isLoading && !isError && customersLength === 0 && (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">No customers yet.</p>
        )}
        {!isLoading && !isError && customersLength > 0 && filteredLength === 0 && (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">
            No customers match the current filters.
          </p>
        )}
        {!isLoading && !isError && filteredLength > 0 && (
          <div className="space-y-4">
            <div className="overflow-x-auto">
              <table className="w-full min-w-[700px] border-collapse text-sm">
                <caption className="sr-only">
                  Customer list with sortable columns and quick actions
                </caption>
                <thead>
                  <tr className="border-b border-[var(--color-line)] text-left text-[var(--color-muted)]">
                    <th className="pb-2 pr-4 font-medium" aria-sort={ariaSortFor("name")}>
                      <SortHeader
                        label="Name"
                        column="name"
                        onSort={handleSort}
                        icon={sortIconFor("name")}
                      />
                    </th>
                    <th className="pb-2 pr-4 font-medium" aria-sort={ariaSortFor("email")}>
                      <SortHeader
                        label="Email"
                        column="email"
                        onSort={handleSort}
                        icon={sortIconFor("email")}
                      />
                    </th>
                    <th className="pb-2 pr-4 font-medium">User ID</th>
                    <th className="pb-2 pr-4 font-medium">Auth</th>
                    <th className="pb-2 pr-4 font-medium" aria-sort={ariaSortFor("orders")}>
                      <SortHeader
                        label="Orders"
                        column="orders"
                        onSort={handleSort}
                        icon={sortIconFor("orders")}
                      />
                    </th>
                    <th className="pb-2 pr-4 font-medium" aria-sort={ariaSortFor("spent")}>
                      <SortHeader
                        label="Spent"
                        column="spent"
                        onSort={handleSort}
                        icon={sortIconFor("spent")}
                      />
                    </th>
                    <th className="pb-2 pr-4 font-medium" aria-sort={ariaSortFor("created")}>
                      <SortHeader
                        label="Created"
                        column="created"
                        onSort={handleSort}
                        icon={sortIconFor("created")}
                      />
                    </th>
                    <th className="pb-2 font-medium">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {pagedCustomers.map((c) => (
                    <CustomerRow
                      key={c.userId}
                      customer={c}
                      selectedCustomer={selectedCustomer}
                      setSelectedCustomer={setSelectedCustomer}
                      stats={orderStats.get(c.userId)}
                    />
                  ))}
                </tbody>
              </table>
            </div>
            <div className="flex items-center justify-between gap-3">
              <p className="text-xs text-[var(--color-muted)]">
                Page {page} . showing up to {pageSize} rows
              </p>
              <div className="flex gap-2">
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  disabled={page <= 1 || isLoading}
                  onClick={() => setPage((p) => Math.max(1, p - 1))}
                >
                  Previous
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  disabled={pagedCustomers.length < pageSize || isLoading}
                  onClick={() => setPage((p) => p + 1)}
                >
                  Next
                </Button>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
