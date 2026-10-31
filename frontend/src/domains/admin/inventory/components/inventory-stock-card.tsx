"use client";

import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Boxes, Trash2 } from "lucide-react";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { DeleteEntityDialog } from "@/components/admin/delete-entity-dialog";
import { deleteInventoryItem } from "@/lib/admin-queries";
import type { InventoryDisplayRow } from "@/domains/admin/inventory/types";

interface InventoryStockCardProps {
  rows: InventoryDisplayRow[];
  isLoading: boolean;
  isError: boolean;
  onlyLowStock: boolean;
  setOnlyLowStock: (value: boolean) => void;
}

export function InventoryStockCard({
  rows,
  isLoading,
  isError,
  onlyLowStock,
  setOnlyLowStock,
}: InventoryStockCardProps) {
  const queryClient = useQueryClient();
  const [deleteConfirm, setDeleteConfirm] = useState<InventoryDisplayRow | null>(null);
  const [deleteError, setDeleteError] = useState("");

  const deleteMutation = useMutation({
    mutationFn: (inventoryId: string) => deleteInventoryItem(inventoryId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin", "inventory-all"] });
      setDeleteConfirm(null);
      setDeleteError("");
    },
    onError: (err: Error) => setDeleteError(err.message || "Failed to delete inventory row."),
  });

  const visibleRows = onlyLowStock ? rows.filter((r) => r.isLowStock) : rows;
  const lowStockCount = rows.filter((r) => r.isLowStock).length;

  return (
    <AdminTableCard
      title="Stock levels"
      icon={<Boxes className="h-4 w-4 text-[var(--color-green)]" />}
    >
      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
        <p className="text-sm text-[var(--color-muted)]">
          {rows.length} inventory row{rows.length === 1 ? "" : "s"}
          {lowStockCount > 0 ? ` · ${lowStockCount} at or below reorder level` : ""}
        </p>
        <label className="flex items-center gap-2 text-sm text-[var(--color-ink)]">
          <input
            type="checkbox"
            checked={onlyLowStock}
            onChange={(e) => setOnlyLowStock(e.target.checked)}
            className="h-4 w-4 rounded border-[var(--color-line)]"
          />
          Low stock only
        </label>
      </div>

      {isLoading ? (
        <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading inventory…</p>
      ) : null}
      {isError ? (
        <p className="py-8 text-center text-sm text-rose-700">Could not load inventory.</p>
      ) : null}

      {!isLoading && !isError && visibleRows.length === 0 ? (
        <p className="py-8 text-center text-sm text-[var(--color-muted)]">
          {onlyLowStock ? "Nothing is low on stock." : "No inventory rows yet."}
        </p>
      ) : null}

      {!isLoading && !isError && visibleRows.length > 0 ? (
        <div className="overflow-x-auto">
          <table className="w-full min-w-[640px] border-collapse text-[15px]">
            <caption className="sr-only">Inventory levels by product variant</caption>
            <thead>
              <tr className="border-b border-[var(--color-line)] text-left text-sm text-[var(--color-muted)]">
                <th className="pb-2 pr-4 font-medium">Product</th>
                <th className="pb-2 pr-4 font-medium">Size</th>
                <th className="pb-2 pr-4 font-medium">Available</th>
                <th className="pb-2 pr-4 font-medium">Reorder level</th>
                <th className="pb-2 font-medium sr-only">Actions</th>
              </tr>
            </thead>
            <tbody>
              {visibleRows.map((row) => (
                <tr key={row.inventoryId} className="border-b border-[var(--color-line)] last:border-0">
                  <td className="py-3 pr-4 text-[var(--color-ink)]">{row.productName}</td>
                  <td className="py-3 pr-4 text-[var(--color-muted)]">{row.sizeName}</td>
                  <td className="py-3 pr-4">
                    <span
                      className={
                        row.isLowStock
                          ? "rounded-full bg-amber-100 px-2.5 py-0.5 text-sm font-medium text-amber-900"
                          : "text-[var(--color-ink)]"
                      }
                    >
                      {row.quantityAvailable}
                    </span>
                  </td>
                  <td className="py-3 pr-4 text-[var(--color-muted)]">{row.reorderLevel}</td>
                  <td className="py-3 text-right">
                    <button
                      type="button"
                      onClick={() => {
                        setDeleteError("");
                        setDeleteConfirm(row);
                      }}
                      aria-label={`Delete inventory row for ${row.productName} — ${row.sizeName}`}
                      className="rounded-md p-1.5 text-[var(--color-muted)] hover:bg-red-50 hover:text-red-600"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : null}

      <DeleteEntityDialog
        entity={
          deleteConfirm ? { name: `${deleteConfirm.productName} — ${deleteConfirm.sizeName}` } : null
        }
        label="inventory row"
        isPending={deleteMutation.isPending}
        error={deleteError}
        warning="This removes stock tracking for this size — the variant itself is not deleted. Only do this for a variant that's been discontinued."
        onClose={() => {
          setDeleteConfirm(null);
          setDeleteError("");
        }}
        onConfirm={() => {
          if (deleteConfirm) deleteMutation.mutate(deleteConfirm.inventoryId);
        }}
      />
    </AdminTableCard>
  );
}
