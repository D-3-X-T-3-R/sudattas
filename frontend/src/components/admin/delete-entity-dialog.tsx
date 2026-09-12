"use client";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent } from "@/components/ui/dialog";

interface DeleteEntityDialogProps {
  /** null closes the dialog. */
  entity: { name: string } | null;
  /** Noun shown in the confirmation copy, e.g. "mood", "size". */
  label: string;
  isPending: boolean;
  error: string;
  onClose: () => void;
  onConfirm: () => void;
  /** Defaults to a generic "cannot be undone" warning. */
  warning?: string;
}

/** Generic delete-confirmation dialog for simple {id, name} reference-data rows (moods, sizes,
 * colors, fabrics, weaves, occasions, ...). Mirrors domains/admin/products/delete-category-dialog.tsx. */
export function DeleteEntityDialog({
  entity,
  label,
  isPending,
  error,
  onClose,
  onConfirm,
  warning,
}: DeleteEntityDialogProps) {
  return (
    <Dialog open={!!entity} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-md">
        <p className="text-[15px] leading-relaxed text-[var(--color-ink)]">
          Delete {label} <strong>{entity?.name ?? ""}</strong>?{" "}
          {warning ?? "This cannot be undone."}
        </p>
        {error && (
          <p className="mt-2 text-sm text-red-600" role="alert">
            {error}
          </p>
        )}
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button
            variant="outline"
            className="border-red-200 text-red-600 hover:bg-red-50"
            onClick={onConfirm}
            disabled={isPending}
          >
            {isPending ? "Deleting…" : "Delete"}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
