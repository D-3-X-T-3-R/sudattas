"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { ArrowLeft, ListOrdered, Package } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { fetchOrderStatuses } from "@/lib/admin-queries";
import {
  fetchAdminOrderById,
  updateAdminPickupTarget,
  type AdminOrderDetail,
} from "@/lib/admin-order-detail";
import { OrderDetailStatusEditor } from "@/domains/admin/orders/components/order-detail-status-editor";
import { OrderTimelineCard } from "@/domains/admin/orders/components/order-timeline-card";
import { OrderShipmentCard } from "@/domains/admin/orders/components/order-shipment-card";
import { toRouteFailureUi } from "@/lib/route-state";
import { formatOrderDate, getStatusLabel } from "@/domains/admin/orders/utils";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { StatusBadge } from "@/components/admin/status-badge";

function toLocalDateTimeInput(iso?: string | null): string {
  if (!iso) return "";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "";
  const year = d.getFullYear();
  const month = `${d.getMonth() + 1}`.padStart(2, "0");
  const day = `${d.getDate()}`.padStart(2, "0");
  const hour = `${d.getHours()}`.padStart(2, "0");
  const minute = `${d.getMinutes()}`.padStart(2, "0");
  return `${year}-${month}-${day}T${hour}:${minute}`;
}

function localDateTimeToIso(localRaw: string): string | null {
  const trimmed = localRaw.trim();
  if (!trimmed) return null;
  const parsed = new Date(trimmed);
  if (Number.isNaN(parsed.getTime())) return null;
  return parsed.toISOString();
}

// eslint-disable-next-line max-lines-per-function
export default function AdminOrderDetailPage() {
  const queryClient = useQueryClient();
  const params = useParams();
  const orderId =
    typeof params?.orderId === "string"
      ? params.orderId
      : Array.isArray(params?.orderId)
        ? params.orderId[0]
        : "";

  const { data: statuses = [] } = useQuery({
    queryKey: ["admin", "order-statuses"],
    queryFn: fetchOrderStatuses,
  });

  const {
    data: order,
    isLoading,
    isError,
    error,
    refetch,
  } = useQuery<AdminOrderDetail | null, Error>({
    queryKey: ["admin", "order", orderId],
    queryFn: () => fetchAdminOrderById(orderId),
    enabled: Boolean(orderId),
  });

  const [pickupTargetDraft, setPickupTargetDraft] = useState("");
  const [pickupTargetDirty, setPickupTargetDirty] = useState(false);
  const [pickupReasonDraft, setPickupReasonDraft] = useState("");

  const pickupTargetValue = pickupTargetDirty
    ? pickupTargetDraft
    : toLocalDateTimeInput(order?.pickupTargetAt);

  const pickupTargetMutation = useMutation({
    mutationFn: async () => {
      if (!order) throw new Error("Order not loaded");
      const pickupTargetAt = localDateTimeToIso(pickupTargetValue);
      if (!pickupTargetAt) {
        throw new Error("Pickup target must be a valid date/time");
      }
      return updateAdminPickupTarget({
        orderId: order.orderId,
        pickupTargetAt,
        reason: pickupReasonDraft.trim() || undefined,
      });
    },
    onSuccess: () => {
      setPickupTargetDirty(false);
      setPickupTargetDraft("");
      setPickupReasonDraft("");
      queryClient.invalidateQueries({ queryKey: ["admin", "order", orderId] });
      queryClient.invalidateQueries({ queryKey: ["admin", "orders"] });
    },
  });

  const errorUi = isError ? toRouteFailureUi("admin", error) : null;
  const notFound = !isLoading && !isError && orderId && order == null;

  const refundTrackingNote =
    order?.refundTrackingState === "processed"
      ? "Refund completed."
      : order?.refundTrackingState === "failed"
        ? "Refund failed. Retry refund from payment dashboard."
        : order?.refundTrackingState === "initiated"
          ? "Refund in progress at Razorpay."
          : null;

  return (
    <AdminPageShell
      label="Orders"
      title={orderId ? `Order #${orderId}` : "Order detail"}
      description="Review order data, status transitions, and shipment scheduling controls."
      action={
        <Button variant="outline" size="sm" asChild>
          <Link href="/imtheboss/orders" className="gap-1">
            <ArrowLeft className="h-4 w-4" />
            Back to orders
          </Link>
        </Button>
      }
    >
      {!orderId ? <p className="text-sm text-[var(--color-muted)]">Invalid order link.</p> : null}

      {isLoading && orderId ? (
        <p className="py-12 text-center text-sm text-[var(--color-muted)]">Loading order...</p>
      ) : null}

      {isError && errorUi ? (
        <div className="rounded-md border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
          <p className="font-medium">{errorUi.title}</p>
          <p className="mt-1 text-xs">{errorUi.message}</p>
          <Button type="button" variant="outline" size="sm" className="mt-3" onClick={() => refetch()}>
            Retry
          </Button>
        </div>
      ) : null}

      {notFound ? (
        <div className="rounded-md border border-[var(--color-line)] bg-white px-4 py-8 text-center shadow-[var(--admin-card-shadow)]">
          <p className="text-sm font-medium text-[var(--color-ink)]">Order not found</p>
          <p className="mt-1 text-xs text-[var(--color-muted)]">
            It may have been removed or the id in the URL is wrong.
          </p>
          <Button type="button" variant="outline" size="sm" className="mt-4" asChild>
            <Link href="/imtheboss/orders">Return to orders</Link>
          </Button>
        </div>
      ) : null}

      {order ? (
        <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_360px]">
          <div className="space-y-6">
            <Card className="bg-[var(--admin-surface-muted)]">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <CardTitle className="flex items-center gap-2.5 text-sm font-semibold normal-case tracking-normal text-[var(--color-ink)] md:text-[15px]">
                  <ListOrdered className="h-4 w-4 text-[var(--color-green)]" />
                  Summary
                </CardTitle>
                <StatusBadge label={getStatusLabel(order.statusId, statuses)} />
              </div>
              <CardContent className="mt-4">
                <div className="flex flex-wrap items-baseline gap-x-8 gap-y-1 border-b border-[var(--color-line)] pb-4">
                  <div>
                    <p className="text-sm text-[var(--color-muted)]">Total</p>
                    <p className="text-2xl font-semibold text-[var(--color-ink)]">{order.totalAmountFormatted}</p>
                  </div>
                  <div>
                    <p className="text-sm text-[var(--color-muted)]">Placed</p>
                    <p className="text-base text-[var(--color-ink)]">{formatOrderDate(order.orderDate)}</p>
                  </div>
                </div>
                <dl className="mt-4 grid gap-3 text-sm sm:grid-cols-2">
                  <div>
                    <dt className="text-[var(--color-muted)]">Order ID</dt>
                    <dd className="mt-0.5 text-[var(--color-ink)]">{order.orderId}</dd>
                  </div>
                  <div>
                    <dt className="text-[var(--color-muted)]">Customer ID</dt>
                    <dd className="mt-0.5 text-[var(--color-ink)]">{order.userId}</dd>
                  </div>
                  <div>
                    <dt className="text-[var(--color-muted)]">Shipping address ID</dt>
                    <dd className="mt-0.5 text-[var(--color-ink)]">{order.shippingAddressId}</dd>
                  </div>
                </dl>
                {order.invoiceAvailable ? (
                  <Button asChild variant="outline" size="sm" className="mt-4">
                    <a href={`/api/admin/orders/${encodeURIComponent(order.orderId)}/invoice`}>
                      Download invoice
                    </a>
                  </Button>
                ) : null}
              </CardContent>
            </Card>

            <Card className="bg-[var(--admin-surface-muted)]">
              <CardTitle className="flex items-center gap-2.5 text-sm font-semibold normal-case tracking-normal text-[var(--color-ink)] md:text-[15px]">
                <Package className="h-4 w-4 text-[var(--color-green)]" />
                Line items
              </CardTitle>
              <CardContent className="mt-4">
                {order.lines.length === 0 ? (
                  <p className="py-6 text-center text-sm text-[var(--color-muted)]">No line items.</p>
                ) : (
                  <div className="overflow-x-auto">
                    <table className="w-full min-w-[640px] border-collapse text-[15px]">
                      <caption className="sr-only">Order line items</caption>
                      <thead>
                        <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                          <th className="pb-2 pr-4 font-medium">Product</th>
                          <th className="pb-2 pr-4 font-medium">Variant</th>
                          <th className="pb-2 pr-4 font-medium">Qty</th>
                          <th className="pb-2 font-medium">Line total</th>
                        </tr>
                      </thead>
                      <tbody>
                        {order.lines.map((line) => (
                          <tr key={line.orderDetailId} className="border-b border-[var(--color-line)] last:border-0">
                            <td className="py-3.5 pr-4 text-[var(--color-ink)]">
                              {line.productName ?? "-"}
                              {line.productId ? (
                                <span className="mt-0.5 block text-xs text-[var(--color-muted)]">Product #{line.productId}</span>
                              ) : null}
                            </td>
                            <td className="py-3.5 pr-4 text-xs text-[var(--color-muted)]">{line.variantId}</td>
                            <td className="py-3.5 pr-4 text-[var(--color-ink)]">{line.quantity}</td>
                            <td className="py-3.5 text-[var(--color-ink)]">{line.priceFormatted}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                )}
              </CardContent>
            </Card>

            <OrderShipmentCard orderId={order.orderId} />

            <OrderTimelineCard orderId={order.orderId} events={order.events} />
          </div>

          <div className="space-y-6">
            <Card className="bg-[var(--admin-surface-muted)]">
              <CardTitle className="text-sm font-semibold normal-case tracking-normal text-[var(--color-ink)] md:text-[15px]">
                Change status
              </CardTitle>
              <CardContent className="mt-4">
                <OrderDetailStatusEditor
                  key={`${order.orderId}-${order.statusId}`}
                  order={order}
                  statuses={statuses}
                  orderIdParam={orderId}
                />
                {refundTrackingNote ? (
                  <p className="mt-3 text-sm text-[var(--color-ink)]">{refundTrackingNote}</p>
                ) : null}
              </CardContent>
            </Card>

            <Card className="bg-[var(--admin-surface-muted)]">
              <CardTitle className="text-sm font-semibold normal-case tracking-normal text-[var(--color-ink)] md:text-[15px]">
                Lifecycle timestamps
              </CardTitle>
              <CardContent className="mt-4 space-y-2 text-[15px] text-[var(--color-ink)]">
                <p>
                  Cancel window ends: <span>{order.cancelWindowEndsAt ? formatOrderDate(order.cancelWindowEndsAt) : "N/A"}</span>
                </p>
                <p>
                  Earliest booking at: <span>{order.earliestBookingAt ? formatOrderDate(order.earliestBookingAt) : "N/A"}</span>
                </p>
                <p>
                  Pickup target: <span>{order.pickupTargetAt ? formatOrderDate(order.pickupTargetAt) : "N/A"}</span>
                </p>

                <div className="mt-4 grid gap-3">
                  <label className="text-sm font-medium text-[var(--color-muted)]">
                    Pickup target
                    <input
                      type="datetime-local"
                      value={pickupTargetValue}
                      onChange={(e) => {
                        setPickupTargetDirty(true);
                        setPickupTargetDraft(e.target.value);
                      }}
                      className="mt-1.5 h-11 w-full rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px] text-[var(--color-ink)]"
                    />
                  </label>

                  <label className="text-sm font-medium text-[var(--color-muted)]">
                    Reason
                    <input
                      type="text"
                      value={pickupReasonDraft}
                      onChange={(e) => setPickupReasonDraft(e.target.value)}
                      placeholder="e.g. rescheduled by courier"
                      className="mt-1.5 h-11 w-full rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px] text-[var(--color-ink)]"
                    />
                  </label>

                  <Button
                    type="button"
                    disabled={pickupTargetMutation.isPending || !pickupTargetValue.trim()}
                    onClick={() => pickupTargetMutation.mutate()}
                  >
                    {pickupTargetMutation.isPending ? "Updating..." : "Update pickup target"}
                  </Button>

                  {pickupTargetMutation.isError ? (
                    <p className="text-sm text-rose-700">
                      {pickupTargetMutation.error instanceof Error
                        ? pickupTargetMutation.error.message
                        : "Could not update pickup target"}
                    </p>
                  ) : null}
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      ) : null}
    </AdminPageShell>
  );
}
