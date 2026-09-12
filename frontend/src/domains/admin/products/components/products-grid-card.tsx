"use client";

import { Pencil, Trash2, RotateCcw, Package, Ban } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/loading";
import type { ProductListRow } from "@/lib/admin-queries";
import { AdminTableCard } from "@/components/admin/admin-cards";
import { StatusBadge } from "@/components/admin/status-badge";

/** ProductStatuses.id for "archived" — matches the admin status dropdown's own hardcoded values. */
const ARCHIVED_STATUS_ID = "3";

interface ProductsGridCardProps {
  products: ProductListRow[];
  productsLoading: boolean;
  productsError: boolean;
  productsErrorUi?: { title?: string; message?: string } | null;
  categoryNameById: Record<string, string>;
  getThumbnail: (product: ProductListRow) => string | null;
  getProductStatusLabel: (statusId?: string | null) => string;
  onRetry: () => void;
  onOpenProduct: (product: ProductListRow) => void;
  onEditProduct: (product: ProductListRow) => void;
  onArchiveProduct: (product: ProductListRow) => void;
  onActivateProduct: (product: ProductListRow) => void;
  onPermanentlyDeleteProduct: (product: ProductListRow) => void;
}

export function ProductsGridCard({
  products,
  productsLoading,
  productsError,
  productsErrorUi,
  categoryNameById,
  getThumbnail,
  getProductStatusLabel,
  onRetry,
  onOpenProduct,
  onEditProduct,
  onArchiveProduct,
  onActivateProduct,
  onPermanentlyDeleteProduct,
}: ProductsGridCardProps) {
  return (
    <AdminTableCard title="Products" icon={<Package className="h-4 w-4 text-[var(--color-green)]" />} className="mt-6">
      {productsError ? (
        <div className="rounded-md border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
          <p className="font-medium">{productsErrorUi?.title ?? "Could not load products."}</p>
          <p className="mt-1 text-xs">{productsErrorUi?.message ?? "Please try again."}</p>
          <Button variant="outline" size="sm" className="mt-2" onClick={onRetry}>
            Try again
          </Button>
        </div>
      ) : null}

      {productsLoading && !productsError ? (
        <div className="flex justify-center py-8">
          <Spinner />
        </div>
      ) : null}

      {!productsLoading && !productsError && products.length === 0 ? (
        <p className="py-8 text-center text-sm text-[var(--color-muted)]">
          No products match these filters.
        </p>
      ) : null}

      {!productsLoading && !productsError && products.length > 0 ? (
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4">
          {products.map((p) => {
            const thumb = getThumbnail(p);
            return (
              <article
                key={p.productId}
                className="overflow-hidden rounded-xl border border-[var(--color-line)] bg-white"
              >
                <button type="button" onClick={() => onOpenProduct(p)} className="block w-full text-left">
                  <div className="aspect-[2/3] w-full bg-[var(--color-surface-soft)]">
                    {thumb ? (
                      // eslint-disable-next-line @next/next/no-img-element
                      <img src={thumb} alt={p.name} className="h-full w-full object-cover" />
                    ) : (
                      <div className="flex h-full items-center justify-center text-sm text-[var(--color-muted)]">
                        No image
                      </div>
                    )}
                  </div>
                </button>

                <div className="space-y-1.5 border-t border-[var(--color-line)] p-3.5">
                  <div className="flex items-start justify-between gap-2">
                    <p className="line-clamp-1 text-[15px] font-semibold text-[var(--color-ink)]">{p.name}</p>
                    <StatusBadge
                      label={getProductStatusLabel(p.productStatusId)}
                      className="shrink-0 px-2 py-0.5 text-xs"
                    />
                  </div>
                  <p className="text-sm text-[var(--color-muted)]">
                    {categoryNameById[p.categoryId ?? ""] ?? p.categoryId ?? "-"}
                  </p>
                  <p className="text-[15px] font-medium text-[var(--color-ink)]">{p.formatted}</p>
                  <p className="text-sm text-[var(--color-muted)]">Stock: {p.stockQuantity ?? "-"}</p>

                  <div className="flex flex-col gap-2 pt-2">
                    <Button
                      type="button"
                      variant="outline"
                      className="w-full justify-center gap-2"
                      aria-label={`Edit ${p.name}`}
                      onClick={() => onEditProduct(p)}
                    >
                      <Pencil className="h-4 w-4" />
                      Edit
                    </Button>
                    {p.productStatusId === ARCHIVED_STATUS_ID ? (
                      <Button
                        type="button"
                        variant="outline"
                        className="w-full justify-center gap-2 border-emerald-200 text-emerald-700 hover:border-emerald-300 hover:bg-emerald-50"
                        aria-label={`Activate ${p.name}`}
                        onClick={() => onActivateProduct(p)}
                      >
                        <RotateCcw className="h-4 w-4" />
                        Activate
                      </Button>
                    ) : (
                      <Button
                        type="button"
                        variant="outline"
                        className="w-full justify-center gap-2 border-[#D8B2A7] text-[#7A5348] hover:border-[#D8B2A7]"
                        aria-label={`Archive ${p.name}`}
                        onClick={() => onArchiveProduct(p)}
                      >
                        <Trash2 className="h-4 w-4" />
                        Archive
                      </Button>
                    )}
                    <Button
                      type="button"
                      variant="outline"
                      className="w-full justify-center gap-2 border-red-200 text-red-600 hover:border-red-300 hover:bg-red-50"
                      aria-label={`Permanently delete ${p.name}`}
                      onClick={() => onPermanentlyDeleteProduct(p)}
                    >
                      <Ban className="h-4 w-4" />
                      Permanently remove
                    </Button>
                  </div>
                </div>
              </article>
            );
          })}
        </div>
      ) : null}
    </AdminTableCard>
  );
}
