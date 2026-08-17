"use client";

import { useState, type ReactNode } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Pencil, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { DeleteEntityDialog } from "@/components/admin/delete-entity-dialog";
import { useToast } from "@/components/ui/toast";

export interface TaxonomyItem {
  id: string;
  name: string;
}

interface TaxonomyRowProps {
  item: TaxonomyItem;
  isEditing: boolean;
  editingName: string;
  setEditingName: (name: string) => void;
  isSaving: boolean;
  onBeginEdit: () => void;
  onSave: () => void;
  onCancel: () => void;
  onRequestDelete: () => void;
}

function TaxonomyRow({
  item,
  isEditing,
  editingName,
  setEditingName,
  isSaving,
  onBeginEdit,
  onSave,
  onCancel,
  onRequestDelete,
}: TaxonomyRowProps) {
  if (isEditing) {
    return (
      <div className="flex items-center gap-2 p-2.5">
        <Input
          value={editingName}
          onChange={(e) => setEditingName(e.target.value)}
          className="min-w-0 flex-1 rounded-lg text-[15px]"
          autoFocus
        />
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
    );
  }
  return (
    <div className="flex items-center gap-2 p-2.5">
      <span className="min-w-0 flex-1 truncate text-[15px] text-[var(--color-ink)]">{item.name}</span>
      <button
        type="button"
        onClick={onBeginEdit}
        aria-label={`Rename ${item.name}`}
        className="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-[var(--color-surface-soft)] hover:text-[var(--color-ink)]"
      >
        <Pencil className="h-4 w-4" />
      </button>
      <button
        type="button"
        onClick={onRequestDelete}
        aria-label={`Delete ${item.name}`}
        className="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-red-50 hover:text-red-600"
      >
        <Trash2 className="h-4 w-4" />
      </button>
    </div>
  );
}

interface TaxonomyManagerCardProps {
  title: string;
  icon: ReactNode;
  /** react-query key for the list this card reads — invalidated after create/rename/delete. */
  queryKey: unknown[];
  items: TaxonomyItem[];
  isLoading: boolean;
  isError: boolean;
  /** Singular noun for messages, e.g. "size", "fabric". */
  noun: string;
  namePlaceholder: string;
  onCreate: (name: string) => Promise<unknown>;
  onUpdate: (id: string, name: string) => Promise<unknown>;
  onDelete: (id: string) => Promise<void>;
}

/**
 * Generic create/rename/delete manager for the flat {id, name} reference-data tables (sizes,
 * colors, fabrics, weaves, occasions — same shape as moods, which gets its own inline UI in the
 * product editor since it also needs product-linking checkboxes there).
 */
export function TaxonomyManagerCard({
  title,
  icon,
  queryKey,
  items,
  isLoading,
  isError,
  noun,
  namePlaceholder,
  onCreate,
  onUpdate,
  onDelete,
}: TaxonomyManagerCardProps) {
  const queryClient = useQueryClient();
  const { showToast } = useToast();
  const [newName, setNewName] = useState("");
  const [createError, setCreateError] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState("");
  const [manageError, setManageError] = useState("");
  const [deleteConfirm, setDeleteConfirm] = useState<TaxonomyItem | null>(null);
  const [deleteError, setDeleteError] = useState("");

  const invalidate = () => queryClient.invalidateQueries({ queryKey });

  const createMutation = useMutation({
    mutationFn: onCreate,
    onSuccess: () => {
      invalidate();
      setNewName("");
      setCreateError("");
    },
    onError: (err: Error) => setCreateError(err.message || `Failed to create ${noun}.`),
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) => onUpdate(id, name),
    onSuccess: () => {
      invalidate();
      setEditingId(null);
      setEditingName("");
      setManageError("");
      showToast({ title, description: `${title.replace(/s$/, "")} renamed.` });
    },
    onError: (err: Error) => setManageError(err.message || `Failed to rename ${noun}.`),
  });

  const deleteMutation = useMutation({
    mutationFn: onDelete,
    onSuccess: () => {
      invalidate();
      setDeleteConfirm(null);
      setDeleteError("");
      showToast({ title, description: "Deleted." });
    },
    onError: (err: Error) => setDeleteError(err.message || `Failed to delete ${noun}.`),
  });

  const saveEdit = () => {
    const name = editingName.trim();
    if (!name) {
      setManageError(`${noun[0].toUpperCase()}${noun.slice(1)} name is required.`);
      return;
    }
    if (editingId) updateMutation.mutate({ id: editingId, name });
  };

  return (
    <AdminTableCard title={title} icon={icon}>
      {isLoading ? (
        <p className="py-4 text-center text-sm text-[var(--color-muted)]">Loading…</p>
      ) : null}
      {isError ? (
        <p className="py-4 text-center text-sm text-rose-700">Could not load {title.toLowerCase()}.</p>
      ) : null}

      {!isLoading && !isError && (
        <>
          {items.length === 0 ? (
            <p className="py-4 text-sm text-[var(--color-muted)]">None yet. Add one below.</p>
          ) : (
            <div className="divide-y divide-[var(--color-line)] rounded-lg border border-[var(--color-line)]">
              {items.map((item) => (
                <TaxonomyRow
                  key={item.id}
                  item={item}
                  isEditing={editingId === item.id}
                  editingName={editingName}
                  setEditingName={(name) => {
                    setEditingName(name);
                    setManageError("");
                  }}
                  isSaving={updateMutation.isPending}
                  onBeginEdit={() => {
                    setEditingId(item.id);
                    setEditingName(item.name);
                    setManageError("");
                  }}
                  onSave={saveEdit}
                  onCancel={() => {
                    setEditingId(null);
                    setEditingName("");
                    setManageError("");
                  }}
                  onRequestDelete={() => {
                    setDeleteError("");
                    setDeleteConfirm(item);
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

          <div className="mt-3 flex flex-wrap items-center gap-2">
            <Input
              value={newName}
              onChange={(e) => {
                setNewName(e.target.value);
                setCreateError("");
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  const name = newName.trim();
                  if (name) createMutation.mutate(name);
                }
              }}
              placeholder={namePlaceholder}
              className="h-10 max-w-[12rem] rounded-lg text-[15px]"
            />
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={!newName.trim() || createMutation.isPending}
              onClick={() => {
                const name = newName.trim();
                if (name) createMutation.mutate(name);
              }}
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
      )}

      <DeleteEntityDialog
        entity={deleteConfirm}
        label={noun}
        isPending={deleteMutation.isPending}
        error={deleteError}
        warning="Products currently using it are not deleted, but may need updating."
        onClose={() => {
          setDeleteConfirm(null);
          setDeleteError("");
        }}
        onConfirm={() => {
          if (deleteConfirm) deleteMutation.mutate(deleteConfirm.id);
        }}
      />
    </AdminTableCard>
  );
}
