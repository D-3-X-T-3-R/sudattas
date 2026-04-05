"use client";

import { useQuery } from "@tanstack/react-query";
import Link from "next/link";
import { useParams } from "next/navigation";
import { ArrowLeft, ListOrdered, Package } from "lucide-react";
import { SectionHeading } from "@/components/ui/typography";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { fetchOrderStatuses } from "@/lib/admin-queries";
import { fetchAdminOrderById, type AdminOrderDetail } from "@/lib/admin-order-detail";
import { OrderDetailStatusEditor } from "@/domains/admin/orders/components/order-detail-status-editor";
import { toRouteFailureUi } from "@/lib/route-state";
import { formatOrderDate } from "@/domains/admin/orders/utils";

export default function AdminOrderDetailPage() {
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

  const errorUi = isError ? toRouteFailureUi("admin", error) : null;
  const notFound = !isLoading && !isError && orderId && order == null;

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
