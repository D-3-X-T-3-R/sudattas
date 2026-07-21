"use client";

import { useMemo, useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import type { ProductListRow, ProductImageListItem } from "@/lib/admin-queries";

function getImageUrl(img: ProductImageListItem | undefined): string {
  if (!img) return "";
  const raw = img as Record<string, unknown>;
  const u =
    (img.url as string | undefined) ??
    (img.thumbnailUrl as string | undefined) ??
    (raw.thumbnail_url as string | undefined) ??
    (raw.url as string | undefined) ??
    "";
  return typeof u === "string" && u.trim() !== "" ? u : "";
}

interface ProductPreviewDialogProps {
  product: ProductListRow | null;
  open: boolean;
  onClose: () => void;
  categoryNameById: Record<string, string>;
  getProductStatusLabel: (statusId?: string | null) => string;
}

export function ProductPreviewDialog({
  product,
  open,
  onClose,
  categoryNameById,
  getProductStatusLabel,
}: ProductPreviewDialogProps) {
  const [selectedImageIndex, setSelectedImageIndex] = useState(0);
  const [touchStartX, setTouchStartX] = useState<number | null>(null);

  const imageUrls = useMemo(() => {
    if (!product) return [];
    return (product.images ?? [])
      .map((img) => {
        const u = getImageUrl(img);
        if (!u) return "";
        const sep = u.includes("?") ? "&" : "?";
        const id =
          (img as ProductImageListItem & { image_id?: string }).imageId ??
          (img as ProductImageListItem & { image_id?: string }).image_id ??
          "";
        return `${u}${sep}v=${id}`;
      })
      .filter((u) => !!u);
  }, [product]);

  if (!product) return null;

  const hasImages = imageUrls.length > 0;
  const activeImage = hasImages
    ? imageUrls[Math.min(selectedImageIndex, imageUrls.length - 1)]
    : null;

  return (
    <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
      <DialogContent className="sm:max-w-2xl">
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div
            className="relative overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-surface)]"
            onTouchStart={(e) => setTouchStartX(e.changedTouches[0]?.clientX ?? null)}
            onTouchEnd={(e) => {
              if (!hasImages || imageUrls.length <= 1 || touchStartX == null) return;
              const endX = e.changedTouches[0]?.clientX ?? touchStartX;
              const delta = endX - touchStartX;
              if (delta > 40) {
                setSelectedImageIndex((prev) =>
                  prev === 0 ? imageUrls.length - 1 : prev - 1
                );
              } else if (delta < -40) {
                setSelectedImageIndex((prev) =>
                  prev === imageUrls.length - 1 ? 0 : prev + 1
                );
              }
              setTouchStartX(null);
            }}
          >
            {activeImage ? (
              // eslint-disable-next-line @next/next/no-img-element
              <img
                src={activeImage}
                alt={product.name}
                className="h-full w-full object-cover"
              />
            ) : (
              <div className="flex aspect-square items-center justify-center text-sm text-[var(--color-muted)]">
                No image
              </div>
            )}
            {imageUrls.length > 1 && (
              <div className="pointer-events-none absolute inset-x-0 bottom-2 flex justify-center">
                <div className="rounded-full bg-black/40 px-2 py-0.5 text-[10px] text-white">
                  {selectedImageIndex + 1} / {imageUrls.length}
                </div>
              </div>
            )}
          </div>
          <div className="space-y-3">
            <div>
              <h3 className="text-xl font-semibold text-[var(--color-ink)]">
                {product.name}
              </h3>
              <p className="mt-1 text-lg font-medium text-[var(--color-ink)]">{product.formatted}</p>
            </div>
            {product.description ? (
              <p className="text-[15px] text-[var(--color-muted)]">{product.description}</p>
            ) : null}
            <div className="grid grid-cols-2 gap-x-4 gap-y-2.5 rounded-lg border border-[var(--color-line)] p-4 text-[15px]">
              <div>
                <p className="text-sm text-[var(--color-muted)]">Category</p>
                <p className="text-[var(--color-ink)]">{categoryNameById[product.categoryId ?? ""] ?? product.categoryId ?? "—"}</p>
              </div>
              <div>
                <p className="text-sm text-[var(--color-muted)]">Stock</p>
                <p className="text-[var(--color-ink)]">{product.stockQuantity ?? "—"}</p>
              </div>
              <div>
                <p className="text-sm text-[var(--color-muted)]">Status</p>
                <p className="text-[var(--color-ink)]">{getProductStatusLabel(product.productStatusId)}</p>
              </div>
              <div>
                <p className="text-sm text-[var(--color-muted)]">Fabric</p>
                <p className="text-[var(--color-ink)]">{product.fabric ?? "—"}</p>
              </div>
              <div>
                <p className="text-sm text-[var(--color-muted)]">Weave</p>
                <p className="text-[var(--color-ink)]">{product.weave ?? "—"}</p>
              </div>
              <div>
                <p className="text-sm text-[var(--color-muted)]">Occasion</p>
                <p className="text-[var(--color-ink)]">{product.occasion ?? "—"}</p>
              </div>
              <div>
                <p className="text-sm text-[var(--color-muted)]">Blouse piece</p>
                <p className="text-[var(--color-ink)]">
                  {product.hasBlousePiece == null ? "—" : product.hasBlousePiece ? "Yes" : "No"}
                </p>
              </div>
              <div>
                <p className="text-sm text-[var(--color-muted)]">SKU</p>
                <p className="text-[var(--color-ink)]">{product.sku ?? "—"}</p>
              </div>
              {product.careInstructions ? (
                <div className="col-span-2">
                  <p className="text-sm text-[var(--color-muted)]">Care instructions</p>
                  <p className="text-[var(--color-ink)]">{product.careInstructions}</p>
                </div>
              ) : null}
              <div className="col-span-2 border-t border-[var(--color-line)] pt-2.5 text-sm text-[var(--color-muted)]">
                Product #{product.productId} &middot; Slug: {product.slug ?? "—"}
              </div>
            </div>
            {imageUrls.length > 1 && (
              <div className="flex gap-2 pt-1">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() =>
                    setSelectedImageIndex((prev) =>
                      prev === 0 ? imageUrls.length - 1 : prev - 1
                    )
                  }
                >
                  Previous photo
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  onClick={() =>
                    setSelectedImageIndex((prev) =>
                      prev === imageUrls.length - 1 ? 0 : prev + 1
                    )
                  }
                >
                  Next photo
                </Button>
              </div>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
