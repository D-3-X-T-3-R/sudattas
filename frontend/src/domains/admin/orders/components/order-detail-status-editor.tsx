"use client";

import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import {
  updateAdminOrderStatus,
  type AdminOrderDetail,
} from "@/lib/admin-order-detail";
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

  const currentRow = statuses.find((s) => s.statusId === order.statusId);
  const currentName = currentRow?.statusName ?? "";
  const selectableStatuses = filterStatusesForTransition(statuses, currentName);

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

  return (
    <div>
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
    </div>
  );
}
