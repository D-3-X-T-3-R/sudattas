"use client";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import type { ProductListRow } from "@/lib/admin-queries";

interface PermanentlyDeleteProductDialogProps {
  product: ProductListRow | null;
  isPending: boolean;
  error: string;
  onClose: () => void;
  onConfirm: () => void;
}

/**
 * The irreversible, second tier of product deletion (see Archive for the reversible one).
 * The backend refuses outright — with a clear message surfaced via `error` — if the product
 * has ever been ordered; that's the only real guardrail, so the copy here focuses on making
 * sure the admin means it, not on re-explaining that check.
 */
export function PermanentlyDeleteProductDialog({
  product,
  isPending,
  error,
  onClose,
  onConfirm,
}: PermanentlyDeleteProductDialogProps) {
  return (
    <Dialog open={!!product} onOpenChange={(open) => !open && !isPending && onClose()}>
      <DialogContent className="sm:max-w-md">
        <p className="text-[15px] leading-relaxed text-[var(--color-ink)]">
          Permanently delete <strong>{product?.name ?? ""}</strong>? This cannot be undone —
          it removes the product, its variants, inventory, cart lines, reviews, wishlist
          entries, and images (including from storage) for good. If it has ever been ordered,
          this will be refused; use Archive for that instead.
        </p>
        {error && (
          <p className="mt-2 text-sm text-red-600" role="alert">
            {error}
          </p>
        )}
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="outline" onClick={onClose} disabled={isPending}>
            Cancel
          </Button>
          <Button
            variant="outline"
            className="border-red-200 text-red-600 hover:bg-red-50"
            onClick={onConfirm}
            disabled={isPending}
          >
            {isPending ? "Deleting…" : "Permanently delete"}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
