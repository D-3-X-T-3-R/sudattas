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
        <p className="text-[15px] leading-relaxed text-[var(--color-ink)]">
          Archive <strong>{product?.name ?? ""}</strong>? It will be hidden from the store but not deleted — you can bring it back later.
        </p>
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
            {isPending ? "Archiving…" : "Archive"}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}

