"use client";

import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import type { OrderListRow } from "@/lib/admin-queries";
import { ListOrdered } from "lucide-react";
import { formatOrderDate, getStatusLabel } from "@/domains/admin/orders/utils";

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
  return (
    <Card className="mt-6 rounded-xl border-[var(--color-line)] border-l-4 border-l-violet-500 bg-white shadow-[var(--admin-card-shadow)]">
      <CardTitle className="flex items-center gap-2 text-[var(--color-muted)]">
        <ListOrdered className="h-4 w-4 text-violet-500" />
        Orders
      </CardTitle>
      <CardContent className="mt-3">
        {isLoading && <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading orders...</p>}
        {isError && (
          <div className="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
            <p className="font-medium">{errorTitle ?? "Could not load orders."}</p>
            <p className="mt-1 text-xs">{errorMessage ?? "Please try again."}</p>
          </div>
        )}
        {!isLoading && !isError && orders.length === 0 && (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">No orders in this range.</p>
        )}
        {!isLoading && !isError && orders.length > 0 && (
          <div className="space-y-4">
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
                      <td className="py-3 text-[var(--color-muted)]">{getStatusLabel(order.statusId, statuses)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
            <div className="flex items-center justify-between gap-3">
              <p className="text-xs text-[var(--color-muted)]">
                Page {page} . showing up to {pageSize} rows
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
        )}
      </CardContent>
    </Card>
  );
}
