"use client";

import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { ResponsivePie } from "@nivo/pie";
import { Card, CardContent } from "@/components/ui/card";
import {
  fetchAllOrdersList,
  fetchAllCustomersList,
  fetchAllProductsSummary,
  fetchOrderStatuses,
  fetchCategories,
} from "@/lib/admin-queries";

/**
 * Validated categorical palette (dataviz skill, `references/palette.md`), fixed order.
 * Assigned in sequence per chart — never cycled, never re-keyed to a specific category.
 */
const CATEGORICAL = ["#2a78d6", "#008300", "#e87ba4", "#eda100", "#1baf7a", "#eb6834", "#4a3aa7", "#e34948"];
const OTHER_GRAY = "#c3c2b7";

export interface Slice {
  id: string;
  label: string;
  value: number;
  color: string;
}

/** Sort descending, assign categorical slots 1..N in that order, fold anything past `cap` into "Other". */
function toSlices(counts: Map<string, number>, cap = 6): Slice[] {
  const sorted = [...counts.entries()].filter(([, v]) => v > 0).sort((a, b) => b[1] - a[1]);
  const head = sorted.slice(0, cap);
  const tail = sorted.slice(cap);
  const slices: Slice[] = head.map(([label, value], i) => ({
    id: label,
    label,
    value,
    color: CATEGORICAL[i % CATEGORICAL.length],
  }));
  const otherTotal = tail.reduce((s, [, v]) => s + v, 0);
  if (otherTotal > 0) {
    slices.push({ id: "Other", label: "Other", value: otherTotal, color: OTHER_GRAY });
  }
  return slices;
}

function useOrdersByStatus() {
  const { data: allOrders = [] } = useQuery({
    queryKey: ["admin", "orders", "all-for-stats"],
    queryFn: () => fetchAllOrdersList(),
    staleTime: 2 * 60 * 1000,
  });
  const { data: statuses = [] } = useQuery({
    queryKey: ["admin", "order-statuses"],
    queryFn: fetchOrderStatuses,
  });

  return useMemo(() => {
    const idToName = new Map(statuses.map((s) => [s.statusId, s.statusName.trim().toLowerCase()]));
    const buckets = new Map<string, number>([
      ["Pending", 0],
      ["In progress", 0],
      ["Delivered", 0],
      ["Cancelled", 0],
      ["Refunded", 0],
    ]);
    for (const o of allOrders) {
      const name = idToName.get(o.statusId) ?? "";
      if (name === "pending" || name === "needs_review") buckets.set("Pending", (buckets.get("Pending") ?? 0) + 1);
      else if (name === "confirmed" || name === "processing" || name === "shipped")
        buckets.set("In progress", (buckets.get("In progress") ?? 0) + 1);
      else if (name === "delivered") buckets.set("Delivered", (buckets.get("Delivered") ?? 0) + 1);
      else if (name === "cancelled") buckets.set("Cancelled", (buckets.get("Cancelled") ?? 0) + 1);
      else if (name === "refunded") buckets.set("Refunded", (buckets.get("Refunded") ?? 0) + 1);
    }
    return toSlices(buckets, 6);
  }, [allOrders, statuses]);
}

function useCustomersByAuth() {
  const { data: customers = [] } = useQuery({
    queryKey: ["admin", "customers"],
    queryFn: () => fetchAllCustomersList(),
    staleTime: 2 * 60 * 1000,
  });

  return useMemo(() => {
    const counts = new Map<string, number>();
    for (const c of customers) {
      const key = (c.authProvider || "other").trim();
      const label = key.charAt(0).toUpperCase() + key.slice(1);
      counts.set(label, (counts.get(label) ?? 0) + 1);
    }
    return toSlices(counts, 6);
  }, [customers]);
}

function useProductsByCategory() {
  const { data: productsSummary = [] } = useQuery({
    queryKey: ["admin", "products-summary"],
    queryFn: fetchAllProductsSummary,
    staleTime: 2 * 60 * 1000,
  });
  const { data: categories = [] } = useQuery({
    queryKey: ["admin", "categories"],
    queryFn: fetchCategories,
    staleTime: 5 * 60 * 1000,
  });

  return useMemo(() => {
    const idToName = new Map(categories.map((c) => [c.categoryId, c.name]));
    const counts = new Map<string, number>();
    for (const p of productsSummary) {
      const isArchived = (p.productStatusId ?? "").trim() === "3";
      if (isArchived) continue;
      const label = (p.categoryId ? idToName.get(p.categoryId) : null) ?? "Uncategorized";
      counts.set(label, (counts.get(label) ?? 0) + 1);
    }
    return toSlices(counts, 5);
  }, [productsSummary, categories]);
}

function CenteredMetric({
  dataWithArc,
  centerX,
  centerY,
}: {
  dataWithArc: readonly { value: number }[];
  centerX: number;
  centerY: number;
}) {
  const total = dataWithArc.reduce((s, d) => s + d.value, 0);
  return (
    <g>
      <text
        x={centerX}
        y={centerY - 6}
        textAnchor="middle"
        dominantBaseline="central"
        style={{ fontSize: 26, fontWeight: 600, fill: "var(--color-ink)" }}
      >
        {total}
      </text>
      <text
        x={centerX}
        y={centerY + 16}
        textAnchor="middle"
        dominantBaseline="central"
        style={{ fontSize: 12, fill: "var(--color-muted)" }}
      >
        total
      </text>
    </g>
  );
}

function DonutLegend({ slices, total }: { slices: Slice[]; total: number }) {
  return (
    <ul className="mt-4 space-y-2">
      {slices.map((s) => {
        const pct = total > 0 ? Math.round((s.value / total) * 100) : 0;
        return (
          <li key={s.id} className="flex items-center justify-between gap-3 text-[15px]">
            <span className="flex min-w-0 items-center gap-2.5">
              <span
                className="h-2.5 w-2.5 shrink-0 rounded-full"
                style={{ backgroundColor: s.color }}
                aria-hidden="true"
              />
              <span className="truncate text-[var(--color-ink)]">{s.label}</span>
            </span>
            <span className="shrink-0 text-[var(--color-muted)]">
              {s.value} <span className="text-[var(--color-ink)]">({pct}%)</span>
            </span>
          </li>
        );
      })}
    </ul>
  );
}

export function DonutCard({ title, subtitle, slices }: { title: string; subtitle: string; slices: Slice[] }) {
  const total = useMemo(() => slices.reduce((s, d) => s + d.value, 0), [slices]);

  return (
    <Card className="overflow-hidden rounded-2xl border-[var(--color-line)] bg-white shadow-[var(--admin-card-shadow)]">
      <div className="p-5 pb-0">
        <p className="text-[15px] font-semibold text-[var(--color-ink)]">{title}</p>
        <p className="mt-0.5 text-sm text-[var(--color-muted)]">{subtitle}</p>
      </div>
      <CardContent className="mt-2 pt-2">
        {total === 0 ? (
          <p className="py-10 text-center text-sm text-[var(--color-muted)]">Not enough data yet.</p>
        ) : (
          <>
            <div style={{ height: 220 }}>
              <ResponsivePie
                data={slices}
                margin={{ top: 8, right: 8, bottom: 8, left: 8 }}
                innerRadius={0.68}
                padAngle={2}
                cornerRadius={3}
                colors={{ datum: "data.color" }}
                borderWidth={0}
                enableArcLinkLabels={false}
                enableArcLabels={false}
                isInteractive
                animate
                motionConfig="gentle"
                layers={["arcs", CenteredMetric]}
                tooltip={({ datum }) => {
                  const pct = total > 0 ? Math.round((datum.value / total) * 100) : 0;
                  return (
                    <div className="rounded-lg border border-[var(--color-line)] bg-white px-3.5 py-2.5 shadow-[0_8px_24px_rgba(45,42,38,0.12)]">
                      <p className="text-base font-semibold text-[var(--color-ink)]">
                        {datum.value} <span className="font-normal text-[var(--color-muted)]">({pct}%)</span>
                      </p>
                      <p className="text-sm text-[var(--color-muted)]">{String(datum.label)}</p>
                    </div>
                  );
                }}
              />
            </div>
            <DonutLegend slices={slices} total={total} />
          </>
        )}
      </CardContent>
    </Card>
  );
}

/** Part-to-whole breakdowns — order status, how customers sign in, and product mix by category. */
export function DashboardDonuts() {
  const ordersByStatus = useOrdersByStatus();
  const customersByAuth = useCustomersByAuth();
  const productsByCategory = useProductsByCategory();

  return (
    <div className="mt-10 space-y-4">
      <div>
        <p className="text-base font-semibold text-[var(--color-ink)]">Breakdown</p>
        <p className="mt-1 text-sm text-[var(--color-muted)]">How orders, customers, and products split up.</p>
      </div>
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
        <DonutCard title="Orders by status" subtitle="Where every order stands right now" slices={ordersByStatus} />
        <DonutCard title="How customers sign in" subtitle="Sign-in method across all customers" slices={customersByAuth} />
        <DonutCard title="Products by category" subtitle="Active catalogue, grouped by category" slices={productsByCategory} />
      </div>
    </div>
  );
}
