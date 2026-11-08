"use client";

import Link from "next/link";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { CustomerListRow } from "@/lib/admin-queries";
import { ExternalLink, Mail, Phone, Chrome, Users } from "lucide-react";
import { formatCreateDate, formatCurrency } from "@/domains/admin/customers/utils";
import { AdminTableCard } from "@/components/admin/admin-cards";

function AuthBadge({ provider }: { provider: string }) {
  const key = provider.trim().toLowerCase();
  const Icon = key === "google" ? Chrome : key === "phone" ? Phone : Mail;
  return (
    <span className="inline-flex items-center gap-1.5 text-[var(--color-muted)]">
      <Icon className="h-3.5 w-3.5 shrink-0" />
      <span className="capitalize">{provider}</span>
    </span>
  );
}

/** Only rendered for a non-active status — an active/never-set account shows nothing here,
 * keeping the common case visually unchanged. */
function StatusBadge({ status }: { status: string | null }) {
  const key = (status ?? "").trim().toLowerCase();
  if (key !== "inactive" && key !== "suspended") return null;
  return (
    <span className="ml-2 inline-flex items-center rounded-full border border-red-200 bg-red-50 px-2 py-0.5 text-xs font-medium capitalize text-red-700">
      {key}
    </span>
  );
}

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
      className="flex items-center font-semibold hover:text-[var(--color-ink)]"
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
        "border-b border-[var(--color-line)] last:border-0 hover:bg-[var(--color-surface-soft)]",
        selectedCustomer?.userId === customer.userId && "bg-[var(--color-surface-soft)]"
      )}
    >
      <td className="py-4 pr-4 font-medium text-[var(--color-ink)]">
        {customer.fullName ?? customer.username ?? "-"}
        <StatusBadge status={customer.userStatus} />
      </td>
      <td className="py-4 pr-4 text-[var(--color-ink)]">{customer.email}</td>
      <td className="py-4 pr-4 text-[var(--color-muted)]">{customer.userId}</td>
      <td className="py-4 pr-4"><AuthBadge provider={customer.authProvider} /></td>
      <td className="py-4 pr-4 text-[var(--color-ink)]">{stats?.count ?? 0}</td>
      <td className="py-4 pr-4 text-[var(--color-ink)]">{stats ? formatCurrency(stats.totalPaise) : "-"}</td>
      <td className="py-4 pr-4 text-[var(--color-muted)]">{formatCreateDate(customer.createDate)}</td>
      <td className="py-4">
        <div className="flex items-center gap-3">
          <Button
            type="button"
            variant="outline"
            onClick={() => setSelectedCustomer(customer)}
            aria-label={`View profile for ${customer.fullName ?? customer.username ?? customer.email}`}
          >
            View
          </Button>
          <Link
            href={`/imtheboss/orders?userId=${encodeURIComponent(customer.userId)}`}
            className="inline-flex items-center gap-1 text-[15px] text-[var(--color-green)] hover:underline"
            aria-label={`Open orders for ${customer.fullName ?? customer.username ?? customer.email}`}
          >
            Orders
            <ExternalLink className="h-3.5 w-3.5" />
          </Link>
        </div>
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
    <AdminTableCard title="Customers" icon={<Users className="h-4 w-4 text-[var(--color-green)]" />} className="mt-6">
      {isLoading ? <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading customers...</p> : null}

      {isError ? (
        <div className="rounded-md border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
          <p className="font-medium">{errorTitle ?? "Could not load customers."}</p>
          <p className="mt-1 text-xs">{errorMessage ?? "Please try again."}</p>
          <Button variant="outline" size="sm" className="mt-2" onClick={onRetry}>
            Try again
          </Button>
        </div>
      ) : null}

      {!isLoading && !isError && customersLength === 0 ? (
        <p className="py-8 text-center text-sm text-[var(--color-muted)]">No customers yet.</p>
      ) : null}

      {!isLoading && !isError && customersLength > 0 && filteredLength === 0 ? (
        <p className="py-8 text-center text-sm text-[var(--color-muted)]">No customers match current filters.</p>
      ) : null}

      {!isLoading && !isError && filteredLength > 0 ? (
        <div className="space-y-4">
          <div className="overflow-x-auto">
            <table className="w-full min-w-[760px] border-collapse text-[15px]">
              <caption className="sr-only">Customer list with sortable columns and quick actions</caption>
              <thead>
                <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                  <th className="pb-3 pr-4" aria-sort={ariaSortFor("name")}>
                    <SortHeader label="Name" column="name" onSort={handleSort} icon={sortIconFor("name")} />
                  </th>
                  <th className="pb-3 pr-4" aria-sort={ariaSortFor("email")}>
                    <SortHeader label="Email" column="email" onSort={handleSort} icon={sortIconFor("email")} />
                  </th>
                  <th className="pb-3 pr-4 font-semibold">Customer ID</th>
                  <th className="pb-3 pr-4 font-semibold">Signed in with</th>
                  <th className="pb-3 pr-4" aria-sort={ariaSortFor("orders")}>
                    <SortHeader label="Orders" column="orders" onSort={handleSort} icon={sortIconFor("orders")} />
                  </th>
                  <th className="pb-3 pr-4" aria-sort={ariaSortFor("spent")}>
                    <SortHeader label="Spent" column="spent" onSort={handleSort} icon={sortIconFor("spent")} />
                  </th>
                  <th className="pb-3 pr-4" aria-sort={ariaSortFor("created")}>
                    <SortHeader label="Joined" column="created" onSort={handleSort} icon={sortIconFor("created")} />
                  </th>
                  <th className="pb-3 font-semibold">Actions</th>
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
            <p className="text-sm text-[var(--color-muted)]">
              Page {page} - showing up to {pageSize} rows
            </p>
            <div className="flex gap-2">
              <Button type="button" variant="outline" disabled={page <= 1 || isLoading} onClick={() => setPage((p) => Math.max(1, p - 1))}>
                Previous
              </Button>
              <Button type="button" variant="outline" disabled={pagedCustomers.length < pageSize || isLoading} onClick={() => setPage((p) => p + 1)}>
                Next
              </Button>
            </div>
          </div>
        </div>
      ) : null}
    </AdminTableCard>
  );
}
