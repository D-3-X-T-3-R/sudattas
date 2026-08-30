"use client";

import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Pencil, Trash2, Truck } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { DeleteEntityDialog } from "@/components/admin/delete-entity-dialog";
import { useToast } from "@/components/ui/toast";
import {
  fetchShippingMethods,
  createShippingMethod,
  updateShippingMethod,
  deleteShippingMethod,
  type ShippingMethodRow,
} from "@/lib/admin-shipping-methods";
import { formatInrFromPaise, paiseToRupeesInput, rupeesInputToPaise } from "@/lib/money";

const QUERY_KEY = ["admin", "shipping-methods"];

interface DraftFields {
  methodName: string;
  costRupees: string;
  estimatedDeliveryTime: string;
}

const EMPTY_DRAFT: DraftFields = { methodName: "", costRupees: "", estimatedDeliveryTime: "" };

function validateDraft(draft: DraftFields): string {
  if (!draft.methodName.trim()) return "Method name is required.";
  if (draft.costRupees.trim() === "" || Number.isNaN(Number(draft.costRupees))) {
    return "Cost must be a number.";
  }
  if (Number(draft.costRupees) < 0) return "Cost can't be negative.";
  if (!draft.estimatedDeliveryTime.trim()) return "Estimated delivery time is required.";
  return "";
}

function DraftFieldsInputs({
  draft,
  setDraft,
  autoFocus,
}: {
  draft: DraftFields;
  setDraft: (draft: DraftFields) => void;
  autoFocus?: boolean;
}) {
  return (
    <div className="grid flex-1 grid-cols-1 gap-2 sm:grid-cols-3">
      <Input
        value={draft.methodName}
        onChange={(e) => setDraft({ ...draft, methodName: e.target.value })}
        placeholder="e.g. Standard Delivery"
        className="h-10 rounded-lg text-[15px]"
        autoFocus={autoFocus}
      />
      <Input
        value={draft.costRupees}
        onChange={(e) => setDraft({ ...draft, costRupees: e.target.value })}
        placeholder="Cost in ₹, e.g. 49"
        inputMode="decimal"
        className="h-10 rounded-lg text-[15px]"
      />
      <Input
        value={draft.estimatedDeliveryTime}
        onChange={(e) => setDraft({ ...draft, estimatedDeliveryTime: e.target.value })}
        placeholder="e.g. 3-5 business days"
        className="h-10 rounded-lg text-[15px]"
      />
    </div>
  );
}

function ShippingMethodRowView({
  method,
  isEditing,
  editDraft,
  setEditDraft,
  isSaving,
  onBeginEdit,
  onSave,
  onCancel,
  onRequestDelete,
}: {
  method: ShippingMethodRow;
  isEditing: boolean;
  editDraft: DraftFields;
  setEditDraft: (draft: DraftFields) => void;
  isSaving: boolean;
  onBeginEdit: () => void;
  onSave: () => void;
  onCancel: () => void;
  onRequestDelete: () => void;
}) {
  if (isEditing) {
    return (
      <div className="flex flex-col gap-2 p-2.5 sm:flex-row sm:items-center">
        <DraftFieldsInputs draft={editDraft} setDraft={setEditDraft} autoFocus />
        <div className="flex gap-2">
          <Button
            type="button"
            size="sm"
            disabled={isSaving}
            onClick={onSave}
            className="rounded-lg bg-[var(--color-accent-brown)] hover:bg-[var(--color-accent-brown)]/90"
          >
            {isSaving ? "Saving…" : "Save"}
          </Button>
          <Button type="button" size="sm" variant="outline" onClick={onCancel}>
            Cancel
          </Button>
        </div>
      </div>
    );
  }
  return (
    <div className="flex items-center gap-2 p-2.5">
      <div className="grid min-w-0 flex-1 grid-cols-1 gap-1 sm:grid-cols-3 sm:gap-2">
        <span className="truncate text-[15px] text-[var(--color-ink)]">{method.methodName}</span>
        <span className="text-[15px] text-[var(--color-muted)]">
          {formatInrFromPaise(method.costPaise)}
        </span>
        <span className="truncate text-[15px] text-[var(--color-muted)]">
          {method.estimatedDeliveryTime}
        </span>
      </div>
      <button
        type="button"
        onClick={onBeginEdit}
        aria-label={`Edit ${method.methodName}`}
        className="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-[var(--color-surface-soft)] hover:text-[var(--color-ink)]"
      >
        <Pencil className="h-4 w-4" />
      </button>
      <button
        type="button"
        onClick={onRequestDelete}
        aria-label={`Delete ${method.methodName}`}
        className="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-red-50 hover:text-red-600"
      >
        <Trash2 className="h-4 w-4" />
      </button>
    </div>
  );
}

/** Full CRUD for the `ShippingMethods` reference table (name, cost, estimated delivery time) —
 * used at checkout to offer delivery options. Cost is entered/shown in rupees; converted to
 * paise only at the API boundary (see money.ts), matching the fix already applied to
 * transactions/variants/manual-order line items after the same raw-paise-input bug there. */
export function ShippingMethodsCard() {
  const queryClient = useQueryClient();
  const { showToast } = useToast();
  const methodsQuery = useQuery({ queryKey: QUERY_KEY, queryFn: fetchShippingMethods });
  const methods = methodsQuery.data ?? [];

  const [newDraft, setNewDraft] = useState<DraftFields>(EMPTY_DRAFT);
  const [createError, setCreateError] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editDraft, setEditDraft] = useState<DraftFields>(EMPTY_DRAFT);
  const [manageError, setManageError] = useState("");
  const [deleteConfirm, setDeleteConfirm] = useState<ShippingMethodRow | null>(null);
  const [deleteError, setDeleteError] = useState("");

  const invalidate = () => queryClient.invalidateQueries({ queryKey: QUERY_KEY });

  const createMutation = useMutation({
    mutationFn: (draft: DraftFields) =>
      createShippingMethod({
        methodName: draft.methodName.trim(),
        costPaise: String(rupeesInputToPaise(draft.costRupees.trim())),
        estimatedDeliveryTime: draft.estimatedDeliveryTime.trim(),
      }),
    onSuccess: () => {
      invalidate();
      setNewDraft(EMPTY_DRAFT);
      setCreateError("");
      showToast({ title: "Shipping methods", description: "Method added." });
    },
    onError: (err: Error) => setCreateError(err.message || "Failed to add shipping method."),
  });

  const updateMutation = useMutation({
    mutationFn: ({ methodId, draft }: { methodId: string; draft: DraftFields }) =>
      updateShippingMethod({
        methodId,
        methodName: draft.methodName.trim(),
        costPaise: String(rupeesInputToPaise(draft.costRupees.trim())),
        estimatedDeliveryTime: draft.estimatedDeliveryTime.trim(),
      }),
    onSuccess: () => {
      invalidate();
      setEditingId(null);
      setEditDraft(EMPTY_DRAFT);
      setManageError("");
      showToast({ title: "Shipping methods", description: "Method updated." });
    },
    onError: (err: Error) => setManageError(err.message || "Failed to update shipping method."),
  });

  const deleteMutation = useMutation({
    mutationFn: (methodId: string) => deleteShippingMethod(methodId),
    onSuccess: () => {
      invalidate();
      setDeleteConfirm(null);
      setDeleteError("");
      showToast({ title: "Shipping methods", description: "Deleted." });
    },
    onError: (err: Error) => setDeleteError(err.message || "Failed to delete shipping method."),
  });

  const handleCreate = () => {
    const error = validateDraft(newDraft);
    if (error) {
      setCreateError(error);
      return;
    }
    createMutation.mutate(newDraft);
  };

  const handleSaveEdit = () => {
    const error = validateDraft(editDraft);
    if (error) {
      setManageError(error);
      return;
    }
    if (editingId) updateMutation.mutate({ methodId: editingId, draft: editDraft });
  };

  return (
    <AdminTableCard title="Shipping methods" icon={<Truck className="h-4 w-4 text-[var(--color-green)]" />}>
      {methodsQuery.isLoading ? (
        <p className="py-4 text-center text-sm text-[var(--color-muted)]">Loading…</p>
      ) : null}
      {methodsQuery.isError ? (
        <p className="py-4 text-center text-sm text-rose-700">Could not load shipping methods.</p>
      ) : null}

      {!methodsQuery.isLoading && !methodsQuery.isError ? (
        <>
          {methods.length === 0 ? (
            <p className="py-4 text-sm text-[var(--color-muted)]">None yet. Add one below.</p>
          ) : (
            <div className="divide-y divide-[var(--color-line)] rounded-lg border border-[var(--color-line)]">
              <div className="hidden gap-2 px-2.5 pt-2 text-xs font-medium text-[var(--color-muted)] sm:grid sm:grid-cols-3">
                <span>Method</span>
                <span>Cost</span>
                <span>Estimated delivery</span>
              </div>
              {methods.map((method) => (
                <ShippingMethodRowView
                  key={method.methodId}
                  method={method}
                  isEditing={editingId === method.methodId}
                  editDraft={editDraft}
                  setEditDraft={(draft) => {
                    setEditDraft(draft);
                    setManageError("");
                  }}
                  isSaving={updateMutation.isPending}
                  onBeginEdit={() => {
                    setEditingId(method.methodId);
                    setEditDraft({
                      methodName: method.methodName,
                      costRupees: paiseToRupeesInput(method.costPaise),
                      estimatedDeliveryTime: method.estimatedDeliveryTime,
                    });
                    setManageError("");
                  }}
                  onSave={handleSaveEdit}
                  onCancel={() => {
                    setEditingId(null);
                    setEditDraft(EMPTY_DRAFT);
                    setManageError("");
                  }}
                  onRequestDelete={() => {
                    setDeleteError("");
                    setDeleteConfirm(method);
                  }}
                />
              ))}
            </div>
          )}
          {manageError && (
            <p className="mt-2 text-sm text-red-600" role="alert">
              {manageError}
            </p>
          )}

          <div className="mt-3 flex flex-col gap-2 sm:flex-row sm:items-center">
            <DraftFieldsInputs
              draft={newDraft}
              setDraft={(draft) => {
                setNewDraft(draft);
                setCreateError("");
              }}
            />
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={createMutation.isPending}
              onClick={handleCreate}
            >
              {createMutation.isPending ? "Adding…" : "Add"}
            </Button>
          </div>
          {createError && (
            <p className="mt-2 text-sm text-red-600" role="alert">
              {createError}
            </p>
          )}
        </>
      ) : null}

      <DeleteEntityDialog
        entity={deleteConfirm ? { name: deleteConfirm.methodName } : null}
        label="shipping method"
        isPending={deleteMutation.isPending}
        error={deleteError}
        warning="Checkout will no longer offer this option. Existing orders are unaffected."
        onClose={() => {
          setDeleteConfirm(null);
          setDeleteError("");
        }}
        onConfirm={() => {
          if (deleteConfirm) deleteMutation.mutate(deleteConfirm.methodId);
        }}
      />
    </AdminTableCard>
  );
}
