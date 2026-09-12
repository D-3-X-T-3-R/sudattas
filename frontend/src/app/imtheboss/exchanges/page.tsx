"use client";

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Repeat } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminGroupTabs } from "@/components/admin/admin-group-tabs";
import { ORDERS_GROUP_TABS } from "@/lib/admin-nav-groups";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/admin/status-badge";
import {
  adminMarkExchangeReceived,
  adminUpdateExchangeStatus,
  fetchExchangeRequestsAdmin,
  scheduleExchangePickup,
  syncExchangePickup,
  type ExchangeRequestRow,
} from "@/lib/admin-exchanges";

const OPEN_STATUSES = ["requested", "approved", "in_transit"];

function formatDate(raw: string): string {
  if (!raw) return "—";
  const d = new Date(raw);
  if (Number.isNaN(d.getTime())) return raw;
  return d.toLocaleDateString("en-IN", { day: "2-digit", month: "short", year: "numeric" });
}

export default function AdminExchangesPage() {
  const queryClient = useQueryClient();
  const [rowError, setRowError] = useState<Record<string, string>>({});

  const exchangesQuery = useQuery({
    queryKey: ["admin", "exchanges"],
    queryFn: fetchExchangeRequestsAdmin,
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["admin", "exchanges"] });
  const setError = (exchangeId: string, message: string) =>
    setRowError((prev) => ({ ...prev, [exchangeId]: message }));

  const receivedMutation = useMutation({
    mutationFn: (exchangeId: string) => adminMarkExchangeReceived(exchangeId),
    onSuccess: (_, exchangeId) => {
      invalidate();
      setError(exchangeId, "");
    },
    onError: (err: Error, exchangeId) =>
      setError(exchangeId, err.message || "Failed to mark received."),
  });

  const statusMutation = useMutation({
    mutationFn: (params: {
      exchangeId: string;
      status: "approved" | "in_transit" | "rejected" | "cancelled";
    }) => adminUpdateExchangeStatus(params),
    onSuccess: (_, { exchangeId }) => {
      invalidate();
      setError(exchangeId, "");
    },
    onError: (err: Error, { exchangeId }) =>
      setError(exchangeId, err.message || "Failed to update exchange."),
  });

  const schedulePickupMutation = useMutation({
    mutationFn: (exchangeId: string) => scheduleExchangePickup(exchangeId),
    onSuccess: (_, exchangeId) => {
      invalidate();
      setError(exchangeId, "");
    },
    onError: (err: Error, exchangeId) =>
      setError(exchangeId, err.message || "Failed to schedule pickup."),
  });

  const syncPickupMutation = useMutation({
    mutationFn: (exchangeId: string) => syncExchangePickup(exchangeId),
    onSuccess: (_, exchangeId) => {
      invalidate();
      setError(exchangeId, "");
    },
    onError: (err: Error, exchangeId) =>
      setError(exchangeId, err.message || "Failed to refresh pickup tracking."),
  });

  const exchanges = exchangesQuery.data ?? [];

  return (
    <AdminPageShell
      label="Exchanges"
      title="Exchange requests"
      description="Category-scoped exchanges — same product, different size, same price. Schedule a reverse pickup so Shiprocket collects the original item from the customer; syncing its tracking auto-completes the exchange (stock restored, $0 replacement order created) once it's delivered back to the warehouse."
    >
      <AdminGroupTabs tabs={ORDERS_GROUP_TABS} />
      <AdminTableCard title="Exchange requests" icon={<Repeat className="h-4 w-4 text-[var(--color-green)]" />}>
        {exchangesQuery.isLoading ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading exchanges…</p>
        ) : null}
        {exchangesQuery.isError ? (
          <p className="py-8 text-center text-sm text-rose-700">Could not load exchange requests.</p>
        ) : null}
        {!exchangesQuery.isLoading && !exchangesQuery.isError && exchanges.length === 0 ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">No exchange requests yet.</p>
        ) : null}

        {!exchangesQuery.isLoading && !exchangesQuery.isError && exchanges.length > 0 ? (
          <div className="space-y-3">
            {exchanges.map((ex: ExchangeRequestRow) => {
              const isOpen = OPEN_STATUSES.includes(ex.status);
              // "received" with no replacement order yet means the replacement-order step
              // failed (e.g. desired size sold out) — admin_mark_exchange_received is safe
              // to retry from here, so keep the retry action available.
              const needsReceivedRetry = ex.status === "received" && !ex.replacementOrderId;
              const canSchedulePickup =
                (ex.status === "approved" || ex.status === "in_transit") && !ex.pickupAwbCode;
              const canSyncPickup = !!ex.pickupAwbCode && !ex.replacementOrderId;
              const pending =
                receivedMutation.isPending ||
                statusMutation.isPending ||
                schedulePickupMutation.isPending ||
                syncPickupMutation.isPending;
              return (
                <div key={ex.exchangeId} className="rounded-lg border border-[var(--color-line)] p-3.5">
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div>
                      <p className="text-[15px] font-medium text-[var(--color-ink)]">
                        Exchange #{ex.exchangeId} · Order #{ex.orderId}
                      </p>
                      <p className="mt-0.5 text-sm text-[var(--color-muted)]">
                        Customer #{ex.userId} · {formatDate(ex.createdAt)}
                      </p>
                      <p className="mt-1 text-sm text-[var(--color-ink)]">{ex.reason}</p>
                    </div>
                    <StatusBadge label={ex.status} />
                  </div>

                  <div className="mt-2.5 overflow-x-auto">
                    <table className="w-full min-w-[420px] border-collapse text-sm">
                      <thead>
                        <tr className="border-b border-[var(--color-line)] text-left text-[var(--color-muted)]">
                          <th className="pb-1.5 pr-4 font-medium">Order item</th>
                          <th className="pb-1.5 pr-4 font-medium">Qty</th>
                          <th className="pb-1.5 pr-4 font-medium">Desired variant</th>
                          <th className="pb-1.5 font-medium">Replacement order</th>
                        </tr>
                      </thead>
                      <tbody>
                        <tr>
                          <td className="py-1.5 pr-4 text-[var(--color-ink)]">#{ex.orderDetailId}</td>
                          <td className="py-1.5 pr-4 text-[var(--color-muted)]">{ex.quantity}</td>
                          <td className="py-1.5 pr-4 text-[var(--color-ink)]">#{ex.desiredVariantId}</td>
                          <td className="py-1.5 text-[var(--color-muted)]">
                            {ex.replacementOrderId ? `#${ex.replacementOrderId}` : "—"}
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>

                  {ex.pickupAwbCode ? (
                    <div className="mt-2.5 rounded-md border border-[var(--color-line)] bg-[var(--color-surface-soft)] p-2.5 text-sm">
                      <p className="font-medium text-[var(--color-ink)]">
                        Reverse pickup: AWB {ex.pickupAwbCode}
                        {ex.pickupCourierName ? ` via ${ex.pickupCourierName}` : ""}
                      </p>
                      <p className="mt-0.5 text-[var(--color-muted)]">
                        Status: {ex.pickupStatus ?? "Booked"}
                        {ex.pickupScheduledAt ? ` · Booked ${formatDate(ex.pickupScheduledAt)}` : ""}
                      </p>
                    </div>
                  ) : null}

                  {isOpen || needsReceivedRetry ? (
                    <div className="mt-3 flex flex-wrap gap-1.5">
                      {ex.status === "requested" && (
                        <Button
                          type="button"
                          size="sm"
                          variant="outline"
                          disabled={pending}
                          onClick={() =>
                            statusMutation.mutate({ exchangeId: ex.exchangeId, status: "approved" })
                          }
                        >
                          Approve
                        </Button>
                      )}
                      {(ex.status === "requested" || ex.status === "approved") && (
                        <Button
                          type="button"
                          size="sm"
                          variant="outline"
                          disabled={pending}
                          onClick={() =>
                            statusMutation.mutate({ exchangeId: ex.exchangeId, status: "in_transit" })
                          }
                        >
                          Mark in transit
                        </Button>
                      )}
                      {canSchedulePickup && (
                        <Button
                          type="button"
                          size="sm"
                          variant="outline"
                          disabled={pending}
                          onClick={() => schedulePickupMutation.mutate(ex.exchangeId)}
                        >
                          {schedulePickupMutation.isPending ? "Scheduling pickup…" : "Schedule pickup"}
                        </Button>
                      )}
                      {canSyncPickup && (
                        <Button
                          type="button"
                          size="sm"
                          variant="outline"
                          disabled={pending}
                          onClick={() => syncPickupMutation.mutate(ex.exchangeId)}
                        >
                          {syncPickupMutation.isPending ? "Syncing…" : "Sync pickup"}
                        </Button>
                      )}
                      <Button
                        type="button"
                        size="sm"
                        disabled={pending}
                        onClick={() => receivedMutation.mutate(ex.exchangeId)}
                      >
                        {receivedMutation.isPending ? "Marking received…" : "Mark received"}
                      </Button>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        className="border-red-200 text-red-600 hover:bg-red-50"
                        disabled={pending}
                        onClick={() =>
                          statusMutation.mutate({ exchangeId: ex.exchangeId, status: "rejected" })
                        }
                      >
                        Reject
                      </Button>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        className="border-red-200 text-red-600 hover:bg-red-50"
                        disabled={pending}
                        onClick={() =>
                          statusMutation.mutate({ exchangeId: ex.exchangeId, status: "cancelled" })
                        }
                      >
                        Cancel
                      </Button>
                    </div>
                  ) : null}
                  {needsReceivedRetry ? (
                    <p className="mt-2 text-sm text-amber-700">
                      Item received but the replacement order hasn&apos;t been created yet (likely the
                      desired size is out of stock) — click &ldquo;Mark received&rdquo; again to retry
                      once stock is available.
                    </p>
                  ) : null}
                  {rowError[ex.exchangeId] && (
                    <p className="mt-2 text-sm text-red-600" role="alert">
                      {rowError[ex.exchangeId]}
                    </p>
                  )}
                </div>
              );
            })}
          </div>
        ) : null}
      </AdminTableCard>
    </AdminPageShell>
  );
}
