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
  cardClass: string;
  overlayClass: string;
}> = [
  {
    key: "total",
    label: "Total orders",
    format: (c) => String(c.total),
    icon: ShoppingCart,
    cardClass: "bg-blue-500",
    overlayClass: "from-white/20 to-transparent",
  },
  {
    key: "pending",
    label: "Pending",
    format: (c) => String(c.pending),
    icon: Clock,
    cardClass: "bg-rose-500",
    overlayClass: "from-white/20 to-transparent",
  },
  {
    key: "delivered",
    label: "Delivered",
    format: (c) => String(c.delivered),
    icon: CheckCircle2,
    cardClass: "bg-violet-500",
    overlayClass: "from-white/20 to-transparent",
  },
  {
    key: "cancelled",
    label: "Cancelled",
    format: (c) => String(c.cancelled),
    icon: XCircle,
    cardClass: "bg-slate-600",
    overlayClass: "from-white/20 to-transparent",
  },
  {
    key: "inTransit",
    label: "In transit",
    format: (c) => String(c.inTransit),
    icon: Truck,
    cardClass: "bg-emerald-600",
    overlayClass: "from-white/20 to-transparent",
  },
];

const EXTRA_STATS_CONFIG: Array<{
  key: keyof DashboardExtras;
  label: string;
  format: (e: DashboardExtras) => string;
  icon: LucideIcon;
  cardClass: string;
  overlayClass: string;
}> = [
  {
    key: "revenueMtdFormatted",
    label: "Revenue (MTD)",
    format: (e) => e.revenueMtdFormatted,
    icon: IndianRupee,
    cardClass: "bg-amber-500",
    overlayClass: "from-white/20 to-transparent",
  },
  {
    key: "revenueTotalFormatted",
    label: "Revenue (Total)",
    format: (e) => e.revenueTotalFormatted,
    icon: IndianRupee,
    cardClass: "bg-orange-500",
    overlayClass: "from-white/20 to-transparent",
  },
  {
    key: "customersCount",
    label: "Customers",
    format: (e) => String(e.customersCount),
    icon: Users,
    cardClass: "bg-teal-600",
    overlayClass: "from-white/20 to-transparent",
  },
];

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
      <div className="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        {[...ORDER_STATS_CONFIG, ...EXTRA_STATS_CONFIG].map(({ label, cardClass }) => (
          <div
            key={label}
            className={`relative flex min-h-[100px] items-center gap-4 overflow-hidden rounded-xl ${cardClass} px-5 py-4 text-white`}
          >
            <div className="h-10 w-10 rounded-lg bg-white/20" />
            <div>
              <p className="text-sm font-medium opacity-90">…</p>
              <p className="mt-0.5 text-2xl font-semibold tracking-tight">…</p>
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (isError || !counts || !extraData) {
    return (
      <div className="mt-8 rounded-xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
        <p className="font-medium">Could not load dashboard stats.</p>
        <p className="mt-1 text-xs">{error?.message ?? "Unknown error"}</p>
      </div>
    );
  }

  return (
    <div className="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
      {ORDER_STATS_CONFIG.map(({ key, label, format, icon: Icon, cardClass, overlayClass }) => (
        <div
          key={key}
          className={`relative flex min-h-[100px] items-center gap-4 overflow-hidden rounded-xl ${cardClass} px-5 py-4 text-white`}
        >
          <div
            className={`absolute right-0 top-0 h-full w-1/3 bg-gradient-to-l ${overlayClass}`}
            aria-hidden
          />
          <div className="relative flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-white/20">
            <Icon className="h-6 w-6" strokeWidth={2} />
          </div>
          <div className="relative min-w-0">
            <p className="text-sm font-medium opacity-90">{label}</p>
            <p className="mt-0.5 text-2xl font-semibold tracking-tight">{format(counts)}</p>
          </div>
        </div>
      ))}
      {EXTRA_STATS_CONFIG.map(({ key, label, format, icon: Icon, cardClass, overlayClass }) => (
        <div
          key={key}
          className={`relative flex min-h-[100px] items-center gap-4 overflow-hidden rounded-xl ${cardClass} px-5 py-4 text-white`}
        >
          <div
            className={`absolute right-0 top-0 h-full w-1/3 bg-gradient-to-l ${overlayClass}`}
            aria-hidden
          />
          <div className="relative flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-white/20">
            <Icon className="h-6 w-6" strokeWidth={2} />
          </div>
          <div className="relative min-w-0">
            <p className="text-sm font-medium opacity-90">{label}</p>
            <p className="mt-0.5 text-2xl font-semibold tracking-tight">{format(extraData)}</p>
          </div>
        </div>
      ))}
    </div>
  );
}
