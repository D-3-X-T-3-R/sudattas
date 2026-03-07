"use client";

import { useMemo, useState } from "react";
import Link from "next/link";
import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Section } from "@/components/ui/section";
import { Kicker, SectionHeading } from "@/components/ui/typography";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import {
  fetchCustomersList,
  fetchOrdersList,
  type CustomerListRow,
  type OrderListRow,
} from "@/lib/admin-queries";
import { cn } from "@/lib/utils";
import { ArrowUpDown, ArrowDown, ArrowUp, Download, ExternalLink, User } from "lucide-react";

function formatCreateDate(createDate: string): string {
  try {
    const d = new Date(createDate);
    if (Number.isNaN(d.getTime())) return createDate;
    return d.toLocaleDateString("en-IN", {
      day: "2-digit",
      month: "short",
      year: "numeric",
    });
  } catch {
    return createDate;
  }
}

function formatCurrency(paise: number): string {
  return new Intl.NumberFormat("en-IN", {
    style: "currency",
    currency: "INR",
    maximumFractionDigits: 0,
  }).format(paise / 100);
}

type SortKey = "name" | "email" | "created" | "orders" | "spent";
type SortDir = "asc" | "desc";

function aggregateOrderStats(orders: OrderListRow[]): Map<string, { count: number; totalPaise: number }> {
  const map = new Map<string, { count: number; totalPaise: number }>();
  for (const o of orders) {
    const cur = map.get(o.userId) ?? { count: 0, totalPaise: 0 };
    cur.count += 1;
    cur.totalPaise += parseInt(o.totalAmountPaise, 10) || 0;
    map.set(o.userId, cur);
  }
  return map;
}

function downloadCsv(
  rows: CustomerListRow[],
  stats: Map<string, { count: number; totalPaise: number }>
): void {
  const headers = [
    "User ID",
    "Email",
    "Name",
    "Auth",
    "Address",
    "Phone",
    "Created",
    "Orders",
    "Total spent (₹)",
  ];
  const lines = [
    headers.join(","),
    ...rows.map((c) => {
      const s = stats.get(c.userId);
      const escaped = (v: string | null | undefined) =>
        v == null ? "" : `"${String(v).replace(/"/g, '""')}"`;
      return [
        c.userId,
        escaped(c.email),
        escaped(c.fullName ?? c.username),
        c.authProvider,
        escaped(c.address),
        escaped(c.phone),
        formatCreateDate(c.createDate),
        s?.count ?? 0,
        s ? (s.totalPaise / 100).toFixed(0) : "0",
      ].join(",");
    }),
  ];
  const blob = new Blob([lines.join("\n")], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `customers-${new Date().toISOString().slice(0, 10)}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}

export default function AdminCustomersPage() {
  const [searchQuery, setSearchQuery] = useState("");
  const [filterAuth, setFilterAuth] = useState("");
  const [sortKey, setSortKey] = useState<SortKey>("created");
  const [sortDir, setSortDir] = useState<SortDir>("desc");
  const [selectedCustomer, setSelectedCustomer] = useState<CustomerListRow | null>(null);

  const {
    data: customers = [],
    isLoading,
    isError,
    error,
    refetch,
  } = useQuery<CustomerListRow[], Error>({
    queryKey: ["admin", "customers"],
    queryFn: fetchCustomersList,
  });

  const { data: allOrders = [] } = useQuery<OrderListRow[], Error>({
    queryKey: ["admin", "orders", "all-for-stats"],
    queryFn: () => fetchOrdersList({ limit: "5000" }),
    enabled: !isError && customers.length > 0,
  });

  const orderStats = useMemo(() => aggregateOrderStats(allOrders), [allOrders]);

  const filteredAndSorted = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    const auth = filterAuth.trim();
    let list = customers.filter((c) => {
      if (auth && c.authProvider !== auth) return false;
      if (!q) return true;
      const name = (c.fullName ?? c.username ?? "").toLowerCase();
      const email = (c.email ?? "").toLowerCase();
      const uid = (c.userId ?? "").toLowerCase();
      return name.includes(q) || email.includes(q) || uid.includes(q);
    });
    list = [...list].sort((a, b) => {
      let cmp = 0;
      const sa = orderStats.get(a.userId);
      const sb = orderStats.get(b.userId);
      switch (sortKey) {
        case "name":
          cmp = (a.fullName ?? a.username ?? "").localeCompare(b.fullName ?? b.username ?? "");
          break;
        case "email":
          cmp = (a.email ?? "").localeCompare(b.email ?? "");
          break;
        case "created":
          cmp = new Date(a.createDate).getTime() - new Date(b.createDate).getTime();
          break;
        case "orders":
          cmp = (sa?.count ?? 0) - (sb?.count ?? 0);
          break;
        case "spent":
          cmp = (sa?.totalPaise ?? 0) - (sb?.totalPaise ?? 0);
          break;
        default:
          break;
      }
      return sortDir === "asc" ? cmp : -cmp;
    });
    return list;
  }, [customers, searchQuery, filterAuth, sortKey, sortDir, orderStats]);

  const handleSort = (key: SortKey) => {
    if (sortKey === key) setSortDir((d) => (d === "asc" ? "desc" : "asc"));
    else {
      setSortKey(key);
      setSortDir(key === "name" || key === "email" ? "asc" : "desc");
    }
  };

  const SortIcon = ({ column }: { column: SortKey }) =>
    sortKey !== column ? (
      <ArrowUpDown className="ml-1 inline h-3.5 w-3 opacity-50" />
    ) : sortDir === "asc" ? (
      <ArrowUp className="ml-1 inline h-3.5 w-3" />
    ) : (
      <ArrowDown className="ml-1 inline h-3.5 w-3" />
    );

  return (
    <Section compact className="max-w-5xl">
      <Kicker className="text-[var(--color-muted)]">Customers</Kicker>
      <SectionHeading size="default" className="mt-2">
        Customer list
      </SectionHeading>

      <Card className="mt-8 border-[var(--color-line)]">
        <CardTitle className="text-[var(--color-muted)]">Filters</CardTitle>
        <CardContent className="mt-3 flex flex-wrap items-end gap-3">
          <div>
            <label htmlFor="customers-search" className="mb-1 block text-xs text-[var(--color-muted)]">
              Search
            </label>
            <Input
              id="customers-search"
              type="text"
              placeholder="Email, name, or user ID"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="h-9 w-56 rounded-md"
            />
          </div>
          <div>
            <label htmlFor="customers-auth" className="mb-1 block text-xs text-[var(--color-muted)]">
              Auth
            </label>
            <select
              id="customers-auth"
              className={cn(
                "h-9 min-w-[8rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              )}
              value={filterAuth}
              onChange={(e) => setFilterAuth(e.target.value)}
            >
              <option value="">All</option>
              <option value="email">Email</option>
              <option value="google">Google</option>
            </select>
          </div>
          <Button type="button" variant="outline" size="sm" onClick={() => refetch()}>
            Refresh
          </Button>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => downloadCsv(filteredAndSorted, orderStats)}
            disabled={filteredAndSorted.length === 0}
          >
            <Download className="mr-1.5 h-4 w-4" />
            Export CSV
          </Button>
        </CardContent>
      </Card>

      <Card className="mt-6 border-[var(--color-line)]">
        <CardTitle className="text-[var(--color-muted)]">Customers</CardTitle>
        <CardContent className="mt-3">
          {isLoading && (
            <p className="py-8 text-center text-sm text-[var(--color-muted)]">
              Loading customers…
            </p>
          )}
          {isError && (
            <div className="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
              <p className="font-medium">Could not load customers.</p>
              <p className="mt-1 text-xs">{error?.message ?? "Unknown error"}</p>
              <Button variant="outline" size="sm" className="mt-2" onClick={() => refetch()}>
                Try again
              </Button>
            </div>
          )}
          {!isLoading && !isError && customers.length === 0 && (
            <p className="py-8 text-center text-sm text-[var(--color-muted)]">No customers yet.</p>
          )}
          {!isLoading && !isError && customers.length > 0 && filteredAndSorted.length === 0 && (
            <p className="py-8 text-center text-sm text-[var(--color-muted)]">
              No customers match the current filters.
            </p>
          )}
          {!isLoading && !isError && filteredAndSorted.length > 0 && (
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
                        <SortIcon column="name" />
                      </button>
                    </th>
                    <th className="pb-2 pr-4 font-medium">
                      <button
                        type="button"
                        className="flex items-center font-medium hover:text-[var(--color-ink)]"
                        onClick={() => handleSort("email")}
                      >
                        Email
                        <SortIcon column="email" />
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
                        <SortIcon column="orders" />
                      </button>
                    </th>
                    <th className="pb-2 pr-4 font-medium">
                      <button
                        type="button"
                        className="flex items-center font-medium hover:text-[var(--color-ink)]"
                        onClick={() => handleSort("spent")}
                      >
                        Spent
                        <SortIcon column="spent" />
                      </button>
                    </th>
                    <th className="pb-2 pr-4 font-medium">
                      <button
                        type="button"
                        className="flex items-center font-medium hover:text-[var(--color-ink)]"
                        onClick={() => handleSort("created")}
                      >
                        Created
                        <SortIcon column="created" />
                      </button>
                    </th>
                    <th className="pb-2 font-medium">Actions</th>
                  </tr>
                </thead>
                <tbody>
                  {filteredAndSorted.map((c) => {
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
                        <td className="py-3 pr-4 text-[var(--color-ink)]">
                          {c.fullName ?? c.username ?? "—"}
                        </td>
                        <td className="py-3 pr-4 text-[var(--color-ink)]">{c.email}</td>
                        <td className="py-3 pr-4 font-mono text-[var(--color-ink)]">{c.userId}</td>
                        <td className="py-3 pr-4 text-[var(--color-muted)]">{c.authProvider}</td>
                        <td className="py-3 pr-4 text-[var(--color-ink)]">
                          {stats?.count ?? 0}
                        </td>
                        <td className="py-3 pr-4 text-[var(--color-ink)]">
                          {stats ? formatCurrency(stats.totalPaise) : "—"}
                        </td>
                        <td className="py-3 pr-4 text-[var(--color-muted)]">
                          {formatCreateDate(c.createDate)}
                        </td>
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
          )}
        </CardContent>
      </Card>

      {selectedCustomer && (
        <Dialog open={!!selectedCustomer} onOpenChange={(open) => !open && setSelectedCustomer(null)}>
          <DialogContent title="Customer profile" className="sm:max-w-md">
            <div className="space-y-4 text-sm">
              <div className="flex items-center gap-3 border-b border-[var(--color-line)] pb-3">
                <div className="flex h-12 w-12 items-center justify-center rounded-full bg-[var(--color-line)]/40">
                  <User className="h-6 w-6 text-[var(--color-muted)]" />
                </div>
                <div>
                  <p className="font-medium text-[var(--color-ink)]">
                    {selectedCustomer.fullName ?? selectedCustomer.username ?? "—"}
                  </p>
                  <p className="text-[var(--color-muted)]">{selectedCustomer.email}</p>
                </div>
              </div>
              <dl className="grid gap-2">
                <div>
                  <dt className="text-xs text-[var(--color-muted)]">User ID</dt>
                  <dd className="font-mono text-[var(--color-ink)]">{selectedCustomer.userId}</dd>
                </div>
                <div>
                  <dt className="text-xs text-[var(--color-muted)]">Auth</dt>
                  <dd className="text-[var(--color-ink)]">{selectedCustomer.authProvider}</dd>
                </div>
                <div>
                  <dt className="text-xs text-[var(--color-muted)]">Created</dt>
                  <dd className="text-[var(--color-ink)]">
                    {formatCreateDate(selectedCustomer.createDate)}
                  </dd>
                </div>
                {selectedCustomer.address && (
                  <div>
                    <dt className="text-xs text-[var(--color-muted)]">Address</dt>
                    <dd className="text-[var(--color-ink)]">{selectedCustomer.address}</dd>
                  </div>
                )}
                {selectedCustomer.phone && (
                  <div>
                    <dt className="text-xs text-[var(--color-muted)]">Phone</dt>
                    <dd className="text-[var(--color-ink)]">{selectedCustomer.phone}</dd>
                  </div>
                )}
              </dl>
              <div className="flex gap-4 rounded-lg bg-[var(--color-surface)] p-3">
                <div>
                  <p className="text-xs text-[var(--color-muted)]">Orders</p>
                  <p className="text-lg font-medium text-[var(--color-ink)]">
                    {orderStats.get(selectedCustomer.userId)?.count ?? 0}
                  </p>
                </div>
                <div>
                  <p className="text-xs text-[var(--color-muted)]">Total spent</p>
                  <p className="text-lg font-medium text-[var(--color-ink)]">
                    {orderStats.get(selectedCustomer.userId)
                      ? formatCurrency(orderStats.get(selectedCustomer.userId)!.totalPaise)
                      : "—"}
                  </p>
                </div>
              </div>
              <div className="flex gap-2 pt-2">
                <Button asChild size="sm" className="flex-1">
                  <Link href={`/imtheboss/orders?userId=${encodeURIComponent(selectedCustomer.userId)}`}>
                    <ExternalLink className="mr-1.5 h-4 w-4" />
                    View orders
                  </Link>
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => setSelectedCustomer(null)}
                >
                  Close
                </Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
      )}
    </Section>
  );
}
