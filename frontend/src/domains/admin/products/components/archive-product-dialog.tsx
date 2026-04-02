"use client";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import type { ProductListRow } from "@/lib/admin-queries";

interface ArchiveProductDialogProps {
  product: ProductListRow | null;
  isPending: boolean;
  onClose: () => void;
  onConfirm: () => void;
}

export function ArchiveProductDialog({
  product,
  isPending,
  onClose,
  onConfirm,
}: ArchiveProductDialogProps) {
  return (
    <Dialog open={!!product} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="sm:max-w-md">
        <p className="text-sm text-[var(--color-ink)]">
          Archive product <strong>{product?.name ?? ""}</strong> (ID: {product?.productId ?? ""})? Its status will be set to Archived; it will not be deleted.
        </p>
        <div className="mt-4 flex justify-end gap-2">
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button
            variant="outline"
            className="border-red-200 text-red-600 hover:bg-red-50"
            onClick={onConfirm}
            disabled={isPending}
          >
            {isPending ? "Archiving…" : "Archive"}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}

