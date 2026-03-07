"use client";

import { useState, useMemo } from "react";
import { useSearchParams } from "next/navigation";
import Link from "next/link";
import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Section } from "@/components/ui/section";
import { Kicker, SectionHeading } from "@/components/ui/typography";
import {
  fetchOrdersList,
  fetchOrderStatuses,
  type OrderListRow,
} from "@/lib/admin-queries";
import { cn } from "@/lib/utils";

type DatePreset = "7" | "30" | "month" | "all";

function getDateRange(preset: DatePreset): { orderDateStart?: string; orderDateEnd?: string } {
  const now = new Date();
  const end = new Date(now);
  end.setHours(23, 59, 59, 999);
  const endSec = Math.floor(end.getTime() / 1000);

  switch (preset) {
    case "7": {
      const start = new Date(now);
      start.setDate(start.getDate() - 7);
      start.setHours(0, 0, 0, 0);
      return { orderDateStart: String(Math.floor(start.getTime() / 1000)), orderDateEnd: String(endSec) };
    }
    case "30": {
      const start = new Date(now);
      start.setDate(start.getDate() - 30);
      start.setHours(0, 0, 0, 0);
      return { orderDateStart: String(Math.floor(start.getTime() / 1000)), orderDateEnd: String(endSec) };
    }
    case "month": {
      const start = new Date(now.getFullYear(), now.getMonth(), 1, 0, 0, 0, 0);
      return { orderDateStart: String(Math.floor(start.getTime() / 1000)), orderDateEnd: String(endSec) };
    }
    default:
      return {};
  }
}

function formatOrderDate(orderDate: string): string {
  try {
    const d = new Date(orderDate);
    if (Number.isNaN(d.getTime())) return orderDate;
    return d.toLocaleDateString("en-IN", {
      day: "2-digit",
      month: "short",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return orderDate;
  }
}

const PRESETS: { key: DatePreset; label: string }[] = [
  { key: "7", label: "Last 7 days" },
  { key: "30", label: "Last 30 days" },
  { key: "month", label: "This month" },
  { key: "all", label: "All" },
];

function getStatusLabel(statusId: string, statuses: { statusId: string; statusName: string }[]): string {
  const s = statuses.find((x) => x.statusId === statusId);
  return s ? s.statusName : statusId;
}

export default function AdminOrdersPage() {
  const searchParams = useSearchParams();
  const userIdFromUrl = searchParams.get("userId") ?? undefined;

  const [datePreset, setDatePreset] = useState<DatePreset>("30");
  const [statusId, setStatusId] = useState("");

  const filters = useMemo(() => {
    const dateRange = getDateRange(datePreset);
    return {
      ...dateRange,
      statusId: statusId.trim() || undefined,
      userId: userIdFromUrl,
      limit: "100",
    };
  }, [datePreset, statusId, userIdFromUrl]);

  const { data: statuses = [] } = useQuery({
    queryKey: ["admin", "order-statuses"],
    queryFn: fetchOrderStatuses,
  });

  const {
    data: orders = [],
    isLoading,
    isError,
    error,
    refetch,
  } = useQuery<OrderListRow[], Error>({
    queryKey: ["admin", "orders", filters],
    queryFn: () => fetchOrdersList(filters),
  });

  return (
    <Section compact className="max-w-5xl">
      <Kicker className="text-[var(--color-muted)]">Orders</Kicker>
      <SectionHeading size="default" className="mt-2">
        Order management
      </SectionHeading>

      <Card className="mt-8 border-[var(--color-line)]">
        <CardTitle className="text-[var(--color-muted)]">Filters</CardTitle>
        <CardContent className="mt-3 flex flex-wrap items-center gap-3">
          <div className="flex flex-wrap gap-2">
            {PRESETS.map(({ key, label }) => (
              <Button
                key={key}
                type="button"
                variant={datePreset === key ? "default" : "outline"}
                size="sm"
                onClick={() => setDatePreset(key)}
              >
                {label}
              </Button>
            ))}
          </div>
          {userIdFromUrl && (
            <div className="flex items-center gap-2 rounded-md border border-[var(--color-line)] bg-[var(--color-surface)] px-3 py-1.5 text-sm">
              <span className="text-[var(--color-muted)]">Customer:</span>
              <span className="font-mono text-[var(--color-ink)]">{userIdFromUrl}</span>
              <Link
                href="/imtheboss/orders"
                className="text-[var(--color-accent-brown)] hover:underline"
              >
                Clear
              </Link>
            </div>
          )}
          <div className="flex items-center gap-2">
            <label htmlFor="orders-status" className="text-sm text-[var(--color-muted)]">
              Status
            </label>
            <select
              id="orders-status"
              className={cn(
                "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
              )}
              value={statusId}
              onChange={(e) => setStatusId(e.target.value)}
            >
              <option value="">All statuses</option>
              {statuses.map((s) => (
                <option key={s.statusId} value={s.statusId}>
                  {s.statusName}
                </option>
              ))}
            </select>
          </div>
          <Button type="button" variant="outline" size="sm" onClick={() => refetch()}>
            Refresh
          </Button>
        </CardContent>
      </Card>

      <Card className="mt-6 border-[var(--color-line)]">
        <CardTitle className="text-[var(--color-muted)]">Orders</CardTitle>
        <CardContent className="mt-3">
          {isLoading && (
            <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading orders…</p>
          )}
          {isError && (
            <div className="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
              <p className="font-medium">Could not load orders.</p>
              <p className="mt-1 text-xs">{error?.message ?? "Unknown error"}</p>
            </div>
          )}
          {!isLoading && !isError && orders.length === 0 && (
            <p className="py-8 text-center text-sm text-[var(--color-muted)]">No orders in this range.</p>
          )}
          {!isLoading && !isError && orders.length > 0 && (
            <div className="overflow-x-auto">
              <table className="w-full min-w-[600px] border-collapse text-sm">
                <thead>
                  <tr className="border-b border-[var(--color-line)] text-left text-[var(--color-muted)]">
                    <th className="pb-2 pr-4 font-medium">Order ID</th>
                    <th className="pb-2 pr-4 font-medium">Date</th>
                    <th className="pb-2 pr-4 font-medium">Customer (user ID)</th>
                    <th className="pb-2 pr-4 font-medium">Amount</th>
                    <th className="pb-2 font-medium">Status</th>
                  </tr>
                </thead>
                <tbody>
                  {orders.map((order) => (
                    <tr
                      key={order.orderId}
                      className="border-b border-[var(--color-line)] last:border-0 hover:bg-[var(--color-surface)]"
                    >
                      <td className="py-3 pr-4 font-mono text-[var(--color-ink)]">{order.orderId}</td>
                      <td className="py-3 pr-4 text-[var(--color-ink)]">{formatOrderDate(order.orderDate)}</td>
                      <td className="py-3 pr-4 text-[var(--color-ink)]">{order.userId}</td>
                      <td className="py-3 pr-4 text-[var(--color-ink)]">{order.totalAmountFormatted}</td>
                      <td className="py-3 text-[var(--color-muted)]">
                        {getStatusLabel(order.statusId, statuses)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </CardContent>
      </Card>
    </Section>
  );
}
