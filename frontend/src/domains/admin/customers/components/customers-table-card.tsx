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
  page: number;
  pageSize: number;
  setPage: (updater: (prev: number) => number) => void;
};

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
  page,
  pageSize,
  setPage,
}: CustomersTableCardProps) {
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
                <thead>
                  <tr className="border-b border-[var(--color-line)] text-left text-[var(--color-muted)]">
                    <th className="pb-2 pr-4 font-medium">
                      <button
                        type="button"
                        className="flex items-center font-medium hover:text-[var(--color-ink)]"
                        onClick={() => handleSort("name")}
                      >
                        Name
                        {sortIconFor("name")}
                      </button>
                    </th>
                    <th className="pb-2 pr-4 font-medium">
                      <button
                        type="button"
                        className="flex items-center font-medium hover:text-[var(--color-ink)]"
                        onClick={() => handleSort("email")}
                      >
                        Email
                        {sortIconFor("email")}
                      </button>
                    </th>
                    <th className="pb-2 pr-4 font-medium">User ID</th>
                    <th className="pb-2 pr-4 font-medium">Auth</th>
                    <th className="pb-2 pr-4 font-medium">
                      <button
                        type="button"
                        className="flex items-center font-medium hover:text-[var(--color-ink)]"
                        onClick={() => handleSort("orders")}
                      >
                        Orders
                        {sortIconFor("orders")}
                      </button>
                    </th>
                    <th className="pb-2 pr-4 font-medium">
                      <button
                        type="button"
                        className="flex items-center font-medium hover:text-[var(--color-ink)]"
                        onClick={() => handleSort("spent")}
                      >
                        Spent
                        {sortIconFor("spent")}
                      </button>
                    </th>
                    <th className="pb-2 pr-4 font-medium">
                      <button
                        type="button"
                        className="flex items-center font-medium hover:text-[var(--color-ink)]"
                        onClick={() => handleSort("created")}
                      >
                        Created
                        {sortIconFor("created")}
                      </button>
                    </th>
                    <th className="pb-2 font-medium">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {pagedCustomers.map((c) => {
                    const stats = orderStats.get(c.userId);
                    return (
                      <tr
                        key={c.userId}
                        className={cn(
                          "border-b border-[var(--color-line)] last:border-0 hover:bg-[var(--color-surface)]",
                          selectedCustomer?.userId === c.userId && "bg-[var(--color-line)]/20"
                        )}
                        onClick={() => setSelectedCustomer(c)}
                        role="button"
                        tabIndex={0}
                        onKeyDown={(e) => {
                          if (e.key === "Enter" || e.key === " ") {
                            e.preventDefault();
                            setSelectedCustomer(c);
                          }
                        }}
                      >
                        <td className="py-3 pr-4 text-[var(--color-ink)]">{c.fullName ?? c.username ?? "-"}</td>
                        <td className="py-3 pr-4 text-[var(--color-ink)]">{c.email}</td>
                        <td className="py-3 pr-4 font-mono text-[var(--color-ink)]">{c.userId}</td>
                        <td className="py-3 pr-4 text-[var(--color-muted)]">{c.authProvider}</td>
                        <td className="py-3 pr-4 text-[var(--color-ink)]">{stats?.count ?? 0}</td>
                        <td className="py-3 pr-4 text-[var(--color-ink)]">
                          {stats ? formatCurrency(stats.totalPaise) : "-"}
                        </td>
                        <td className="py-3 pr-4 text-[var(--color-muted)]">{formatCreateDate(c.createDate)}</td>
                        <td className="py-3">
                          <Link
                            href={`/imtheboss/orders?userId=${encodeURIComponent(c.userId)}`}
                            className="inline-flex items-center gap-1 text-sm text-[var(--color-accent-brown)] hover:underline"
                            onClick={(e) => e.stopPropagation()}
                          >
                            Orders
                            <ExternalLink className="h-3.5 w-3" />
                          </Link>
                        </td>
                      </tr>
                    );
                  })}
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
