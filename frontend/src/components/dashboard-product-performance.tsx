"use client";

import { useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import Link from "next/link";
import { ResponsiveBar } from "@nivo/bar";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { AdminFilterCard } from "@/components/admin/admin-cards";
import {
  fetchOrderLinesByDateRange,
  fetchOrderStatuses,
  fetchAllProductsSummary,
  fetchCategories,
  lastNDaysRange,
} from "@/lib/admin-queries";
import { formatInrFromPaise } from "@/lib/money";
import { Filter, Flame, Boxes } from "lucide-react";

// Chart chrome tokens — shared with dashboard-charts.tsx's mark specs (hairline grid, muted axis).
const GRID_LINE = "#e1e0d9";
const AXIS_MUTED = "#898781";
const chartTheme = {
  axis: { ticks: { text: { fill: AXIS_MUTED, fontSize: 13 } } },
  grid: { line: { stroke: GRID_LINE, strokeWidth: 1 } },
};

function truncateLabel(label: string, max = 28): string {
  return label.length > max ? `${label.slice(0, max - 1)}…` : label;
}

type RangeKey = "7" | "30" | "90" | "365";
const RANGE_OPTIONS: { key: RangeKey; label: string; days: number }[] = [
  { key: "7", label: "Last 7 days", days: 7 },
  { key: "30", label: "Last 30 days", days: 30 },
  { key: "90", label: "Last 90 days", days: 90 },
  { key: "365", label: "Last year", days: 365 },
];

export type HBarDatum = { label: string; value: number };

export function HorizontalBarChart({
  data,
  valueFormat,
  barColor,
}: {
  data: HBarDatum[];
  valueFormat: (v: number) => string;
  barColor: string;
}) {
  const maxVal = useMemo(() => Math.max(1, ...data.map((d) => d.value)), [data]);
  const height = Math.max(200, data.length * 42 + 24);

  return (
    <div style={{ height, width: "100%" }}>
      <ResponsiveBar
        data={data}
        indexBy="label"
        keys={["value"]}
        layout="horizontal"
        valueScale={{ type: "linear", min: 0, max: maxVal, clamp: true }}
        margin={{ top: 4, right: 36, bottom: 28, left: 152 }}
        padding={0.35}
        theme={chartTheme}
        colors={[barColor]}
        borderRadius={4}
        enableLabel={false}
        enableGridX
        enableGridY={false}
        gridXValues={4}
        axisBottom={{ tickSize: 0, tickPadding: 8, tickValues: 4, format: (v) => valueFormat(Number(v)) }}
        axisLeft={{ tickSize: 0, tickPadding: 10 }}
        valueFormat={valueFormat}
        tooltip={({ indexValue, value }) => (
          <div className="rounded-lg border border-[var(--color-line)] bg-white px-3.5 py-2.5 shadow-[0_8px_24px_rgba(45,42,38,0.12)]">
            <p className="text-base font-semibold text-[var(--color-ink)]">{valueFormat(Number(value))}</p>
            <p className="text-sm text-[var(--color-muted)]">{String(indexValue)}</p>
          </div>
        )}
      />
    </div>
  );
}

function ChartCard({ title, subtitle, children }: { title: string; subtitle: string; children: React.ReactNode }) {
  return (
    <Card className="overflow-hidden rounded-2xl border-[var(--color-line)] bg-white shadow-[var(--admin-card-shadow)]">
      <div className="p-5 pb-1">
        <p className="text-[15px] font-semibold text-[var(--color-ink)]">{title}</p>
        <p className="mt-0.5 text-sm text-[var(--color-muted)]">{subtitle}</p>
      </div>
      <CardContent className="mt-2 pt-2">{children}</CardContent>
    </Card>
  );
}

function useProductPerformance(range: { orderDateStart: string; orderDateEnd: string }) {
  const linesQuery = useQuery({
    queryKey: ["admin", "order-lines", range],
    queryFn: () => fetchOrderLinesByDateRange(range),
    staleTime: 2 * 60 * 1000,
  });
  const { data: statuses = [] } = useQuery({
    queryKey: ["admin", "order-statuses"],
    queryFn: fetchOrderStatuses,
  });
  const { data: categories = [] } = useQuery({
    queryKey: ["admin", "categories"],
    queryFn: fetchCategories,
    staleTime: 5 * 60 * 1000,
  });

  const aggregates = useMemo(() => {
    const lines = linesQuery.data ?? [];
    const idToStatusName = new Map(statuses.map((s) => [s.statusId, s.statusName.trim().toLowerCase()]));
    const idToCategoryName = new Map(categories.map((c) => [c.categoryId, c.name]));
    const validLines = lines.filter((l) => idToStatusName.get(l.statusId) !== "cancelled");

    const unitsByProduct = new Map<string, HBarDatum>();
    const revenueByProduct = new Map<string, HBarDatum>();
    const revenueByCategory = new Map<string, number>();

    for (const l of validLines) {
      const key = l.productId ?? l.productName ?? "unknown";
      const label = truncateLabel(l.productName ?? "Unknown product");

      const u = unitsByProduct.get(key) ?? { label, value: 0 };
      u.value += l.quantity;
      unitsByProduct.set(key, u);

      const r = revenueByProduct.get(key) ?? { label, value: 0 };
      r.value += l.linePricePaise;
      revenueByProduct.set(key, r);

      const catLabel = (l.categoryId ? idToCategoryName.get(l.categoryId) : null) ?? "Uncategorized";
      revenueByCategory.set(catLabel, (revenueByCategory.get(catLabel) ?? 0) + l.linePricePaise);
    }

    const topUnits = [...unitsByProduct.values()].sort((a, b) => b.value - a.value).slice(0, 8);
    const topRevenue = [...revenueByProduct.values()].sort((a, b) => b.value - a.value).slice(0, 8);
    const topCategories = [...revenueByCategory.entries()]
      .map(([label, value]) => ({ label, value }))
      .sort((a, b) => b.value - a.value)
      .slice(0, 6);

    return { topUnits, topRevenue, topCategories };
  }, [linesQuery.data, statuses, categories]);

  return { ...aggregates, isLoading: linesQuery.isLoading, isError: linesQuery.isError };
}

function useRestockingAndMovers(topUnits: HBarDatum[]) {
  const { data: productsSummary = [] } = useQuery({
    queryKey: ["admin", "products-summary"],
    queryFn: fetchAllProductsSummary,
    staleTime: 2 * 60 * 1000,
  });

  const needsRestocking = useMemo(() => {
    return productsSummary
      .filter((p) => (p.productStatusId ?? "").trim() !== "3" && p.stockQuantity != null)
      .sort((a, b) => (a.stockQuantity ?? 0) - (b.stockQuantity ?? 0))
      .slice(0, 5);
  }, [productsSummary]);

  const fastMovers = topUnits.slice(0, 5);

  return { needsRestocking, fastMovers };
}

function ListCard({
  title,
  icon,
  emptyText,
  rows,
}: {
  title: string;
  icon: React.ReactNode;
  emptyText: string;
  rows: { key: string; label: string; value: string; href?: string }[];
}) {
  return (
    <Card className="rounded-2xl border-[var(--color-line)] bg-white shadow-[var(--admin-card-shadow)]">
      <div className="flex items-center gap-2.5 p-5 pb-1">
        {icon}
        <p className="text-[15px] font-semibold text-[var(--color-ink)]">{title}</p>
      </div>
      <CardContent className="mt-2 pt-2">
        {rows.length === 0 ? (
          <p className="py-6 text-center text-sm text-[var(--color-muted)]">{emptyText}</p>
        ) : (
          <ul className="divide-y divide-[var(--color-line)]">
            {rows.map((row, i) => {
              const content = (
                <div className="flex items-center justify-between gap-3 py-3 text-[15px]">
                  <span className="flex items-center gap-2.5 min-w-0">
                    <span className="shrink-0 text-sm font-semibold text-[var(--color-muted)]">{i + 1}</span>
                    <span className="truncate text-[var(--color-ink)]">{row.label}</span>
                  </span>
                  <span className="shrink-0 font-medium text-[var(--color-ink)]">{row.value}</span>
                </div>
              );
              return (
                <li key={row.key}>
                  {row.href ? (
                    <Link href={row.href} className="block hover:bg-[var(--color-surface-soft)]">
                      {content}
                    </Link>
                  ) : (
                    content
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}

/** Top sellers, top revenue, best categories, and stock-health call-outs for the selected date range. */
export function DashboardProductPerformance() {
  const [rangeKey, setRangeKey] = useState<RangeKey>("30");
  const range = useMemo(() => {
    const opt = RANGE_OPTIONS.find((o) => o.key === rangeKey) ?? RANGE_OPTIONS[1];
    return lastNDaysRange(opt.days);
  }, [rangeKey]);

  const { topUnits, topRevenue, topCategories, isLoading, isError } = useProductPerformance(range);
  const { needsRestocking, fastMovers } = useRestockingAndMovers(topUnits);

  return (
    <div className="mt-10 space-y-4">
      <div>
        <p className="text-base font-semibold text-[var(--color-ink)]">Product performance</p>
        <p className="mt-1 text-sm text-[var(--color-muted)]">What&apos;s selling, and what needs attention.</p>
      </div>

      <AdminFilterCard title="Date range" icon={<Filter className="h-4 w-4 text-[var(--color-green)]" />}>
        <div className="flex flex-wrap gap-2">
          {RANGE_OPTIONS.map((opt) => (
            <Button
              key={opt.key}
              type="button"
              variant={rangeKey === opt.key ? "default" : "outline"}
              onClick={() => setRangeKey(opt.key)}
            >
              {opt.label}
            </Button>
          ))}
        </div>
      </AdminFilterCard>

      {isError ? (
        <div className="rounded-md border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
          Could not load product performance for this range.
        </div>
      ) : isLoading ? (
        <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading product performance...</p>
      ) : (
        <>
          <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
            <ChartCard title="Top products by units sold" subtitle="Most units sold in the selected range">
              {topUnits.length === 0 ? (
                <p className="py-10 text-center text-sm text-[var(--color-muted)]">No sales in this range yet.</p>
              ) : (
                <HorizontalBarChart data={topUnits} valueFormat={(v) => String(v)} barColor="var(--color-green-2)" />
              )}
            </ChartCard>

            <ChartCard title="Top products by revenue" subtitle="Highest-earning products in the selected range">
              {topRevenue.length === 0 ? (
                <p className="py-10 text-center text-sm text-[var(--color-muted)]">No sales in this range yet.</p>
              ) : (
                <HorizontalBarChart data={topRevenue} valueFormat={(v) => formatInrFromPaise(v)} barColor="var(--color-accent-gold)" />
              )}
            </ChartCard>

            <ChartCard title="Best-performing categories" subtitle="Revenue by category in the selected range">
              {topCategories.length === 0 ? (
                <p className="py-10 text-center text-sm text-[var(--color-muted)]">No sales in this range yet.</p>
              ) : (
                <HorizontalBarChart data={topCategories} valueFormat={(v) => formatInrFromPaise(v)} barColor="#2a78d6" />
              )}
            </ChartCard>
          </div>

          <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
            <ListCard
              title="Needs restocking"
              icon={<Boxes className="h-4 w-4 text-[var(--color-green)]" />}
              emptyText="Nothing running low right now."
              rows={needsRestocking.map((p) => ({
                key: p.productId,
                label: p.name,
                value: `${p.stockQuantity ?? 0} left`,
                href: "/imtheboss/products",
              }))}
            />
            <ListCard
              title="Fast movers"
              icon={<Flame className="h-4 w-4 text-[var(--color-green)]" />}
              emptyText="No sales in this range yet."
              rows={fastMovers.map((p, i) => ({
                key: `${p.label}-${i}`,
                label: p.label,
                value: `${p.value} sold`,
              }))}
            />
          </div>
        </>
      )}
    </div>
  );
}
