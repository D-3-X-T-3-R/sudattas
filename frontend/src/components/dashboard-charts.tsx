"use client";

import { useState, useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { ResponsiveBar } from "@nivo/bar";
import { Card, CardContent } from "@/components/ui/card";
import {
  fetchOrdersByDateRange,
  fetchCustomersList,
  lastNYearsRange,
} from "@/lib/admin-queries";
import { cn } from "@/lib/utils";

type Granularity = "day" | "week" | "month" | "year";

type BarDatum = { label: string; value: number };

function parseOrderDate(orderDate: string): Date {
  const d = new Date(orderDate);
  return Number.isNaN(d.getTime()) ? new Date(0) : d;
}

function dayKey(d: Date): string {
  return d.toISOString().slice(0, 10);
}

function weekKey(d: Date): string {
  const copy = new Date(d);
  const day = copy.getDay();
  const toMonday = day === 0 ? -6 : 1 - day;
  copy.setDate(copy.getDate() + toMonday);
  return copy.toISOString().slice(0, 10);
}

function monthKey(d: Date): string {
  return d.toISOString().slice(0, 7);
}

function yearKey(d: Date): string {
  return String(d.getFullYear());
}

function formatDayLabel(isoDay: string): string {
  const d = new Date(isoDay + "T12:00:00");
  return d.toLocaleDateString("en-IN", { day: "numeric", month: "short" });
}

function formatWeekLabel(isoMonday: string): string {
  const d = new Date(isoMonday + "T12:00:00");
  const end = new Date(d);
  end.setDate(end.getDate() + 6);
  return d.toLocaleDateString("en-IN", { day: "numeric", month: "short" }) + " - " + end.toLocaleDateString("en-IN", { day: "numeric", month: "short" });
}

function formatMonthLabel(isoMonth: string): string {
  const [y, m] = isoMonth.split("-");
  const d = new Date(Number(y), Number(m) - 1, 1);
  return d.toLocaleDateString("en-IN", { month: "short", year: "2-digit" });
}

function formatYearLabel(y: string): string {
  return y;
}

function getKeyForGranularity(d: Date, g: Granularity): string {
  if (g === "day") return dayKey(d);
  if (g === "week") return weekKey(d);
  if (g === "year") return yearKey(d);
  return monthKey(d);
}

function getLabelFormatter(g: Granularity): (key: string) => string {
  if (g === "day") return formatDayLabel;
  if (g === "week") return formatWeekLabel;
  if (g === "year") return formatYearLabel;
  return formatMonthLabel;
}

function buildEmptyBuckets(g: Granularity): Record<string, number> {
  const buckets: Record<string, number> = {};
  const now = new Date();
  if (g === "day") {
    for (let i = 29; i >= 0; i--) {
      const d = new Date(now);
      d.setDate(d.getDate() - i);
      buckets[dayKey(d)] = 0;
    }
  } else if (g === "week") {
    for (let i = 11; i >= 0; i--) {
      const d = new Date(now);
      d.setDate(d.getDate() - i * 7);
      buckets[weekKey(d)] = 0;
    }
  } else if (g === "year") {
    for (let i = 4; i >= 0; i--) {
      buckets[String(now.getFullYear() - i)] = 0;
    }
  } else {
    for (let i = 11; i >= 0; i--) {
      const d = new Date(now.getFullYear(), now.getMonth() - i, 1);
      buckets[monthKey(d)] = 0;
    }
  }
  return buckets;
}

function aggregateByGranularity(
  orders: { orderDate: string; totalAmountPaise: string }[],
  granularity: Granularity,
  valueFn: (o: { orderDate: string; totalAmountPaise: string }) => number
): BarDatum[] {
  const buckets = buildEmptyBuckets(granularity);
  for (const o of orders) {
    const d = parseOrderDate(o.orderDate);
    const k = getKeyForGranularity(d, granularity);
    if (buckets[k] !== undefined) buckets[k] += valueFn(o);
  }
  const format = getLabelFormatter(granularity);
  return Object.entries(buckets)
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([label, value]) => ({ label: format(label), value }));
}

function aggregateCustomersByGranularity(
  customers: { createDate: string | null }[],
  granularity: Granularity
): BarDatum[] {
  const buckets = buildEmptyBuckets(granularity);
  for (const c of customers) {
    if (!c.createDate) continue;
    const d = new Date(c.createDate);
    if (Number.isNaN(d.getTime())) continue;
    const k = getKeyForGranularity(d, granularity);
    if (buckets[k] !== undefined) buckets[k]++;
  }
  const format = getLabelFormatter(granularity);
  return Object.entries(buckets)
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([label, value]) => ({ label: format(label), value }));
}

const chartTheme = {
  axis: {
    ticks: {
      text: {
        fill: "#6b7280",
        fontSize: 11,
      },
      line: {
        stroke: "#e5e7eb",
        strokeWidth: 1,
      },
    },
    domain: {
      line: {
        stroke: "#e5e7eb",
        strokeWidth: 1,
      },
    },
  },
  grid: {
    line: {
      stroke: "#e5e7eb",
      strokeWidth: 1,
    },
  },
  tooltip: {
    container: {
      background: "var(--color-paper)",
      border: "1px solid var(--color-line)",
      borderRadius: 8,
      padding: "8px 12px",
    },
  },
};

function BarChart({
  data,
  valueFormat = (v) => String(v),
  barColor = "var(--color-accent-brown)",
  className,
}: {
  data: BarDatum[];
  valueFormat?: (v: number) => string;
  barColor?: string;
  className?: string;
}) {
  const total = useMemo(
    () => data.reduce((s, d) => s + d.value, 0),
    [data]
  );
  const maxVal = useMemo(() => Math.max(1, ...data.map((d) => d.value)), [data]);

  return (
    <div className={className}>
      <div
        className="rounded-lg bg-white"
        style={{ height: 260, width: "100%", padding: "12px 8px 8px 8px" }}
      >
        <ResponsiveBar
          data={data}
          indexBy="label"
          keys={["value"]}
          valueScale={{ type: "linear", min: 0, max: maxVal, clamp: true }}
          margin={{ top: 12, right: 12, left: 36, bottom: 32 }}
          padding={0.4}
          theme={chartTheme}
          colors={[barColor]}
          borderRadius={6}
          axisBottom={{
            tickSize: 0,
            tickPadding: 8,
            tickRotation: 0,
          }}
          axisLeft={{
            tickSize: 0,
            tickPadding: 8,
            tickValues: 5,
          }}
          enableGridY={true}
          gridYValues={5}
          valueFormat={valueFormat}
          tooltip={({ label, value }) => (
            <div
              style={{
                background: "var(--color-paper)",
                border: "1px solid var(--color-line)",
                borderRadius: 8,
                padding: "8px 12px",
                fontSize: 12,
              }}
            >
              <strong>{label}</strong>: {valueFormat(Number(value))}
            </div>
          )}
        />
      </div>
      <p className="mt-2 text-xs text-[var(--color-muted)]">
        Max: {valueFormat(maxVal)} · Total: {valueFormat(total)}
      </p>
    </div>
  );
}

const GRANULARITY_OPTIONS: { value: Granularity; label: string }[] = [
  { value: "day", label: "By day" },
  { value: "week", label: "By week" },
  { value: "month", label: "By month" },
  { value: "year", label: "By year" },
];

function DashboardChartsInner() {
  const [ordersGranularity, setOrdersGranularity] = useState<Granularity>("month");
  const [revenueGranularity, setRevenueGranularity] = useState<Granularity>("day");
  const [customersGranularity, setCustomersGranularity] = useState<Granularity>("month");

  const years5 = lastNYearsRange(5);

  const { data: orders = [] } = useQuery({
    queryKey: ["admin", "dashboard-orders-charts", years5],
    queryFn: () =>
      fetchOrdersByDateRange({
        ...years5,
        limit: "10000",
      }),
    staleTime: 2 * 60 * 1000,
  });

  const { data: customers = [] } = useQuery({
    queryKey: ["admin", "customers"],
    queryFn: () => fetchCustomersList(),
    staleTime: 2 * 60 * 1000,
  });

  const ordersData = useMemo(
    () => aggregateByGranularity(orders, ordersGranularity, () => 1),
    [orders, ordersGranularity]
  );

  const revenueData = useMemo(
    () =>
      aggregateByGranularity(orders, revenueGranularity, (o) =>
        parseInt(o.totalAmountPaise, 10) || 0
      ),
    [orders, revenueGranularity]
  );

  const customersData = useMemo(
    () => aggregateCustomersByGranularity(customers, customersGranularity),
    [customers, customersGranularity]
  );

  const formatCurrency = (paise: number) =>
    new Intl.NumberFormat("en-IN", {
      style: "currency",
      currency: "INR",
      maximumFractionDigits: 0,
    }).format(paise / 100);

  return (
    <div className="mt-10 space-y-6">
      <p className="text-sm font-medium text-[var(--color-muted)]">Charts</p>

      <div className="grid gap-6 grid-cols-1">
        <Card className="overflow-hidden rounded-xl border-[var(--color-line)] bg-white shadow-sm">
          <div className="flex flex-wrap items-center justify-between gap-2 p-4 pb-0">
            <p className="text-sm font-medium text-[var(--color-muted)]">Orders</p>
            <select
              className={cn(
                "h-9 rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              )}
              value={ordersGranularity}
              onChange={(e) => setOrdersGranularity(e.target.value as Granularity)}
              aria-label="Orders chart granularity"
            >
              {GRANULARITY_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
          <CardContent className="mt-0 pt-3">
            <BarChart
              data={ordersData}
              valueFormat={(v) => String(v)}
              barColor="var(--color-accent-brown)"
            />
          </CardContent>
        </Card>

        <Card className="overflow-hidden rounded-xl border-[var(--color-line)] bg-white shadow-sm">
          <div className="flex flex-wrap items-center justify-between gap-2 p-4 pb-0">
            <p className="text-sm font-medium text-[var(--color-muted)]">Revenue</p>
            <select
              className={cn(
                "h-9 rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              )}
              value={revenueGranularity}
              onChange={(e) => setRevenueGranularity(e.target.value as Granularity)}
              aria-label="Revenue chart granularity"
            >
              {GRANULARITY_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
          <CardContent className="mt-0 pt-3">
            <BarChart
              data={revenueData}
              valueFormat={formatCurrency}
              barColor="var(--color-accent-gold)"
            />
          </CardContent>
        </Card>
        <Card className="overflow-hidden rounded-xl border-[var(--color-line)] bg-white shadow-sm">
          <div className="flex flex-wrap items-center justify-between gap-2 p-4 pb-0">
            <p className="text-sm font-medium text-[var(--color-muted)]">New customers</p>
            <select
              className={cn(
                "h-9 rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              )}
              value={customersGranularity}
              onChange={(e) => setCustomersGranularity(e.target.value as Granularity)}
              aria-label="New customers chart granularity"
            >
              {GRANULARITY_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
          <CardContent className="mt-0 pt-3">
            <BarChart
              data={customersData}
              valueFormat={(v) => String(v)}
              barColor="var(--color-accent-brown)"
            />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

export const DashboardCharts = DashboardChartsInner;
