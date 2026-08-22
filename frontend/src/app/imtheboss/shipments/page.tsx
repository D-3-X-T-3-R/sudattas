"use client";

import Link from "next/link";
import { useQuery } from "@tanstack/react-query";
import { Truck } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { StatusBadge } from "@/components/admin/status-badge";
import { fetchOrdersList } from "@/lib/admin-queries";
import { fetchShipmentsForOrder, type AdminShipmentRow } from "@/lib/admin-shipments";
import { formatOrderDate } from "@/domains/admin/orders/utils";

/** Bounded sweep of recent orders to build a fleet-wide tracking view — get_shipment requires an
 * order_id (no "list all shipments" query exists server-side), so this fans out one lookup per
 * recent order. Fine at boutique order volume; would need a real backend list query to scale
 * further, same caveat as the other admin screens built this way (inventory, taxonomy). */
const RECENT_ORDERS_LIMIT = "150";

interface ShipmentWithOrder extends AdminShipmentRow {
  customerUserId: string;
}

export default function AdminShipmentsPage() {
  const ordersQuery = useQuery({
    queryKey: ["admin", "shipments-recent-orders"],
    queryFn: () => fetchOrdersList({ limit: RECENT_ORDERS_LIMIT }),
  });

  const shipmentsQuery = useQuery({
    queryKey: ["admin", "shipments-all", ordersQuery.data?.map((o) => o.orderId).join(",")],
    queryFn: async (): Promise<ShipmentWithOrder[]> => {
      const orders = ordersQuery.data ?? [];
      const results = await Promise.all(
        orders.map(async (o) => {
          const shipments = await fetchShipmentsForOrder(o.orderId);
          return shipments.map((s) => ({ ...s, customerUserId: o.userId }));
        })
      );
      return results
        .flat()
        .sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
    },
    enabled: !!ordersQuery.data,
  });

  const isLoading = ordersQuery.isLoading || shipmentsQuery.isLoading;
  const isError = ordersQuery.isError || shipmentsQuery.isError;
  const shipments = shipmentsQuery.data ?? [];

  return (
    <AdminPageShell
      label="Shipments"
      title="Shipments"
      description={`Tracking across the ${RECENT_ORDERS_LIMIT} most recent orders. Create or edit a shipment from its order page.`}
    >
      <AdminTableCard title="Shipments" icon={<Truck className="h-4 w-4 text-[var(--color-green)]" />}>
        {isLoading ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading shipments…</p>
        ) : null}
        {isError ? (
          <p className="py-8 text-center text-sm text-rose-700">Could not load shipments.</p>
        ) : null}
        {!isLoading && !isError && shipments.length === 0 ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">
            No shipments created yet among recent orders.
          </p>
        ) : null}

        {!isLoading && !isError && shipments.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full min-w-[720px] border-collapse text-[15px]">
              <caption className="sr-only">Shipments across recent orders</caption>
              <thead>
                <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                  <th className="pb-2 pr-4 font-medium">Order</th>
                  <th className="pb-2 pr-4 font-medium">Customer</th>
                  <th className="pb-2 pr-4 font-medium">AWB</th>
                  <th className="pb-2 pr-4 font-medium">Carrier</th>
                  <th className="pb-2 pr-4 font-medium">Status</th>
                  <th className="pb-2 font-medium">Created</th>
                </tr>
              </thead>
              <tbody>
                {shipments.map((s) => (
                  <tr key={s.shipmentId} className="border-b border-[var(--color-line)] last:border-0">
                    <td className="py-3 pr-4">
                      <Link
                        href={`/imtheboss/orders/${s.orderId}`}
                        className="font-medium text-[var(--color-green)] hover:underline"
                      >
                        #{s.orderId}
                      </Link>
                    </td>
                    <td className="py-3 pr-4 text-[var(--color-muted)]">{s.customerUserId}</td>
                    <td className="py-3 pr-4 text-[var(--color-ink)]">{s.awbCode || "—"}</td>
                    <td className="py-3 pr-4 text-[var(--color-ink)]">{s.carrier || "—"}</td>
                    <td className="py-3 pr-4">
                      <StatusBadge label={s.customerTrackingStatus || s.status} />
                    </td>
                    <td className="py-3 text-[var(--color-muted)]">{formatOrderDate(s.createdAt)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : null}
      </AdminTableCard>
    </AdminPageShell>
  );
}
