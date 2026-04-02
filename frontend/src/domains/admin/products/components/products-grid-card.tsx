"use client";

import { Pencil, Trash2, Package } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Spinner } from "@/components/ui/loading";
import type { ProductListRow } from "@/lib/admin-queries";

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
    <Card className="mt-6 rounded-xl border-[var(--color-line)] border-l-4 border-l-violet-500 bg-white shadow-[var(--admin-card-shadow)]">
      <CardTitle className="flex items-center gap-2 text-[var(--color-muted)]">
        <Package className="h-4 w-4 text-violet-500" />
        Products
      </CardTitle>
      <CardContent className="mt-3">
        {productsError && (
          <div className="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
            <p className="font-medium">{productsErrorUi?.title ?? "Could not load products."}</p>
            <p className="mt-1 text-xs">{productsErrorUi?.message ?? "Please try again."}</p>
            <Button variant="outline" size="sm" className="mt-2" onClick={onRetry}>
              Try again
            </Button>
          </div>
        )}
        {productsLoading && !productsError && (
          <div className="flex justify-center py-8">
            <Spinner />
          </div>
        )}
        {!productsLoading && !productsError && products.length === 0 && (
          <p className="py-8 text-center text-sm text-[var(--color-muted)]">
            No products match. Create some in the <strong>Add product</strong> tab.
          </p>
        )}
        {!productsLoading && !productsError && products.length > 0 && (
          <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
            {products.map((p) => {
              const thumb = getThumbnail(p);
              return (
                <div
                  key={p.productId}
                  className="overflow-hidden rounded-xl border border-[var(--color-line)] bg-white"
                  onClick={() => onOpenProduct(p)}
                >
                  <div className="aspect-square w-full bg-[var(--color-surface)]">
                    {thumb ? (
                      <img
                        src={thumb}
                        alt={p.name}
                        className="h-full w-full object-cover"
                      />
                    ) : (
                      <div className="flex h-full items-center justify-center text-xs text-[var(--color-muted)]">
                        No image
                      </div>
                    )}
                  </div>
                  <div className="space-y-1.5 p-2">
                    <div className="line-clamp-1 text-xs font-semibold text-[var(--color-ink)]">
                      {p.name}
                    </div>
                    <div className="text-[11px] text-[var(--color-muted)]">
                      {categoryNameById[p.categoryId ?? ""] ?? p.categoryId ?? "—"}
                    </div>
                    <div className="text-xs text-[var(--color-ink)]">{p.formatted}</div>
                    <div className="text-[11px] text-[var(--color-muted)]">
                      Stock: {p.stockQuantity ?? "—"}
                    </div>
                    <div className="flex items-center justify-between pt-1">
                      <span className="font-mono text-[11px] text-[var(--color-muted)]">
                        #{p.productId}
                      </span>
                      <div className="flex items-center gap-1">
                        <Button
                          type="button"
                          variant="outline"
                          className="h-8 w-8 p-0"
                          aria-label={`Edit ${p.name}`}
                          title="Edit"
                          onClick={(e) => {
                            e.stopPropagation();
                            onEditProduct(p);
                          }}
                        >
                          <Pencil className="h-4 w-4" />
                        </Button>
                        <Button
                          type="button"
                          variant="outline"
                          className="h-8 w-8 p-0 text-red-600 border-red-200 hover:bg-red-50"
                          aria-label={`Archive ${p.name}`}
                          title="Archive"
                          onClick={(e) => {
                            e.stopPropagation();
                            onArchiveProduct(p);
                          }}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

