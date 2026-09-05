"use client";

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Undo2 } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/admin/status-badge";
import {
  adminMarkReturnReceived,
  adminUpdateReturnStatus,
  fetchReturnRequestsAdmin,
  type ReturnRequestRow,
} from "@/lib/admin-returns";
import { formatInrFromPaise } from "@/lib/money";

const OPEN_STATUSES = ["requested", "approved", "in_transit"];

function formatDate(raw: string): string {
  if (!raw) return "—";
  const d = new Date(raw);
  if (Number.isNaN(d.getTime())) return raw;
  return d.toLocaleDateString("en-IN", { day: "2-digit", month: "short", year: "numeric" });
}

export default function AdminReturnsPage() {
  const queryClient = useQueryClient();
  const [rowError, setRowError] = useState<Record<string, string>>({});

  const returnsQuery = useQuery({
    queryKey: ["admin", "returns"],
    queryFn: fetchReturnRequestsAdmin,
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["admin", "returns"] });
  const setError = (returnId: string, message: string) =>
    setRowError((prev) => ({ ...prev, [returnId]: message }));

  const receivedMutation = useMutation({
    mutationFn: (returnId: string) => adminMarkReturnReceived(returnId),
    onSuccess: (_, returnId) => {
      invalidate();
      setError(returnId, "");
    },
    onError: (err: Error, returnId) => setError(returnId, err.message || "Failed to mark received."),
  });

  const statusMutation = useMutation({
    mutationFn: (params: { returnId: string; status: "approved" | "in_transit" | "rejected" | "cancelled" }) =>
      adminUpdateReturnStatus(params),
    onSuccess: (_, { returnId }) => {
      invalidate();
      setError(returnId, "");
    },
    onError: (err: Error, { returnId }) => setError(returnId, err.message || "Failed to update return."),
  });

  const returns = returnsQuery.data ?? [];

  return (
    <AdminPageShell
      label="Returns"
      title="Returns approval queue"
      description="Review, approve, and process customer return requests. Marking an item received starts the refund against the original payment automatically."
    >
      <AdminTableCard title="Return requests" icon={<Undo2 className="h-4 w-4 text-[var(--color-green)]" />}>
        {returnsQuery.isLoading ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading returns…</p>
        ) : null}
        {returnsQuery.isError ? (
          <p className="py-8 text-center text-sm text-rose-700">Could not load return requests.</p>
        ) : null}
        {!returnsQuery.isLoading && !returnsQuery.isError && returns.length === 0 ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">No return requests yet.</p>
        ) : null}

        {!returnsQuery.isLoading && !returnsQuery.isError && returns.length > 0 ? (
          <div className="space-y-3">
            {returns.map((r: ReturnRequestRow) => {
              const isOpen = OPEN_STATUSES.includes(r.status);
              const pending = receivedMutation.isPending || statusMutation.isPending;
              return (
                <div
                  key={r.returnId}
                  className="rounded-lg border border-[var(--color-line)] p-3.5"
                >
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div>
                      <p className="text-[15px] font-medium text-[var(--color-ink)]">
                        Return #{r.returnId} · Order #{r.orderId}
                      </p>
                      <p className="mt-0.5 text-sm text-[var(--color-muted)]">
                        Customer #{r.userId} · {formatDate(r.createdAt)}
                      </p>
                      <p className="mt-1 text-sm text-[var(--color-ink)]">{r.reason}</p>
                    </div>
                    <StatusBadge label={r.status} />
                  </div>

                  <div className="mt-2.5 overflow-x-auto">
                    <table className="w-full min-w-[420px] border-collapse text-sm">
                      <thead>
                        <tr className="border-b border-[var(--color-line)] text-left text-[var(--color-muted)]">
                          <th className="pb-1.5 pr-4 font-medium">Order item</th>
                          <th className="pb-1.5 pr-4 font-medium">Qty</th>
                          <th className="pb-1.5 pr-4 font-medium">Refund amount</th>
                          <th className="pb-1.5 font-medium">Item status</th>
                        </tr>
                      </thead>
                      <tbody>
                        {r.items.map((item) => (
                          <tr key={item.orderDetailId} className="border-b border-[var(--color-line)] last:border-0">
                            <td className="py-1.5 pr-4 text-[var(--color-ink)]">#{item.orderDetailId}</td>
                            <td className="py-1.5 pr-4 text-[var(--color-muted)]">{item.quantity}</td>
                            <td className="py-1.5 pr-4 text-[var(--color-ink)]">
                              {formatInrFromPaise(item.refundAmountMinor)}
                            </td>
                            <td className="py-1.5 text-[var(--color-muted)]">{item.status}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>

                  {isOpen && (
                    <div className="mt-3 flex flex-wrap gap-1.5">
                      {r.status === "requested" && (
                        <Button
                          type="button"
                          size="sm"
                          variant="outline"
                          disabled={pending}
                          onClick={() => statusMutation.mutate({ returnId: r.returnId, status: "approved" })}
                        >
                          Approve
                        </Button>
                      )}
                      {(r.status === "requested" || r.status === "approved") && (
                        <Button
                          type="button"
                          size="sm"
                          variant="outline"
                          disabled={pending}
                          onClick={() => statusMutation.mutate({ returnId: r.returnId, status: "in_transit" })}
                        >
                          Mark in transit
                        </Button>
                      )}
                      <Button
                        type="button"
                        size="sm"
                        disabled={pending}
                        onClick={() => receivedMutation.mutate(r.returnId)}
                      >
                        {receivedMutation.isPending ? "Marking received…" : "Mark received"}
                      </Button>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        className="border-red-200 text-red-600 hover:bg-red-50"
                        disabled={pending}
                        onClick={() => statusMutation.mutate({ returnId: r.returnId, status: "rejected" })}
                      >
                        Reject
                      </Button>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        className="border-red-200 text-red-600 hover:bg-red-50"
                        disabled={pending}
                        onClick={() => statusMutation.mutate({ returnId: r.returnId, status: "cancelled" })}
                      >
                        Cancel
                      </Button>
                    </div>
                  )}
                  {rowError[r.returnId] && (
                    <p className="mt-2 text-sm text-red-600" role="alert">
                      {rowError[r.returnId]}
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
