"use client";

import { Pencil, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { formatInrFromPaise } from "@/lib/money";
import type { TransactionRow as TransactionRowData } from "@/lib/admin-transactions";

function formatTxDate(raw: string): string {
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

interface EditState {
  userId: string;
  /** Rupees, as typed — converted to paise only when the edit is saved. */
  amountRupees: string;
  type: string;
}

interface TransactionRowProps {
  tx: TransactionRowData;
  isEditing: boolean;
  editState: EditState;
  setEditState: (state: EditState) => void;
  isSaving: boolean;
  onBeginEdit: () => void;
  onSave: () => void;
  onCancel: () => void;
  onRequestDelete: () => void;
}

export function TransactionRow({
  tx,
  isEditing,
  editState,
  setEditState,
  isSaving,
  onBeginEdit,
  onSave,
  onCancel,
  onRequestDelete,
}: TransactionRowProps) {
  if (isEditing) {
    return (
      <tr className="border-b border-[var(--color-line)] last:border-0">
        <td className="py-2.5 pr-4 text-[var(--color-muted)]">{formatTxDate(tx.transactionDate)}</td>
        <td className="py-2.5 pr-4">
          <Input
            value={editState.userId}
            onChange={(e) => setEditState({ ...editState, userId: e.target.value })}
            className="h-9 max-w-[7rem] rounded-lg text-[15px]"
          />
        </td>
        <td className="py-2.5 pr-4">
          <Input
            value={editState.amountRupees}
            onChange={(e) => setEditState({ ...editState, amountRupees: e.target.value })}
            placeholder="Amount (₹)"
            className="h-9 max-w-[9rem] rounded-lg text-[15px]"
          />
        </td>
        <td className="py-2.5 pr-4">
          <Input
            value={editState.type}
            onChange={(e) => setEditState({ ...editState, type: e.target.value })}
            placeholder="Type"
            className="h-9 max-w-[9rem] rounded-lg text-[15px]"
          />
        </td>
        <td className="py-2.5">
          <div className="flex gap-1.5">
            <Button type="button" size="sm" disabled={isSaving} onClick={onSave}>
              {isSaving ? "Saving…" : "Save"}
            </Button>
            <Button type="button" size="sm" variant="outline" onClick={onCancel}>
              Cancel
            </Button>
          </div>
        </td>
      </tr>
    );
  }

  return (
    <tr className="border-b border-[var(--color-line)] last:border-0">
      <td className="py-2.5 pr-4 text-[var(--color-muted)]">{formatTxDate(tx.transactionDate)}</td>
      <td className="py-2.5 pr-4 text-[var(--color-ink)]">#{tx.userId}</td>
      <td className="py-2.5 pr-4 text-[var(--color-ink)]">{formatInrFromPaise(tx.amountPaise)}</td>
      <td className="py-2.5 pr-4 text-[var(--color-muted)] capitalize">{tx.type}</td>
      <td className="py-2.5">
        <div className="flex gap-1.5">
          <button
            type="button"
            onClick={onBeginEdit}
            aria-label={`Edit transaction ${tx.transactionId}`}
            className="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-[var(--color-surface-soft)] hover:text-[var(--color-ink)]"
          >
            <Pencil className="h-4 w-4" />
          </button>
          <button
            type="button"
            onClick={onRequestDelete}
            aria-label={`Delete transaction ${tx.transactionId}`}
            className="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-red-50 hover:text-red-600"
          >
            <Trash2 className="h-4 w-4" />
          </button>
        </div>
      </td>
    </tr>
  );
}
