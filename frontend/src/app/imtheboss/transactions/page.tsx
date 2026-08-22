"use client";

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Receipt } from "lucide-react";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { DeleteEntityDialog } from "@/components/admin/delete-entity-dialog";
import { TransactionRow } from "@/domains/admin/transactions/components/transaction-row";
import {
  createTransactionAdmin,
  deleteTransactionAdmin,
  fetchTransactions,
  updateTransactionAdmin,
  type TransactionRow as TransactionRowData,
} from "@/lib/admin-transactions";

export default function AdminTransactionsPage() {
  const queryClient = useQueryClient();
  const txQuery = useQuery({ queryKey: ["admin", "transactions"], queryFn: fetchTransactions });

  const [newUserId, setNewUserId] = useState("");
  const [newAmount, setNewAmount] = useState("");
  const [newType, setNewType] = useState("");
  const [createError, setCreateError] = useState("");

  const [editingId, setEditingId] = useState<string | null>(null);
  const [editState, setEditState] = useState({ userId: "", amountPaise: "", type: "" });
  const [rowError, setRowError] = useState("");

  const [deleteConfirm, setDeleteConfirm] = useState<TransactionRowData | null>(null);
  const [deleteError, setDeleteError] = useState("");

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["admin", "transactions"] });

  const createMutation = useMutation({
    mutationFn: () =>
      createTransactionAdmin({ userId: newUserId.trim(), amountPaise: newAmount.trim(), type: newType.trim() }),
    onSuccess: () => {
      invalidate();
      setNewUserId("");
      setNewAmount("");
      setNewType("");
      setCreateError("");
    },
    onError: (err: Error) => setCreateError(err.message || "Failed to record transaction."),
  });

  const updateMutation = useMutation({
    mutationFn: (transactionId: string) =>
      updateTransactionAdmin({
        transactionId,
        userId: editState.userId.trim(),
        amountPaise: editState.amountPaise.trim(),
        type: editState.type.trim(),
      }),
    onSuccess: () => {
      invalidate();
      setEditingId(null);
      setRowError("");
    },
    onError: (err: Error) => setRowError(err.message || "Failed to update transaction."),
  });

  const deleteMutation = useMutation({
    mutationFn: (transactionId: string) => deleteTransactionAdmin(transactionId),
    onSuccess: () => {
      invalidate();
      setDeleteConfirm(null);
      setDeleteError("");
    },
    onError: (err: Error) => setDeleteError(err.message || "Failed to delete transaction."),
  });

  const transactions = txQuery.data ?? [];

  return (
    <AdminPageShell
      label="Transactions"
      title="Transaction ledger"
      description="A manually maintained payment ledger — nothing in checkout, refunds, or shipping writes here automatically yet, so entries only appear once added below."
    >
      <AdminTableCard title="Transactions" icon={<Receipt className="h-4 w-4 text-[var(--color-green)]" />}>
        {txQuery.isLoading ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading transactions…</p>
        ) : null}
        {txQuery.isError ? (
          <p className="py-8 text-center text-sm text-rose-700">Could not load transactions.</p>
        ) : null}
        {!txQuery.isLoading && !txQuery.isError && transactions.length === 0 ? (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">No transactions recorded yet.</p>
        ) : null}

        {!txQuery.isLoading && !txQuery.isError && transactions.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full min-w-[640px] border-collapse text-[15px]">
              <caption className="sr-only">Transaction ledger</caption>
              <thead>
                <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                  <th className="pb-2 pr-4 font-medium">Date</th>
                  <th className="pb-2 pr-4 font-medium">User</th>
                  <th className="pb-2 pr-4 font-medium">Amount</th>
                  <th className="pb-2 pr-4 font-medium">Type</th>
                  <th className="pb-2 font-medium sr-only">Actions</th>
                </tr>
              </thead>
              <tbody>
                {transactions.map((tx) => (
                  <TransactionRow
                    key={tx.transactionId}
                    tx={tx}
                    isEditing={editingId === tx.transactionId}
                    editState={editState}
                    setEditState={setEditState}
                    isSaving={updateMutation.isPending}
                    onBeginEdit={() => {
                      setEditingId(tx.transactionId);
                      setEditState({ userId: tx.userId, amountPaise: tx.amountPaise, type: tx.type });
                      setRowError("");
                    }}
                    onSave={() => updateMutation.mutate(tx.transactionId)}
                    onCancel={() => {
                      setEditingId(null);
                      setRowError("");
                    }}
                    onRequestDelete={() => {
                      setDeleteError("");
                      setDeleteConfirm(tx);
                    }}
                  />
                ))}
              </tbody>
            </table>
          </div>
        ) : null}
        {rowError && (
          <p className="mt-2 text-sm text-red-600" role="alert">
            {rowError}
          </p>
        )}

        <div className="mt-4 border-t border-[var(--color-line)] pt-4">
          <p className="mb-2 text-sm font-medium text-[var(--color-muted)]">Record a transaction</p>
          <div className="flex flex-wrap items-end gap-2.5">
            <Input
              value={newUserId}
              onChange={(e) => setNewUserId(e.target.value)}
              placeholder="User ID"
              className="h-10 max-w-[8rem] rounded-lg text-[15px]"
            />
            <Input
              value={newAmount}
              onChange={(e) => setNewAmount(e.target.value)}
              placeholder="Amount (paise)"
              className="h-10 max-w-[10rem] rounded-lg text-[15px]"
            />
            <Input
              value={newType}
              onChange={(e) => setNewType(e.target.value)}
              placeholder="Type (e.g. payment, refund)"
              className="h-10 max-w-[13rem] rounded-lg text-[15px]"
            />
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={!newUserId.trim() || !newAmount.trim() || !newType.trim() || createMutation.isPending}
              onClick={() => createMutation.mutate()}
            >
              {createMutation.isPending ? "Recording…" : "Record"}
            </Button>
          </div>
          {createError && (
            <p className="mt-2 text-sm text-red-600" role="alert">
              {createError}
            </p>
          )}
        </div>
      </AdminTableCard>

      <DeleteEntityDialog
        entity={deleteConfirm ? { name: `transaction #${deleteConfirm.transactionId}` } : null}
        label="transaction"
        isPending={deleteMutation.isPending}
        error={deleteError}
        onClose={() => {
          setDeleteConfirm(null);
          setDeleteError("");
        }}
        onConfirm={() => {
          if (deleteConfirm) deleteMutation.mutate(deleteConfirm.transactionId);
        }}
      />
    </AdminPageShell>
  );
}
