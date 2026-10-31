"use client";

import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { History, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { createInventoryLog, deleteInventoryLog, type InventoryLogRow } from "@/lib/admin-queries";
import type { VariantLabelMap } from "@/domains/admin/inventory/types";

function formatLogTime(raw: string): string {
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

interface InventoryLogsCardProps {
  logs: InventoryLogRow[];
  isLoading: boolean;
  isError: boolean;
  variantLabels: VariantLabelMap;
}

export function InventoryLogsCard({ logs, isLoading, isError, variantLabels }: InventoryLogsCardProps) {
  const queryClient = useQueryClient();
  const [variantId, setVariantId] = useState("");
  const [changeQuantity, setChangeQuantity] = useState("");
  const [reason, setReason] = useState("");
  const [formError, setFormError] = useState("");

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["admin", "inventory-logs"] });

  const createMutation = useMutation({
    mutationFn: () => createInventoryLog({ variantId, changeQuantity, reason: reason.trim() }),
    onSuccess: () => {
      invalidate();
      setVariantId("");
      setChangeQuantity("");
      setReason("");
      setFormError("");
    },
    onError: (err: Error) => setFormError(err.message || "Failed to add log entry."),
  });

  const deleteMutation = useMutation({
    mutationFn: (logId: string) => deleteInventoryLog(logId),
    onSuccess: () => invalidate(),
  });

  const variantOptions = Array.from(variantLabels.entries());

  return (
    <AdminTableCard title="Inventory logs" icon={<History className="h-4 w-4 text-[var(--color-green)]" />}>
      <p className="mb-3 text-sm text-[var(--color-muted)]">
        A manual audit trail of stock changes — nothing in the system writes here automatically
        yet (orders, cancellations, and restocks don&apos;t log here), so this stays empty unless
        entries are added below.
      </p>

      {isLoading ? (
        <p className="py-6 text-center text-sm text-[var(--color-muted)]">Loading…</p>
      ) : null}
      {isError ? (
        <p className="py-6 text-center text-sm text-rose-700">Could not load inventory logs.</p>
      ) : null}

      {!isLoading && !isError && logs.length === 0 ? (
        <p className="py-6 text-center text-sm text-[var(--color-muted)]">No log entries yet.</p>
      ) : null}

      {!isLoading && !isError && logs.length > 0 ? (
        <div className="overflow-x-auto">
          <table className="w-full min-w-[560px] border-collapse text-[15px]">
            <caption className="sr-only">Inventory change log</caption>
            <thead>
              <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                <th className="pb-2 pr-4 font-medium">When</th>
                <th className="pb-2 pr-4 font-medium">Variant</th>
                <th className="pb-2 pr-4 font-medium">Change</th>
                <th className="pb-2 pr-4 font-medium">Reason</th>
                <th className="pb-2 font-medium sr-only">Actions</th>
              </tr>
            </thead>
            <tbody>
              {logs.map((log) => {
                const label = variantLabels.get(log.variantId);
                return (
                  <tr key={log.logId} className="border-b border-[var(--color-line)] last:border-0">
                    <td className="py-3 pr-4 text-[var(--color-muted)]">{formatLogTime(log.logTime)}</td>
                    <td className="py-3 pr-4 text-[var(--color-ink)]">
                      {label ? `${label.productName} — ${label.sizeName}` : `Variant #${log.variantId}`}
                    </td>
                    <td className="py-3 pr-4 text-[var(--color-ink)]">{log.changeQuantity}</td>
                    <td className="py-3 pr-4 text-[var(--color-muted)]">{log.reason}</td>
                    <td className="py-3 text-right">
                      <button
                        type="button"
                        onClick={() => deleteMutation.mutate(log.logId)}
                        disabled={deleteMutation.isPending}
                        aria-label="Delete log entry"
                        className="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-red-50 hover:text-red-600"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      ) : null}

      <div className="mt-4 border-t border-[var(--color-line)] pt-4">
        <p className="mb-2 text-sm font-medium text-[var(--color-muted)]">Add a manual entry</p>
        <div className="flex flex-wrap items-end gap-2.5">
          <select
            value={variantId}
            onChange={(e) => setVariantId(e.target.value)}
            className="h-10 min-w-[14rem] rounded-lg border border-[var(--color-line)] bg-white px-3 text-[15px] focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
          >
            <option value="">Select variant…</option>
            {variantOptions.map(([id, label]) => (
              <option key={id} value={id}>
                {label.productName} — {label.sizeName}
              </option>
            ))}
          </select>
          <Input
            value={changeQuantity}
            onChange={(e) => setChangeQuantity(e.target.value)}
            placeholder="Change (e.g. -2 or 10)"
            className="h-10 max-w-[9rem] rounded-lg text-[15px]"
          />
          <Input
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            placeholder="Reason (e.g. Manual recount)"
            className="h-10 max-w-[14rem] rounded-lg text-[15px]"
          />
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={!variantId || !changeQuantity.trim() || !reason.trim() || createMutation.isPending}
            onClick={() => createMutation.mutate()}
          >
            {createMutation.isPending ? "Adding…" : "Add entry"}
          </Button>
        </div>
        {formError && (
          <p className="mt-2 text-sm text-red-600" role="alert">
            {formError}
          </p>
        )}
      </div>
    </AdminTableCard>
  );
}
