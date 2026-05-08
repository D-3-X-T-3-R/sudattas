"use client";

import { Pencil, Trash2, Package } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/loading";
import type { ProductListRow } from "@/lib/admin-queries";
import { AdminTableCard } from "@/components/admin/admin-cards";

interface ProductsGridCardProps {
  products: ProductListRow[];
  productsLoading: boolean;
  productsError: boolean;
  productsErrorUi?: { title?: string; message?: string } | null;
  categoryNameById: Record<string, string>;
  getThumbnail: (product: ProductListRow) => string | null;
  onRetry: () => void;
  onOpenProduct: (product: ProductListRow) => void;
  onEditProduct: (product: ProductListRow) => void;
  onArchiveProduct: (product: ProductListRow) => void;
}

export function ProductsGridCard({
  products,
  productsLoading,
  productsError,
  productsErrorUi,
  categoryNameById,
  getThumbnail,
  onRetry,
  onOpenProduct,
  onEditProduct,
  onArchiveProduct,
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
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-5 xl:grid-cols-6">
          {products.map((p) => {
            const thumb = getThumbnail(p);
            return (
              <article
                key={p.productId}
                className="overflow-hidden rounded-md border border-[var(--color-line)] bg-white"
              >
                <button type="button" onClick={() => onOpenProduct(p)} className="block w-full text-left">
                  <div className="aspect-square w-full bg-[var(--color-surface-soft)]">
                    {thumb ? (
                      // eslint-disable-next-line @next/next/no-img-element
                      <img src={thumb} alt={p.name} className="h-full w-full object-cover" />
                    ) : (
                      <div className="flex h-full items-center justify-center text-xs text-[var(--color-muted)]">
                        No image
                      </div>
                    )}
                  </div>
                </button>

                <div className="space-y-1.5 border-t border-[var(--color-line)] p-2.5">
                  <p className="line-clamp-1 text-xs font-semibold text-[var(--color-ink)]">{p.name}</p>
                  <p className="text-[11px] text-[var(--color-muted)]">
                    {categoryNameById[p.categoryId ?? ""] ?? p.categoryId ?? "-"}
                  </p>
                  <p className="text-xs text-[var(--color-ink)]">{p.formatted}</p>
                  <p className="text-[11px] text-[var(--color-muted)]">Stock: {p.stockQuantity ?? "-"}</p>

                  <div className="flex items-center justify-between pt-1">
                    <span className="font-mono text-[11px] text-[var(--color-muted)]">#{p.productId}</span>
                    <div className="flex items-center gap-1">
                      <Button
                        type="button"
                        variant="outline"
                        className="h-8 w-8 p-0"
                        aria-label={`Edit ${p.name}`}
                        title="Edit"
                        onClick={() => onEditProduct(p)}
                      >
                        <Pencil className="h-4 w-4" />
                      </Button>
                      <Button
                        type="button"
                        variant="outline"
                        className="h-8 w-8 border-[#D8B2A7] p-0 text-[#7A5348]"
                        aria-label={`Archive ${p.name}`}
                        title="Archive"
                        onClick={() => onArchiveProduct(p)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
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
