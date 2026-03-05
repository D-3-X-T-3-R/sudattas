"use client";

import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import {
  fetchDashboardStats,
  type DashboardStats,
} from "@/lib/admin-queries";

const STATS_CONFIG: Array<{
  key: keyof DashboardStats;
  label: string;
  sub: string;
  format: (stats: DashboardStats) => string;
}> = [
  {
    key: "ordersToday",
    label: "Orders today",
    sub: "Orders placed today.",
    format: (s) => String(s.ordersToday),
  },
  {
    key: "revenueMtdFormatted",
    label: "Revenue (MTD)",
    sub: "Month-to-date from completed orders.",
    format: (s) => s.revenueMtdFormatted,
  },
  {
    key: "productsCount",
    label: "Products",
    sub: "Total products in catalog.",
    format: (s) => String(s.productsCount),
  },
  {
    key: "customersCount",
    label: "Customers",
    sub: "Total registered users.",
    format: (s) => (s.customersCount != null ? String(s.customersCount) : "—"),
  },
];

export function DashboardStats() {
  const { data: stats, isLoading, isError, error } = useQuery({
    queryKey: ["admin", "dashboard-stats"],
    queryFn: fetchDashboardStats,
    staleTime: 60 * 1000,
  });

  if (isLoading) {
    return (
      <div className="mt-8 grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {STATS_CONFIG.map(({ label }) => (
          <Card key={label} className="border-[var(--color-line)]">
            <CardTitle className="text-[var(--color-muted)]">{label}</CardTitle>
            <CardContent className="mt-2">
              <div className="font-display text-2xl font-medium tracking-tight text-[var(--color-muted)]">
                …
              </div>
              <div className="mt-1.5 text-xs leading-relaxed text-[var(--color-muted)]">
                Loading…
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  if (isError || !stats) {
    return (
      <div className="mt-8 rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
        <p className="font-medium">Could not load dashboard stats.</p>
        <p className="mt-1 text-xs">{error?.message ?? "Unknown error"}</p>
      </div>
    );
  }

  return (
    <div className="mt-8 grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      {STATS_CONFIG.map(({ label, sub, format }) => (
        <Card key={label} className="border-[var(--color-line)]">
          <CardTitle className="text-[var(--color-muted)]">{label}</CardTitle>
          <CardContent className="mt-2">
            <div className="font-display text-2xl font-medium tracking-tight text-[var(--color-ink)]">
              {format(stats)}
            </div>
            <div className="mt-1.5 text-xs leading-relaxed text-[var(--color-muted)]">
              {sub}
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
