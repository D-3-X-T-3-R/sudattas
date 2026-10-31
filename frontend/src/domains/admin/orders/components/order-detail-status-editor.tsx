"use client";

import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import {
  updateAdminOrderStatus,
  resolveOrderNeedsReview,
  cancelOrderAdmin,
  type AdminOrderDetail,
} from "@/lib/admin-order-detail";
import { DeleteEntityDialog } from "@/components/admin/delete-entity-dialog";
import { cn } from "@/lib/utils";
import { filterStatusesForTransition } from "@/domains/admin/orders/order-status-transitions";
import { formatOrderStatusName, getStatusLabel } from "@/domains/admin/orders/utils";

function formatStatusMutationError(err: unknown): string {
  if (!(err instanceof Error)) return "Could not update status.";
  const msg = err.message;
  const inner = msg.match(/message:\s*"([^"]+)"/);
  if (inner?.[1]) return inner[1];
  if (msg.includes("Illegal order state transition")) {
    const short = msg.match(/Illegal order state transition[^,}\]]*/);
    if (short) return short[0].trim();
  }
  return msg.length > 280 ? `${msg.slice(0, 280)}...` : msg;
}

type OrderDetailStatusEditorProps = {
  order: AdminOrderDetail;
  statuses: { statusId: string; statusName: string }[];
  orderIdParam: string;
};

export function OrderDetailStatusEditor({
  order,
  statuses,
  orderIdParam,
}: OrderDetailStatusEditorProps) {
  const queryClient = useQueryClient();
  const [statusDraft, setStatusDraft] = useState(order.statusId);
  const [cancelConfirmOpen, setCancelConfirmOpen] = useState(false);

  const currentRow = statuses.find((s) => s.statusId === order.statusId);
  const currentName = currentRow?.statusName ?? "";
  const selectableStatuses = filterStatusesForTransition(statuses, currentName);

  const invalidateOrder = () => {
    queryClient.invalidateQueries({ queryKey: ["admin", "order", orderIdParam] });
    queryClient.invalidateQueries({ queryKey: ["admin", "orders"] });
  };

  const cancelMutation = useMutation({
    mutationFn: () => cancelOrderAdmin(order.orderId),
    onSuccess: () => {
      invalidateOrder();
      setCancelConfirmOpen(false);
    },
  });

  const statusMutation = useMutation({
    mutationFn: async (newStatusId: string) => {
      const target = statuses.find((s) => s.statusId === newStatusId);
      const normalizedName = target?.statusName?.trim().toLowerCase() ?? "";
      const shouldBookShiprocket =
        normalizedName === "shipped" ||
        normalizedName === "in transit" ||
        normalizedName.includes("shipped");
      await updateAdminOrderStatus(order, newStatusId, {
        shiprocketBook: shouldBookShiprocket,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin", "order", orderIdParam] });
      queryClient.invalidateQueries({ queryKey: ["admin", "orders"] });
    },
  });

  const resolveMutation = useMutation({
    mutationFn: (resolution: "paid" | "cancelled" | "refunded") =>
      resolveOrderNeedsReview(order.orderId, resolution),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin", "order", orderIdParam] });
      queryClient.invalidateQueries({ queryKey: ["admin", "orders"] });
    },
  });

  const needsReview = currentName === "needs_review";
  const canCancel = !needsReview && currentName !== "cancelled" && currentName !== "refunded";

  return (
    <div>
      {needsReview ? (
        <div className="mb-4 rounded-lg border border-amber-200 bg-amber-50 p-3">
          <p className="text-sm font-medium text-amber-900">
            This order needs manual review (e.g. an ambiguous payment result).
          </p>
          <div className="mt-2 flex flex-wrap gap-2">
            <Button
              type="button"
              size="sm"
              disabled={resolveMutation.isPending}
              onClick={() => resolveMutation.mutate("paid")}
            >
              Mark paid
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={resolveMutation.isPending}
              onClick={() => resolveMutation.mutate("cancelled")}
            >
              Cancel order
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={resolveMutation.isPending}
              onClick={() => resolveMutation.mutate("refunded")}
            >
              Mark refunded
            </Button>
          </div>
          {resolveMutation.isError ? (
            <p className="mt-2 text-sm text-rose-700" role="alert">
              {formatStatusMutationError(resolveMutation.error)}
            </p>
          ) : null}
        </div>
      ) : null}

      <div className="flex flex-wrap items-end gap-3">
        <select
          className={cn(
            "h-11 min-w-[13rem] rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px]",
            "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
          )}
          value={statusDraft}
          onChange={(e) => setStatusDraft(e.target.value)}
          disabled={statusMutation.isPending}
        >
          {selectableStatuses.length === 0 ? (
            <option value={order.statusId}>{getStatusLabel(order.statusId, statuses)} (id {order.statusId})</option>
          ) : (
            <>
              {selectableStatuses.map((s) => (
                <option key={s.statusId} value={s.statusId}>
                  {formatOrderStatusName(s.statusName)}
                </option>
              ))}
              {!selectableStatuses.some((s) => s.statusId === order.statusId) ? (
                <option value={order.statusId}>Current (id {order.statusId})</option>
              ) : null}
            </>
          )}
        </select>

        <Button
          type="button"
          disabled={statusMutation.isPending || !statusDraft || statusDraft === order.statusId}
          onClick={() => statusMutation.mutate(statusDraft)}
        >
          {statusMutation.isPending ? "Saving..." : "Save status"}
        </Button>
      </div>

      <p className="mt-3 text-sm text-[var(--color-muted)]">
        Usual order of things: pending &rarr; confirmed &rarr; processing &rarr; shipped &rarr; delivered.
      </p>

      {statusMutation.isError ? (
        <p className="mt-2 text-sm text-rose-700" role="alert">
          {formatStatusMutationError(statusMutation.error)}
        </p>
      ) : null}

      {canCancel ? (
        <div className="mt-5 border-t border-[var(--color-line)] pt-4">
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="border-red-200 text-red-600 hover:bg-red-50"
            disabled={cancelMutation.isPending}
            onClick={() => setCancelConfirmOpen(true)}
          >
            Cancel order
          </Button>
          <p className="mt-2 text-xs leading-relaxed text-[var(--color-muted)]">
            Only works within the cancellation window and before a shipment is booked — restores
            inventory and settles any refund owed. Once that window closes, use returns / refuse
            delivery instead.
          </p>
        </div>
      ) : null}

      <DeleteEntityDialog
        entity={cancelConfirmOpen ? { name: `order #${order.orderId}` } : null}
        label="order"
        isPending={cancelMutation.isPending}
        error={cancelMutation.isError ? formatStatusMutationError(cancelMutation.error) : ""}
        warning="Restores inventory for the cancelled lines and settles any refund owed — either queued with the payment gateway, or, if this order has no online payment on file (e.g. a manually created order), recorded as already refunded by you outside the system. If that's the case, make sure you've actually returned the money before confirming. Cannot be undone from here."
        onClose={() => {
          setCancelConfirmOpen(false);
          cancelMutation.reset();
        }}
        onConfirm={() => cancelMutation.mutate()}
      />
    </div>
  );
}
