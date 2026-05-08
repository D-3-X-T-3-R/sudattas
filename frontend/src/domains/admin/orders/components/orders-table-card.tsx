"use client";

import { useRouter } from "next/navigation";
import { Button } from "@/components/ui/button";
import type { OrderListRow } from "@/lib/admin-queries";
import { ListOrdered } from "lucide-react";
import { formatOrderDate, getStatusLabel } from "@/domains/admin/orders/utils";
import { AdminTableCard } from "@/components/admin/admin-cards";

type OrdersTableCardProps = {
  isLoading: boolean;
  isError: boolean;
  errorTitle?: string;
  errorMessage?: string;
  orders: OrderListRow[];
  statuses: { statusId: string; statusName: string }[];
  page: number;
  pageSize: number;
  setPage: (updater: (prev: number) => number) => void;
};

export function OrdersTableCard({
  isLoading,
  isError,
  errorTitle,
  errorMessage,
  orders,
  statuses,
  page,
  pageSize,
  setPage,
}: OrdersTableCardProps) {
  const router = useRouter();

  return (
    <AdminTableCard title="Orders" icon={<ListOrdered className="h-4 w-4 text-[var(--color-green)]" />} className="mt-6">
      {isLoading ? (
        <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading orders...</p>
      ) : null}

      {isError ? (
        <div className="rounded-md border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
          <p className="font-medium">{errorTitle ?? "Could not load orders."}</p>
          <p className="mt-1 text-xs">{errorMessage ?? "Please try again."}</p>
        </div>
      ) : null}

      {!isLoading && !isError && orders.length === 0 ? (
        <p className="py-8 text-center text-sm text-[var(--color-muted)]">No orders in this range.</p>
      ) : null}

      {!isLoading && !isError && orders.length > 0 ? (
        <div className="space-y-4">
          <div className="overflow-x-auto">
            <table className="w-full min-w-[680px] border-collapse text-sm">
              <caption className="sr-only">Orders list with pagination controls</caption>
              <thead>
                <tr className="border-b border-[var(--color-line)] text-left text-[var(--color-muted)]">
                  <th className="pb-3 pr-4 font-semibold uppercase tracking-[0.12em]">Order ID</th>
                  <th className="pb-3 pr-4 font-semibold uppercase tracking-[0.12em]">Date</th>
                  <th className="pb-3 pr-4 font-semibold uppercase tracking-[0.12em]">Customer</th>
                  <th className="pb-3 pr-4 font-semibold uppercase tracking-[0.12em]">Amount</th>
                  <th className="pb-3 font-semibold uppercase tracking-[0.12em]">Status</th>
                </tr>
              </thead>
              <tbody>
                {orders.map((order) => (
                  <tr
                    key={order.orderId}
                    role="link"
                    tabIndex={0}
                    className="cursor-pointer border-b border-[var(--color-line)] text-[var(--color-ink)] hover:bg-[var(--color-surface-soft)]"
                    onClick={() => router.push(`/imtheboss/orders/${order.orderId}`)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" || e.key === " ") {
                        e.preventDefault();
                        router.push(`/imtheboss/orders/${order.orderId}`);
                      }
                    }}
                  >
                    <td className="py-3 pr-4 font-mono">{order.orderId}</td>
                    <td className="py-3 pr-4">{formatOrderDate(order.orderDate)}</td>
                    <td className="py-3 pr-4">{order.userId}</td>
                    <td className="py-3 pr-4">{order.totalAmountFormatted}</td>
                    <td className="py-3 text-[var(--color-muted)]">{getStatusLabel(order.statusId, statuses)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="flex items-center justify-between gap-3">
            <p className="text-xs text-[var(--color-muted)]">
              Page {page} - showing up to {pageSize} rows
            </p>
            <div className="flex gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={page <= 1 || isLoading}
                onClick={() => setPage((p) => Math.max(1, p - 1))}
              >
                Previous
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={orders.length < pageSize || isLoading}
                onClick={() => setPage((p) => p + 1)}
              >
                Next
              </Button>
            </div>
          </div>
        </div>
      ) : null}
    </AdminTableCard>
  );
}
