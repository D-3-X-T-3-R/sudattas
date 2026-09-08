"use client";

import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { CreditCard } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminGroupTabs } from "@/components/admin/admin-group-tabs";
import { FINANCE_GROUP_TABS } from "@/lib/admin-nav-groups";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { StatusBadge } from "@/components/admin/status-badge";
import { searchPaymentIntentsAdmin } from "@/lib/admin-payment-intents";
import { formatInrFromPaise } from "@/lib/money";

const STATUS_OPTIONS = [
  { value: "", label: "All statuses" },
  { value: "pending", label: "Pending" },
  { value: "processed", label: "Processed" },
  { value: "client_verified", label: "Client verified" },
  { value: "needs_review", label: "Needs review" },
  { value: "failed", label: "Failed" },
];

function formatDate(raw: string): string {
  if (!raw) return "—";
  const d = new Date(raw);
  if (Number.isNaN(d.getTime())) return raw;
  return d.toLocaleString("en-IN", {
    day: "2-digit",
    month: "short",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export default function AdminPaymentsPage() {
  const [orderId, setOrderId] = useState("");
  const [razorpayPaymentId, setRazorpayPaymentId] = useState("");
  const [status, setStatus] = useState("");
  const [appliedFilters, setAppliedFilters] = useState<{
    orderId?: string;
    razorpayPaymentId?: string;
    status?: string;
  }>({});

  const paymentsQuery = useQuery({
    queryKey: ["admin", "payment-intents", appliedFilters],
    queryFn: () => searchPaymentIntentsAdmin(appliedFilters),
  });

  const handleSearch = () => {
    setAppliedFilters({
      orderId: orderId.trim() || undefined,
      razorpayPaymentId: razorpayPaymentId.trim() || undefined,
      status: status || undefined,
    });
  };

  const handleClear = () => {
    setOrderId("");
    setRazorpayPaymentId("");
    setStatus("");
    setAppliedFilters({});
  };

  const payments = paymentsQuery.data ?? [];

  return (
    <AdminPageShell
      label="Payments"
      title="Payment intents"
      description="Every real Razorpay checkout attempt — the gateway-side record, distinct from the manually-maintained Transactions ledger. Search by order, Razorpay payment ID, or status."
    >
      <AdminGroupTabs tabs={FINANCE_GROUP_TABS} />
      <AdminTableCard
        title="Payment intents"
        icon={<CreditCard className="h-4 w-4 text-[var(--color-green)]" />}
      >
        <div className="mb-4 flex flex-wrap items-end gap-2.5">
          <div>
            <label htmlFor="admin-payments-order-id" className="mb-1.5 block text-sm font-medium text-[var(--color-muted)]">
              Order ID
            </label>
            <Input
              id="admin-payments-order-id"
              value={orderId}
              onChange={(e) => setOrderId(e.target.value)}
              placeholder="e.g. 120"
              className="h-10 w-32 rounded-lg text-[15px]"
            />
          </div>
          <div>
            <label htmlFor="admin-payments-razorpay-id" className="mb-1.5 block text-sm font-medium text-[var(--color-muted)]">
              Razorpay payment ID
            </label>
            <Input
              id="admin-payments-razorpay-id"
              value={razorpayPaymentId}
              onChange={(e) => setRazorpayPaymentId(e.target.value)}
              placeholder="pay_..."
              className="h-10 w-48 rounded-lg text-[15px]"
            />
          </div>
          <div>
            <label htmlFor="admin-payments-status" className="mb-1.5 block text-sm font-medium text-[var(--color-muted)]">
              Status
            </label>
            <select
              id="admin-payments-status"
              value={status}
              onChange={(e) => setStatus(e.target.value)}
              className="h-10 rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
            >
              {STATUS_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </select>
          </div>
          <Button type="button" size="sm" onClick={handleSearch}>
            Search
          </Button>
          <Button type="button" size="sm" variant="outline" onClick={handleClear}>
            Clear
          </Button>
        </div>

        {paymentsQuery.isLoading ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading payment intents…</p>
        ) : null}
        {paymentsQuery.isError ? (
          <p className="py-8 text-center text-sm text-rose-700">Could not load payment intents.</p>
        ) : null}
        {!paymentsQuery.isLoading && !paymentsQuery.isError && payments.length === 0 ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">No payment intents match these filters.</p>
        ) : null}

        {!paymentsQuery.isLoading && !paymentsQuery.isError && payments.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full min-w-[900px] border-collapse text-[15px]">
              <caption className="sr-only">Payment intents</caption>
              <thead>
                <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                  <th className="pb-2 pr-4 font-medium">Intent</th>
                  <th className="pb-2 pr-4 font-medium">Order</th>
                  <th className="pb-2 pr-4 font-medium">Razorpay payment ID</th>
                  <th className="pb-2 pr-4 font-medium">Amount</th>
                  <th className="pb-2 pr-4 font-medium">Gateway fee/tax</th>
                  <th className="pb-2 pr-4 font-medium">Status</th>
                  <th className="pb-2 font-medium">Created</th>
                </tr>
              </thead>
              <tbody>
                {payments.map((p) => (
                  <tr key={p.intentId} className="border-b border-[var(--color-line)] last:border-0 align-top">
                    <td className="py-3 pr-4 text-[var(--color-ink)]">#{p.intentId}</td>
                    <td className="py-3 pr-4 text-[var(--color-muted)]">{p.orderId ? `#${p.orderId}` : "—"}</td>
                    <td className="max-w-[14rem] truncate py-3 pr-4 text-xs text-[var(--color-muted)]" title={p.razorpayPaymentId ?? undefined}>
                      {p.razorpayPaymentId ?? "—"}
                    </td>
                    <td className="py-3 pr-4 text-[var(--color-ink)]">{formatInrFromPaise(p.amountPaise)}</td>
                    <td className="py-3 pr-4 text-xs text-[var(--color-muted)]">
                      {p.gatewayFeePaise || p.gatewayTaxPaise
                        ? `${formatInrFromPaise(p.gatewayFeePaise ?? "0")} / ${formatInrFromPaise(p.gatewayTaxPaise ?? "0")}`
                        : "—"}
                    </td>
                    <td className="py-3 pr-4">
                      <StatusBadge label={p.status} />
                    </td>
                    <td className="py-3 text-[var(--color-muted)]">{formatDate(p.createdAt)}</td>
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
