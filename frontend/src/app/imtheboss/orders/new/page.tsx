"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useMutation, useQuery } from "@tanstack/react-query";
import { ArrowLeft } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { Button } from "@/components/ui/button";
import { fetchOrderStatuses, type CustomerListRow } from "@/lib/admin-queries";
import { createOrderAdmin, createOrderDetailsAdmin } from "@/lib/admin-order-create";
import {
  OrderCreateCustomerCard,
} from "@/domains/admin/orders/components/order-create-customer-card";
import {
  OrderCreateLineItems,
  type OrderLineDraft,
} from "@/domains/admin/orders/components/order-create-line-items";

export default function AdminCreateOrderPage() {
  const router = useRouter();
  const [selectedCustomer, setSelectedCustomer] = useState<CustomerListRow | null>(null);
  const [selectedAddressId, setSelectedAddressId] = useState("");
  const [statusId, setStatusId] = useState("");
  const [lines, setLines] = useState<OrderLineDraft[]>([]);
  const [submitError, setSubmitError] = useState("");

  const { data: statuses = [] } = useQuery({
    queryKey: ["admin", "order-statuses"],
    queryFn: fetchOrderStatuses,
  });

  const subtotalPaise = lines.reduce(
    (sum, l) => sum + (Number(l.unitPricePaise) || 0) * (Number(l.quantity) || 0),
    0
  );

  const createMutation = useMutation({
    mutationFn: async () => {
      if (!selectedCustomer) throw new Error("Select a customer.");
      if (!selectedAddressId) throw new Error("Select or add a shipping address.");
      if (!statusId) throw new Error("Select an order status.");
      if (lines.length === 0) throw new Error("Add at least one line item.");

      const totalPaise = String(subtotalPaise);
      const orderId = await createOrderAdmin({
        userId: selectedCustomer.userId,
        shippingAddressId: selectedAddressId,
        statusId,
        totalAmountPaise: totalPaise,
        subtotalMinor: totalPaise,
        shippingMinor: "0",
        grandTotalMinor: totalPaise,
      });
      await createOrderDetailsAdmin(
        orderId,
        lines.map((l) => ({
          variantId: l.variantId,
          quantity: l.quantity,
          pricePaise: String((Number(l.unitPricePaise) || 0) * (Number(l.quantity) || 0)),
        }))
      );
      return orderId;
    },
    onSuccess: (orderId) => router.push(`/imtheboss/orders/${orderId}`),
    onError: (err: Error) => setSubmitError(err.message || "Failed to create order."),
  });

  return (
    <AdminPageShell
      label="Orders"
      title="Create order"
      description="Manually record an order — this does not decrement stock, charge payment, or send a confirmation email. Use it for phone orders, gifts, or backfilling records only."
      action={
        <Button variant="outline" size="sm" asChild>
          <Link href="/imtheboss/orders" className="gap-1">
            <ArrowLeft className="h-4 w-4" />
            Back to orders
          </Link>
        </Button>
      }
    >
      <div className="space-y-6">
        <OrderCreateCustomerCard
          selectedCustomer={selectedCustomer}
          setSelectedCustomer={setSelectedCustomer}
          selectedAddressId={selectedAddressId}
          setSelectedAddressId={setSelectedAddressId}
        />

        <AdminTableCard title="Status">
          <select
            value={statusId}
            onChange={(e) => setStatusId(e.target.value)}
            className="h-11 min-w-[13rem] rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
          >
            <option value="">Select status…</option>
            {statuses.map((s) => (
              <option key={s.statusId} value={s.statusId}>
                {s.statusName}
              </option>
            ))}
          </select>
        </AdminTableCard>

        <OrderCreateLineItems
          lines={lines}
          onAdd={(line) => setLines((prev) => [...prev, line])}
          onRemove={(key) => setLines((prev) => prev.filter((l) => l.key !== key))}
        />

        <div className="sticky bottom-4 z-10 flex items-center gap-3 rounded-xl border border-[var(--color-line)] bg-white/95 p-3 shadow-[0_8px_24px_rgba(45,42,38,0.12)] backdrop-blur">
          <Button
            type="button"
            disabled={createMutation.isPending}
            onClick={() => createMutation.mutate()}
          >
            {createMutation.isPending ? "Creating…" : "Create order"}
          </Button>
          {submitError && (
            <p className="text-sm text-rose-700" role="alert">
              {submitError}
            </p>
          )}
        </div>
      </div>
    </AdminPageShell>
  );
}
