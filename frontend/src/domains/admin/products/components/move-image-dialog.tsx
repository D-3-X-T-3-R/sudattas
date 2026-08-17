"use client";

import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { fetchProductsList } from "@/lib/admin-queries";

interface MoveImageDialogProps {
  /** null closes the dialog. */
  open: boolean;
  isPending: boolean;
  error: string;
  /** Product the image currently belongs to — excluded from the results, and shown for context. */
  currentProductId: string | null;
  onClose: () => void;
  onConfirm: (targetProductId: string) => void;
}

/** Move an existing product image to a different product — see moveProductImageToProduct in
 * admin-product-queries.ts for why this, not alt-text editing, is what update_product_image
 * actually supports. */
export function MoveImageDialog({
  open,
  isPending,
  error,
  currentProductId,
  onClose,
  onConfirm,
}: MoveImageDialogProps) {
  const [search, setSearch] = useState("");
  const [selectedId, setSelectedId] = useState("");

  const searchQuery = useQuery({
    queryKey: ["admin", "move-image-search", search],
    queryFn: () => fetchProductsList({ name: search.trim() || undefined, limit: "20" }),
    enabled: open,
  });

  const results = (searchQuery.data ?? []).filter((p) => p.productId !== currentProductId);

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (!next) {
          setSearch("");
          setSelectedId("");
          onClose();
        }
      }}
    >
      <DialogContent className="sm:max-w-md">
        <p className="text-[15px] font-medium text-[var(--color-ink)]">Move image to another product</p>
        <p className="mt-1 text-sm text-[var(--color-muted)]">
          Search for the product this image should belong to instead.
        </p>

        <Input
          value={search}
          onChange={(e) => {
            setSearch(e.target.value);
            setSelectedId("");
          }}
          placeholder="Search products by name…"
          className="mt-3 rounded-lg text-[15px]"
          autoFocus
        />

        <div className="mt-2 max-h-56 overflow-y-auto rounded-lg border border-[var(--color-line)]">
          {searchQuery.isLoading ? (
            <p className="p-3 text-sm text-[var(--color-muted)]">Searching…</p>
          ) : results.length === 0 ? (
            <p className="p-3 text-sm text-[var(--color-muted)]">
              {search.trim() ? "No matching products." : "Type to search for a product."}
            </p>
          ) : (
            <ul>
              {results.map((p) => (
                <li key={p.productId}>
                  <button
                    type="button"
                    onClick={() => setSelectedId(p.productId)}
                    className={`w-full px-3 py-2 text-left text-[15px] hover:bg-[var(--color-surface-soft)] ${
                      selectedId === p.productId ? "bg-[var(--color-surface-soft)] font-medium" : ""
                    }`}
                  >
                    {p.name}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>

        {error && (
          <p className="mt-2 text-sm text-red-600" role="alert">
            {error}
          </p>
        )}

        <div className="mt-5 flex justify-end gap-2">
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button disabled={!selectedId || isPending} onClick={() => onConfirm(selectedId)}>
            {isPending ? "Moving…" : "Move image"}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
