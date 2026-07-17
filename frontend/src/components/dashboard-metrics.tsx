"use client";

import { useMemo } from "react";
import Link from "next/link";
import { useQuery } from "@tanstack/react-query";
import {
  Wallet,
  Repeat,
  PackageCheck,
  XCircle,
  TrendingUp,
  TrendingDown,
  Minus,
  Boxes,
  type LucideIcon,
} from "lucide-react";
import { fetchAllOrdersList, fetchOrderStatuses, fetchAllProductsSummary } from "@/lib/admin-queries";
import { formatInrFromPaise } from "@/lib/money";
import { cn } from "@/lib/utils";

const LOW_STOCK_THRESHOLD = 5;

function monthKey(d: Date): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
}

interface Metric {
  key: string;
  label: string;
  value: string;
  hint: string;
  Icon: LucideIcon;
  href?: string;
}

function useDashboardMetrics() {
  const { data: allOrders = [] } = useQuery({
    queryKey: ["admin", "orders", "all-for-stats"],
    queryFn: () => fetchAllOrdersList(),
    staleTime: 2 * 60 * 1000,
  });

  const { data: statuses = [] } = useQuery({
    queryKey: ["admin", "order-statuses"],
    queryFn: fetchOrderStatuses,
  });

  const { data: productsSummary = [] } = useQuery({
    queryKey: ["admin", "products-summary"],
    queryFn: fetchAllProductsSummary,
    staleTime: 2 * 60 * 1000,
  });

  return useMemo(() => {
    const idToName = new Map(statuses.map((s) => [s.statusId, s.statusName.trim().toLowerCase()]));
    const totalOrders = allOrders.length;
    const totalRevenuePaise = allOrders.reduce((sum, o) => sum + (parseInt(o.totalAmountPaise, 10) || 0), 0);
    const aovPaise = totalOrders > 0 ? Math.round(totalRevenuePaise / totalOrders) : 0;

    const ordersByUser = new Map<string, number>();
    for (const o of allOrders) ordersByUser.set(o.userId, (ordersByUser.get(o.userId) ?? 0) + 1);
    const buyingCustomers = ordersByUser.size;
    const repeatCustomers = [...ordersByUser.values()].filter((c) => c > 1).length;
    const repeatRate = buyingCustomers > 0 ? (repeatCustomers / buyingCustomers) * 100 : 0;

    let delivered = 0;
    let cancelled = 0;
    for (const o of allOrders) {
      const name = idToName.get(o.statusId) ?? "";
      if (name === "delivered") delivered++;
      else if (name === "cancelled") cancelled++;
    }
    const fulfillmentRate = totalOrders > 0 ? (delivered / totalOrders) * 100 : 0;
    const cancellationRate = totalOrders > 0 ? (cancelled / totalOrders) * 100 : 0;

    const now = new Date();
    const thisMonthKey = monthKey(now);
    const lastMonthKey = monthKey(new Date(now.getFullYear(), now.getMonth() - 1, 1));
    let thisMonthRevenue = 0;
    let lastMonthRevenue = 0;
    for (const o of allOrders) {
      const d = new Date(o.orderDate);
      if (Number.isNaN(d.getTime())) continue;
      const k = monthKey(d);
      const paise = parseInt(o.totalAmountPaise, 10) || 0;
      if (k === thisMonthKey) thisMonthRevenue += paise;
      else if (k === lastMonthKey) lastMonthRevenue += paise;
    }
    const revenueGrowthPct = lastMonthRevenue > 0 ? ((thisMonthRevenue - lastMonthRevenue) / lastMonthRevenue) * 100 : null;

    const lowStockCount = productsSummary.filter((p) => {
      const isArchived = (p.productStatusId ?? "").trim() === "3";
      if (isArchived) return false;
      return p.stockQuantity != null && p.stockQuantity <= LOW_STOCK_THRESHOLD;
    }).length;

    return {
      aovPaise,
      buyingCustomers,
      repeatCustomers,
      repeatRate,
      totalOrders,
      delivered,
      cancelled,
      fulfillmentRate,
      cancellationRate,
      revenueGrowthPct,
      lowStockCount,
    };
  }, [allOrders, statuses, productsSummary]);
}

function DeltaBadge({ pct }: { pct: number | null }) {
  if (pct == null) {
    return <span className="text-sm text-[var(--color-muted)]">Not enough data yet</span>;
  }
  if (Math.abs(pct) < 0.5) {
    return (
      <span className="inline-flex items-center gap-1 text-sm font-medium text-[var(--color-muted)]">
        <Minus className="h-3.5 w-3.5" /> Flat vs last month
      </span>
    );
  }
  const up = pct > 0;
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 text-sm font-semibold",
        up ? "text-emerald-600" : "text-rose-600"
      )}
    >
      {up ? <TrendingUp className="h-3.5 w-3.5" /> : <TrendingDown className="h-3.5 w-3.5" />}
      {Math.abs(pct).toFixed(0)}% vs last month
    </span>
  );
}

function MetricTile({ metric }: { metric: Metric }) {
  const body = (
    <article className="h-full rounded-2xl border border-[var(--color-line)] bg-[var(--admin-surface-muted)] p-5 shadow-[var(--admin-card-shadow)] transition-colors hover:border-[var(--color-gold)]">
      <div className="flex items-start justify-between gap-3">
        <p className="text-sm font-medium text-[var(--color-muted)]">{metric.label}</p>
        <span className="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border border-[var(--color-line)] bg-white text-[var(--color-green)]">
          <metric.Icon className="h-5 w-5" />
        </span>
      </div>
      <p className="mt-3 text-3xl font-semibold tracking-tight text-[var(--color-ink)]">{metric.value}</p>
      <p className="mt-1.5 text-sm text-[var(--color-muted)]">{metric.hint}</p>
    </article>
  );
  if (!metric.href) return body;
  return (
    <Link href={metric.href} className="block h-full">
      {body}
    </Link>
  );
}

/** Row of derived business metrics — the "so what" numbers behind the raw counts in DashboardStats. */
export function DashboardMetrics() {
  const m = useDashboardMetrics();

  const metrics: Metric[] = [
    {
      key: "aov",
      label: "Average order value",
      value: m.totalOrders > 0 ? formatInrFromPaise(m.aovPaise) : "—",
      hint: "Per order, all-time",
      Icon: Wallet,
    },
    {
      key: "repeat",
      label: "Repeat customers",
      value: m.buyingCustomers > 0 ? `${m.repeatRate.toFixed(0)}%` : "—",
      hint:
        m.buyingCustomers > 0
          ? `${m.repeatCustomers} of ${m.buyingCustomers} customers ordered again`
          : "No orders yet",
      Icon: Repeat,
    },
    {
      key: "fulfillment",
      label: "Orders delivered",
      value: m.totalOrders > 0 ? `${m.fulfillmentRate.toFixed(0)}%` : "—",
      hint: m.totalOrders > 0 ? `${m.delivered} of ${m.totalOrders} orders` : "No orders yet",
      Icon: PackageCheck,
    },
    {
      key: "cancelled",
      label: "Cancelled orders",
      value: m.totalOrders > 0 ? `${m.cancellationRate.toFixed(0)}%` : "—",
      hint: m.totalOrders > 0 ? `${m.cancelled} of ${m.totalOrders} orders` : "No orders yet",
      Icon: XCircle,
    },
    {
      key: "low-stock",
      label: "Running low on stock",
      value: String(m.lowStockCount),
      hint: `${LOW_STOCK_THRESHOLD} units or fewer left — tap to review`,
      Icon: Boxes,
      href: "/imtheboss/products",
    },
  ];

  return (
    <section className="mt-8">
      <div className="mb-3">
        <h2 className="text-base font-semibold text-[var(--color-ink)]">Business at a glance</h2>
        <p className="mt-1 text-sm text-[var(--color-muted)]">The numbers behind the numbers.</p>
      </div>
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <article className="h-full rounded-2xl border border-[var(--color-line)] bg-[var(--admin-surface-muted)] p-5 shadow-[var(--admin-card-shadow)]">
          <div className="flex items-start justify-between gap-3">
            <p className="text-sm font-medium text-[var(--color-muted)]">Revenue growth</p>
            <span className="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border border-[var(--color-line)] bg-white text-[var(--color-green)]">
              <TrendingUp className="h-5 w-5" />
            </span>
          </div>
          <p className="mt-3 text-3xl font-semibold tracking-tight text-[var(--color-ink)]">
            {m.revenueGrowthPct == null ? "—" : `${m.revenueGrowthPct > 0 ? "+" : ""}${m.revenueGrowthPct.toFixed(0)}%`}
          </p>
          <p className="mt-1.5">
            <DeltaBadge pct={m.revenueGrowthPct} />
          </p>
        </article>
        {metrics.map((metric) => (
          <MetricTile key={metric.key} metric={metric} />
        ))}
      </div>
    </section>
  );
}
