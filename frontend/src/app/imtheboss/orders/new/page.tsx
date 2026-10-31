"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useMutation } from "@tanstack/react-query";
import { ArrowLeft } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { Button } from "@/components/ui/button";
import type { CustomerListRow } from "@/lib/admin-queries";
import { placeOrderAdmin } from "@/lib/admin-order-create";
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
  const [paymentMethod, setPaymentMethod] = useState<"cod" | "prepaid">("cod");
  const [lines, setLines] = useState<OrderLineDraft[]>([]);
  const [submitError, setSubmitError] = useState("");

  const createMutation = useMutation({
    mutationFn: async () => {
      if (!selectedCustomer) throw new Error("Select a customer.");
      if (!selectedAddressId) throw new Error("Select or add a shipping address.");
      if (lines.length === 0) throw new Error("Add at least one line item.");

      return placeOrderAdmin({
        userId: selectedCustomer.userId,
        shippingAddressId: selectedAddressId,
        paymentMethod,
        lineItems: lines.map((l) => ({
          variantId: l.variantId,
          quantity: l.quantity,
          pricePaise: String((Number(l.unitPricePaise) || 0) * (Number(l.quantity) || 0)),
        })),
      });
    },
    onSuccess: (orderId) => router.push(`/imtheboss/orders/${orderId}`),
    onError: (err: Error) => setSubmitError(err.message || "Failed to create order."),
  });

  return (
    <AdminPageShell
      label="Orders"
      title="Create order"
      description="Place an order on a customer's behalf (phone/in-person sale, gift, backfill). This follows the exact same confirmation flow as a real order — stock is checked and reserved, the order is immediately confirmed, the payment is marked captured, an invoice is generated, and the customer gets the usual confirmation notification. There's never a live payment step: pick whichever method matches how the customer actually paid."
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

        <AdminTableCard title="Payment method">
          <label className="text-sm text-[var(--color-muted)]">
            How did the customer pay?
            <select
              value={paymentMethod}
              onChange={(e) => setPaymentMethod(e.target.value as "cod" | "prepaid")}
              className="mt-1 block h-11 min-w-[16rem] rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px] text-[var(--color-ink)] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
            >
              <option value="cod">Cash on delivery</option>
              <option value="prepaid">Prepaid — already paid outside the site</option>
            </select>
            <span className="mt-1 block max-w-md text-xs leading-relaxed text-[var(--color-muted)]">
              Neither option collects payment here — there is no live payment step for an
              admin-placed order. Both are recorded as already settled the moment you click
              &ldquo;Create order&rdquo;, exactly like a real cash-on-delivery order; this field
              only records how the customer actually paid, for accounting.
            </span>
          </label>
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
