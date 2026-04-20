"use client";

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { ArrowLeft, ListOrdered, Package } from "lucide-react";
import { SectionHeading } from "@/components/ui/typography";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { fetchOrderStatuses } from "@/lib/admin-queries";
import {
  fetchAdminOrderById,
  updateAdminPickupTarget,
  type AdminOrderDetail,
} from "@/lib/admin-order-detail";
import { OrderDetailStatusEditor } from "@/domains/admin/orders/components/order-detail-status-editor";
import { toRouteFailureUi } from "@/lib/route-state";
import { formatOrderDate } from "@/domains/admin/orders/utils";

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

  useEffect(() => {
    if (!order) return;
    console.info("[orders-flow][admin-ui] admin order page state", {
      orderId: order.orderId,
      statusId: order.statusId,
      refundTrackingState: order.refundTrackingState,
      refundTrackingNote,
    });
  }, [order, refundTrackingNote]);

  return (
    <div className="mx-auto w-full max-w-6xl">
      <div className="mb-6">
        <Button variant="ghost" size="sm" className="mb-4 -ml-2 gap-1 text-[var(--color-muted)]" asChild>
          <Link href="/imtheboss/orders">
            <ArrowLeft className="h-4 w-4" />
            Back to orders
          </Link>
        </Button>
        <p className="text-sm text-[var(--color-muted)]">Orders</p>
        <SectionHeading size="default" className="mt-1">
          Order {orderId ? `#${orderId}` : "detail"}
        </SectionHeading>
        <p className="mt-1 text-sm leading-relaxed text-[var(--color-muted)]">
          Summary, shipping address id, and line items for this order.
        </p>
      </div>

      {!orderId && (
        <p className="text-sm text-[var(--color-muted)]">Invalid order link.</p>
      )}

      {isLoading && orderId && (
        <p className="py-12 text-center text-sm text-[var(--color-muted)]">Loading order…</p>
      )}

      {isError && errorUi && (
        <div className="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
          <p className="font-medium">{errorUi.title}</p>
          <p className="mt-1 text-xs">{errorUi.message}</p>
          <Button type="button" variant="outline" size="sm" className="mt-3" onClick={() => refetch()}>
            Retry
          </Button>
        </div>
      )}

      {notFound && (
        <div className="rounded-lg border border-[var(--color-line)] bg-white px-4 py-8 text-center shadow-[var(--admin-card-shadow)]">
          <p className="text-sm font-medium text-[var(--color-ink)]">Order not found</p>
          <p className="mt-1 text-xs text-[var(--color-muted)]">
            It may have been removed or the id in the URL is wrong.
          </p>
          <Button type="button" variant="outline" size="sm" className="mt-4" asChild>
            <Link href="/imtheboss/orders">Return to orders</Link>
          </Button>
        </div>
      )}

      {order && (
        <div className="space-y-6">
          <Card className="rounded-xl border-[var(--color-line)] border-l-4 border-l-blue-500 bg-white shadow-[var(--admin-card-shadow)]">
            <CardTitle className="flex items-center gap-2 text-[var(--color-muted)]">
              <ListOrdered className="h-4 w-4 text-blue-500" />
              Summary
            </CardTitle>
            <CardContent className="mt-3">
              <dl className="grid gap-3 text-sm sm:grid-cols-2">
                <div>
                  <dt className="text-[var(--color-muted)]">Order ID</dt>
                  <dd className="mt-0.5 font-mono text-[var(--color-ink)]">{order.orderId}</dd>
                </div>
                <div>
                  <dt className="text-[var(--color-muted)]">Placed</dt>
                  <dd className="mt-0.5 text-[var(--color-ink)]">{formatOrderDate(order.orderDate)}</dd>
                </div>
                <div>
                  <dt className="text-[var(--color-muted)]">Customer (user ID)</dt>
                  <dd className="mt-0.5 font-mono text-[var(--color-ink)]">{order.userId}</dd>
                </div>
                <OrderDetailStatusEditor
                  key={`${order.orderId}-${order.statusId}`}
                  order={order}
                  statuses={statuses}
                  orderIdParam={orderId}
                />
                {refundTrackingNote ? (
                  <div className="sm:col-span-2">
                    <dt className="text-[var(--color-muted)]">Track refund</dt>
                    <dd className="mt-0.5 text-sm text-[var(--color-ink)]">{refundTrackingNote}</dd>
                  </div>
                ) : null}
                <div className="sm:col-span-2">
                  <dt className="text-[var(--color-muted)]">Lifecycle timestamps</dt>
                  <dd className="mt-0.5 space-y-1 text-sm text-[var(--color-ink)]">
                    <p>
                      Cancel window ends:{" "}
                      <span className="font-mono">
                        {order.cancelWindowEndsAt ? formatOrderDate(order.cancelWindowEndsAt) : "N/A"}
                      </span>
                    </p>
                    <p>
                      Earliest booking at:{" "}
                      <span className="font-mono">
                        {order.earliestBookingAt ? formatOrderDate(order.earliestBookingAt) : "N/A"}
                      </span>
                    </p>
                    <p>
                      Pickup target:{" "}
                      <span className="font-mono">
                        {order.pickupTargetAt ? formatOrderDate(order.pickupTargetAt) : "N/A"}
                      </span>
                    </p>
                  </dd>
                  <div className="mt-3 grid gap-2 sm:grid-cols-[minmax(12rem,1fr)_minmax(12rem,1fr)_auto] sm:items-end">
                    <label className="text-xs text-[var(--color-muted)]">
                      Pickup target
                      <input
                        type="datetime-local"
                        value={pickupTargetValue}
                        onChange={(e) => {
                          setPickupTargetDirty(true);
                          setPickupTargetDraft(e.target.value);
                        }}
                        className="mt-1 h-9 w-full rounded-md border border-[var(--color-line)] bg-white px-2 text-sm text-[var(--color-ink)]"
                      />
                    </label>
                    <label className="text-xs text-[var(--color-muted)]">
                      Reason
                      <input
                        type="text"
                        value={pickupReasonDraft}
                        onChange={(e) => setPickupReasonDraft(e.target.value)}
                        placeholder="ops reprioritization"
                        className="mt-1 h-9 w-full rounded-md border border-[var(--color-line)] bg-white px-2 text-sm text-[var(--color-ink)]"
                      />
                    </label>
                    <Button
                      type="button"
                      size="sm"
                      disabled={pickupTargetMutation.isPending || !pickupTargetValue.trim()}
                      onClick={() => pickupTargetMutation.mutate()}
                    >
                      {pickupTargetMutation.isPending ? "Updating..." : "Update pickup target"}
                    </Button>
                  </div>
                  {pickupTargetMutation.isError ? (
                    <p className="mt-2 text-xs text-rose-700">
                      {pickupTargetMutation.error instanceof Error
                        ? pickupTargetMutation.error.message
                        : "Could not update pickup target"}
                    </p>
                  ) : null}
                </div>
                <div>
                  <dt className="text-[var(--color-muted)]">Shipping address ID</dt>
                  <dd className="mt-0.5 font-mono text-[var(--color-ink)]">{order.shippingAddressId}</dd>
                </div>
                <div>
                  <dt className="text-[var(--color-muted)]">Total</dt>
                  <dd className="mt-0.5 font-medium text-[var(--color-ink)]">
                    {order.totalAmountFormatted}
                  </dd>
                </div>
              </dl>
            </CardContent>
          </Card>

          <Card className="rounded-xl border-[var(--color-line)] border-l-4 border-l-violet-500 bg-white shadow-[var(--admin-card-shadow)]">
            <CardTitle className="flex items-center gap-2 text-[var(--color-muted)]">
              <Package className="h-4 w-4 text-violet-500" />
              Line items
            </CardTitle>
            <CardContent className="mt-3">
              {order.lines.length === 0 ? (
                <p className="py-6 text-center text-sm text-[var(--color-muted)]">No line items.</p>
              ) : (
                <div className="overflow-x-auto">
                  <table className="w-full min-w-[640px] border-collapse text-sm">
                    <caption className="sr-only">Order line items</caption>
                    <thead>
                      <tr className="border-b border-[var(--color-line)] text-left text-[var(--color-muted)]">
                        <th className="pb-2 pr-4 font-medium">Product</th>
                        <th className="pb-2 pr-4 font-medium">Variant ID</th>
                        <th className="pb-2 pr-4 font-medium">Qty</th>
                        <th className="pb-2 font-medium">Line total</th>
                      </tr>
                    </thead>
                    <tbody>
                      {order.lines.map((line) => (
                        <tr
                          key={line.orderDetailId}
                          className="border-b border-[var(--color-line)] last:border-0"
                        >
                          <td className="py-3 pr-4 text-[var(--color-ink)]">
                            {line.productName ?? "—"}
                            {line.productId ? (
                              <span className="mt-0.5 block font-mono text-xs text-[var(--color-muted)]">
                                Product #{line.productId}
                              </span>
                            ) : null}
                          </td>
                          <td className="py-3 pr-4 font-mono text-[var(--color-muted)]">{line.variantId}</td>
                          <td className="py-3 pr-4 text-[var(--color-ink)]">{line.quantity}</td>
                          <td className="py-3 text-[var(--color-ink)]">{line.priceFormatted}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
}
