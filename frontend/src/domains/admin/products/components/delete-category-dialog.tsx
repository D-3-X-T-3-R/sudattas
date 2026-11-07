"use client";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import type { CategoryRow } from "@/lib/admin-queries";

interface DeleteCategoryDialogProps {
  category: CategoryRow | null;
  isPending: boolean;
  error: string;
  onClose: () => void;
  onConfirm: () => void;
}

export function DeleteCategoryDialog({
  category,
  isPending,
  error,
  onClose,
  onConfirm,
}: DeleteCategoryDialogProps) {
  return (
    <Dialog open={!!category} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-md">
        <p className="text-[15px] leading-relaxed text-[var(--color-ink)]">
          Delete category <strong>{category?.name ?? ""}</strong>? This cannot be undone. Products
          still assigned to it must be moved to another category first.
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
