"use client";

import { useQuery } from "@tanstack/react-query";
import {
  ShoppingCart,
  Clock,
  CheckCircle2,
  XCircle,
  Truck,
  IndianRupee,
  Users,
  type LucideIcon,
} from "lucide-react";
import {
  fetchOrderCountsByStatus,
  fetchDashboardExtras,
  type OrderCountsByStatus,
  type DashboardExtras,
} from "@/lib/admin-queries";

const ORDER_STATS_CONFIG: Array<{
  key: keyof OrderCountsByStatus;
  label: string;
  format: (c: OrderCountsByStatus) => string;
  icon: LucideIcon;
}> = [
  { key: "total", label: "Total orders", format: (c) => String(c.total), icon: ShoppingCart },
  { key: "pending", label: "Pending", format: (c) => String(c.pending), icon: Clock },
  { key: "delivered", label: "Delivered", format: (c) => String(c.delivered), icon: CheckCircle2 },
  { key: "cancelled", label: "Cancelled", format: (c) => String(c.cancelled), icon: XCircle },
  { key: "inTransit", label: "In transit", format: (c) => String(c.inTransit), icon: Truck },
];

const EXTRA_STATS_CONFIG: Array<{
  key: keyof DashboardExtras;
  label: string;
  format: (e: DashboardExtras) => string;
  icon: LucideIcon;
}> = [
  { key: "revenueMtdFormatted", label: "Revenue (MTD)", format: (e) => e.revenueMtdFormatted, icon: IndianRupee },
  { key: "revenueTotalFormatted", label: "Revenue (Total)", format: (e) => e.revenueTotalFormatted, icon: IndianRupee },
  { key: "customersCount", label: "Customers", format: (e) => String(e.customersCount), icon: Users },
];

function StatCard({ label, value, Icon }: { label: string; value: string; Icon: LucideIcon }) {
  return (
    <article className="rounded-2xl border border-[var(--color-line)] bg-[var(--admin-surface-muted)] p-5 shadow-[var(--admin-card-shadow)]">
      <div className="flex items-center justify-between gap-3">
        <p className="text-sm font-medium text-[var(--color-muted)]">{label}</p>
        <span className="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-lg border border-[var(--color-line)] bg-white text-[var(--color-green)]">
          <Icon className="h-5 w-5" />
        </span>
      </div>
      <p className="mt-3 text-3xl font-semibold tracking-tight text-[var(--color-ink)]">{value}</p>
    </article>
  );
}

export function DashboardStats() {
  const orderCounts = useQuery({
    queryKey: ["admin", "dashboard-order-counts"],
    queryFn: fetchOrderCountsByStatus,
    staleTime: 60 * 1000,
  });
  const extras = useQuery({
    queryKey: ["admin", "dashboard-extras"],
    queryFn: fetchDashboardExtras,
    staleTime: 60 * 1000,
  });

  const isLoading = orderCounts.isLoading || extras.isLoading;
  const isError = orderCounts.isError || extras.isError;
  const error = orderCounts.error ?? extras.error;
  const counts = orderCounts.data;
  const extraData = extras.data;

  if (isLoading) {
    return (
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        {[...ORDER_STATS_CONFIG, ...EXTRA_STATS_CONFIG].map(({ label }) => (
          <div key={label} className="min-h-[104px] animate-pulse rounded-2xl border border-[var(--color-line)] bg-[var(--admin-surface-muted)]" />
        ))}
      </div>
    );
  }

  if (isError || !counts || !extraData) {
    return (
      <div className="rounded-md border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
        <p className="font-medium">Could not load dashboard stats.</p>
        <p className="mt-1 text-xs">{error?.message ?? "Unknown error"}</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
      {ORDER_STATS_CONFIG.map(({ key, label, format, icon: Icon }) => (
        <StatCard key={key} label={label} value={format(counts)} Icon={Icon} />
      ))}
      {EXTRA_STATS_CONFIG.map(({ key, label, format, icon: Icon }) => (
        <StatCard key={key} label={label} value={format(extraData)} Icon={Icon} />
      ))}
    </div>
  );
}
