"use client";

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useSession } from "next-auth/react";
import { Banknote, AlertTriangle } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { StatusBadge } from "@/components/admin/status-badge";
import {
  createRefundAdmin,
  fetchRefundAttemptsAdmin,
  fetchRefundsAdmin,
  resolveRefundAttemptNeedsReview,
} from "@/lib/admin-refunds";
import { formatInrFromPaise, rupeesInputToPaise } from "@/lib/money";

function formatDate(raw: string): string {
  if (!raw) return "—";
  const d = new Date(raw);
  if (Number.isNaN(d.getTime())) return raw;
  return d.toLocaleDateString("en-IN", { day: "2-digit", month: "short", year: "numeric" });
}

export default function AdminRefundsPage() {
  const queryClient = useQueryClient();
  const { data: session } = useSession();
  const actorId = session?.user?.email ?? "admin";

  const [attemptRowError, setAttemptRowError] = useState<Record<string, string>>({});
  const [newOrderId, setNewOrderId] = useState("");
  const [newGatewayRefundId, setNewGatewayRefundId] = useState("");
  const [newAmountRupees, setNewAmountRupees] = useState("");
  const [createError, setCreateError] = useState("");

  const attemptsQuery = useQuery({
    queryKey: ["admin", "refund-attempts"],
    queryFn: () => fetchRefundAttemptsAdmin(),
  });
  const refundsQuery = useQuery({
    queryKey: ["admin", "refunds"],
    queryFn: fetchRefundsAdmin,
  });

  const invalidateAttempts = () =>
    queryClient.invalidateQueries({ queryKey: ["admin", "refund-attempts"] });
  const invalidateRefunds = () => queryClient.invalidateQueries({ queryKey: ["admin", "refunds"] });

  const setAttemptError = (attemptId: string, message: string) =>
    setAttemptRowError((prev) => ({ ...prev, [attemptId]: message }));

  const resolveMutation = useMutation({
    mutationFn: (params: { attemptId: string; resolution: "retry" | "mark_settled" }) =>
      resolveRefundAttemptNeedsReview({ ...params, actorId }),
    onSuccess: (_, { attemptId }) => {
      invalidateAttempts();
      setAttemptError(attemptId, "");
    },
    onError: (err: Error, { attemptId }) =>
      setAttemptError(attemptId, err.message || "Failed to resolve refund attempt."),
  });

  const createRefundMutation = useMutation({
    mutationFn: () =>
      createRefundAdmin({
        orderId: newOrderId.trim(),
        gatewayRefundId: newGatewayRefundId.trim(),
        amountPaise: String(rupeesInputToPaise(newAmountRupees.trim())),
      }),
    onSuccess: () => {
      invalidateRefunds();
      setNewOrderId("");
      setNewGatewayRefundId("");
      setNewAmountRupees("");
      setCreateError("");
    },
    onError: (err: Error) => setCreateError(err.message || "Failed to record refund."),
  });

  const attempts = attemptsQuery.data ?? [];
  const refunds = refundsQuery.data ?? [];
  const needsReviewCount = attempts.filter((a) => a.status === "needs_review").length;

  return (
    <AdminPageShell
      label="Refunds"
      title="Refunds"
      description="Refund attempts against real gateway payments (automatic), and a manually maintained ledger of refunds settled outside the system."
    >
      <AdminTableCard
        title="Refund attempts"
        icon={<AlertTriangle className="h-4 w-4 text-[var(--color-green)]" />}
      >
        <p className="mb-3 text-sm text-[var(--color-muted)]">
          {needsReviewCount > 0
            ? `${needsReviewCount} attempt${needsReviewCount === 1 ? "" : "s"} need${needsReviewCount === 1 ? "s" : ""} review.`
            : "Automatic refund attempts against Razorpay payments — most resolve on their own; only ones stuck in needs_review require action."}
        </p>
        {attemptsQuery.isLoading ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading refund attempts…</p>
        ) : null}
        {attemptsQuery.isError ? (
          <p className="py-8 text-center text-sm text-rose-700">Could not load refund attempts.</p>
        ) : null}
        {!attemptsQuery.isLoading && !attemptsQuery.isError && attempts.length === 0 ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">No refund attempts recorded yet.</p>
        ) : null}

        {!attemptsQuery.isLoading && !attemptsQuery.isError && attempts.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full min-w-[820px] border-collapse text-[15px]">
              <caption className="sr-only">Refund attempts against gateway payments</caption>
              <thead>
                <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                  <th className="pb-2 pr-4 font-medium">Attempt</th>
                  <th className="pb-2 pr-4 font-medium">Order</th>
                  <th className="pb-2 pr-4 font-medium">Amount sent</th>
                  <th className="pb-2 pr-4 font-medium">Status</th>
                  <th className="pb-2 pr-4 font-medium">Error</th>
                  <th className="pb-2 pr-4 font-medium">Updated</th>
                  <th className="pb-2 font-medium">Actions</th>
                </tr>
              </thead>
              <tbody>
                {attempts.map((a) => (
                  <tr key={a.attemptId} className="border-b border-[var(--color-line)] last:border-0 align-top">
                    <td className="py-3 pr-4 text-[var(--color-ink)]">#{a.attemptId}</td>
                    <td className="py-3 pr-4 text-[var(--color-muted)]">#{a.orderId}</td>
                    <td className="py-3 pr-4 text-[var(--color-ink)]">
                      {formatInrFromPaise(a.amountSentToGatewayPaise)}
                    </td>
                    <td className="py-3 pr-4">
                      <StatusBadge label={a.status} />
                    </td>
                    <td className="max-w-[14rem] py-3 pr-4 text-xs text-rose-700">
                      {a.providerError ?? "—"}
                    </td>
                    <td className="py-3 pr-4 text-[var(--color-muted)]">{formatDate(a.updatedAt)}</td>
                    <td className="py-3">
                      {a.status === "needs_review" ? (
                        <div className="flex flex-col gap-1.5">
                          <div className="flex flex-wrap gap-1.5">
                            <Button
                              type="button"
                              size="sm"
                              variant="outline"
                              disabled={resolveMutation.isPending}
                              onClick={() =>
                                resolveMutation.mutate({ attemptId: a.attemptId, resolution: "retry" })
                              }
                            >
                              Retry
                            </Button>
                            <Button
                              type="button"
                              size="sm"
                              variant="outline"
                              disabled={resolveMutation.isPending}
                              onClick={() =>
                                resolveMutation.mutate({ attemptId: a.attemptId, resolution: "mark_settled" })
                              }
                            >
                              Mark settled
                            </Button>
                          </div>
                          {attemptRowError[a.attemptId] && (
                            <p className="text-xs text-red-600" role="alert">
                              {attemptRowError[a.attemptId]}
                            </p>
                          )}
                        </div>
                      ) : (
                        <span className="text-sm text-[var(--color-muted)]">—</span>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : null}
      </AdminTableCard>

      <AdminTableCard title="Refunds" icon={<Banknote className="h-4 w-4 text-[var(--color-green)]" />}>
        <p className="mb-3 text-sm text-[var(--color-muted)]">
          A manually maintained record of settled refunds — nothing writes here automatically for
          refunds processed outside this system; use it to log those for accounting.
        </p>
        {refundsQuery.isLoading ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading refunds…</p>
        ) : null}
        {refundsQuery.isError ? (
          <p className="py-8 text-center text-sm text-rose-700">Could not load refunds.</p>
        ) : null}
        {!refundsQuery.isLoading && !refundsQuery.isError && refunds.length === 0 ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">No refunds recorded yet.</p>
        ) : null}

        {!refundsQuery.isLoading && !refundsQuery.isError && refunds.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full min-w-[640px] border-collapse text-[15px]">
              <caption className="sr-only">Recorded refunds</caption>
              <thead>
                <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                  <th className="pb-2 pr-4 font-medium">Order</th>
                  <th className="pb-2 pr-4 font-medium">Gateway refund ID</th>
                  <th className="pb-2 pr-4 font-medium">Amount</th>
                  <th className="pb-2 pr-4 font-medium">Status</th>
                  <th className="pb-2 font-medium">Date</th>
                </tr>
              </thead>
              <tbody>
                {refunds.map((r) => (
                  <tr key={r.refundId} className="border-b border-[var(--color-line)] last:border-0">
                    <td className="py-3 pr-4 text-[var(--color-ink)]">#{r.orderId}</td>
                    <td className="py-3 pr-4 text-[var(--color-muted)]">{r.gatewayRefundId}</td>
                    <td className="py-3 pr-4 text-[var(--color-ink)]">{formatInrFromPaise(r.amountPaise)}</td>
                    <td className="py-3 pr-4">
                      <StatusBadge label={r.status} />
                    </td>
                    <td className="py-3 text-[var(--color-muted)]">{formatDate(r.createdAt)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : null}

        <div className="mt-4 border-t border-[var(--color-line)] pt-4">
          <p className="mb-2 text-sm font-medium text-[var(--color-muted)]">Record a refund</p>
          <div className="flex flex-wrap items-end gap-2.5">
            <Input
              value={newOrderId}
              onChange={(e) => setNewOrderId(e.target.value)}
              placeholder="Order ID"
              className="h-10 max-w-[8rem] rounded-lg text-[15px]"
            />
            <Input
              value={newGatewayRefundId}
              onChange={(e) => setNewGatewayRefundId(e.target.value)}
              placeholder="Gateway refund ID"
              className="h-10 max-w-[14rem] rounded-lg text-[15px]"
            />
            <Input
              value={newAmountRupees}
              onChange={(e) => setNewAmountRupees(e.target.value)}
              placeholder="Amount (₹)"
              className="h-10 max-w-[10rem] rounded-lg text-[15px]"
            />
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={
                !newOrderId.trim() ||
                !newGatewayRefundId.trim() ||
                !newAmountRupees.trim() ||
                createRefundMutation.isPending
              }
              onClick={() => createRefundMutation.mutate()}
            >
              {createRefundMutation.isPending ? "Recording…" : "Record"}
            </Button>
          </div>
          {createError && (
            <p className="mt-2 text-sm text-red-600" role="alert">
              {createError}
            </p>
          )}
        </div>
      </AdminTableCard>
    </AdminPageShell>
  );
}
