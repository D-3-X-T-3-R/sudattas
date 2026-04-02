"use client";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { cn } from "@/lib/utils";
import type { ProductImageListItem } from "@/lib/admin-queries";

export type AdminReorderableImage =
  | { type: "existing"; image: ProductImageListItem }
  | { type: "new"; file: File; previewUrl: string };

type ProductImagesDialogsProps = {
  imageDialogOpen: boolean;
  setImageDialogOpen: (open: boolean) => void;
  reorderImagesOpen: boolean;
  setReorderImagesOpen: (open: boolean) => void;
  editingProductId: string | null;
  existingProductImages: ProductImageListItem[];
  imagePreviews: string[];
  imageFiles: File[];
  reviewImagesList: AdminReorderableImage[];
  setReviewImagesList: React.Dispatch<React.SetStateAction<AdminReorderableImage[]>>;
  reviewDragIndex: number | null;
  setReviewDragIndex: React.Dispatch<React.SetStateAction<number | null>>;
  dragIndex: number | null;
  setDragIndex: React.Dispatch<React.SetStateAction<number | null>>;
  setImageFiles: React.Dispatch<React.SetStateAction<File[]>>;
  setOrderedProductImages: React.Dispatch<React.SetStateAction<AdminReorderableImage[] | null>>;
  setExistingProductImages: React.Dispatch<React.SetStateAction<ProductImageListItem[]>>;
  reorderableImages: AdminReorderableImage[];
  setReorderableImages: React.Dispatch<React.SetStateAction<AdminReorderableImage[]>>;
  reorderDragIndex: number | null;
  setReorderDragIndex: React.Dispatch<React.SetStateAction<number | null>>;
  productImagesLoadKey: string;
  getImageUrlWithCacheBuster: (img: ProductImageListItem | undefined, loadKey?: string) => string;
};

export function ProductImagesDialogs({
  imageDialogOpen,
  setImageDialogOpen,
  reorderImagesOpen,
  setReorderImagesOpen,
  editingProductId,
  existingProductImages,
  imagePreviews,
  imageFiles,
  reviewImagesList,
  setReviewImagesList,
  reviewDragIndex,
  setReviewDragIndex,
  dragIndex,
  setDragIndex,
  setImageFiles,
  setOrderedProductImages,
  setExistingProductImages,
  reorderableImages,
  setReorderableImages,
  reorderDragIndex,
  setReorderDragIndex,
  productImagesLoadKey,
  getImageUrlWithCacheBuster,
}: ProductImagesDialogsProps) {
  return (
    <>
      <Dialog open={imageDialogOpen} onOpenChange={setImageDialogOpen}>
        <DialogContent
          title="Review images"
          showClose
          onEscapeKeyDown={() => setImageDialogOpen(false)}
          onPointerDownOutside={() => setImageDialogOpen(false)}
        >
          <div className="space-y-4 text-sm text-[var(--color-muted)]">
            <p className="font-medium text-[var(--color-ink)]">
              {editingProductId
                ? "All product images - drag to reorder. First is the thumbnail."
                : "Add your product images"}
            </p>
            {editingProductId && (existingProductImages.length > 0 || imagePreviews.length > 0) ? (
              <div className="grid max-h-[60vh] grid-cols-3 gap-2 overflow-y-auto">
                {(reviewImagesList.length > 0
                  ? reviewImagesList
                  : [
                      ...existingProductImages.map((image) => ({
                        type: "existing" as const,
                        image,
                      })),
                      ...imagePreviews.map((url, i) => ({
                        type: "new" as const,
                        file: imageFiles[i],
                        previewUrl: url,
                      })),
                    ]
                ).map((item, idx) => (
                  <div
                    key={
                      item.type === "existing"
                        ? `existing-${item.image.imageId ?? item.image.url ?? idx}`
                        : `new-${idx}-${item.previewUrl}`
                    }
                    className={cn(
                      "relative aspect-square cursor-move overflow-hidden rounded border bg-[var(--color-ivory)] transition-transform duration-150",
                      item.type === "existing"
                        ? "border border-[var(--color-line)]"
                        : "border border-dashed border-[var(--color-line)]",
                      reviewDragIndex === idx &&
                        "scale-[1.03] border-[var(--color-ink)] ring-1 ring-[var(--color-ink)]"
                    )}
                    draggable
                    onDragStart={() => setReviewDragIndex(idx)}
                    onDragOver={(e) => {
                      e.preventDefault();
                      if (reviewDragIndex === null || reviewDragIndex === idx) return;
                      setReviewImagesList((prev) => {
                        const next = [...prev];
                        const [moved] = next.splice(reviewDragIndex, 1);
                        next.splice(idx, 0, moved);
                        return next;
                      });
                      setReviewDragIndex(idx);
                    }}
                    onDragEnd={() => setReviewDragIndex(null)}
                    onDrop={() => setReviewDragIndex(null)}
                  >
                    {item.type === "existing" ? (
                      <img
                        src={getImageUrlWithCacheBuster(item.image)}
                        alt=""
                        className="h-full w-full object-cover"
                      />
                    ) : (
                      <img src={item.previewUrl} alt="" className="h-full w-full object-cover" />
                    )}
                    <span className="absolute bottom-0 left-0 right-0 bg-black/50 px-1 py-0.5 text-[10px] text-white">
                      {item.type === "existing" ? "Existing" : "New"}
                    </span>
                    {idx === 0 && (
                      <span className="absolute left-1 top-1 rounded-full bg-[var(--color-ink)] px-2 py-0.5 text-[10px] font-medium text-white">
                        Thumbnail
                      </span>
                    )}
                  </div>
                ))}
              </div>
            ) : (
              <div className="grid grid-cols-3 gap-2">
                {imagePreviews.length > 0
                  ? imagePreviews.map((url, idx) => (
                      <div
                        key={imageFiles[idx]?.name ?? idx}
                        className={cn(
                          "relative aspect-square overflow-hidden rounded border border-dashed border-[var(--color-line)] bg-white cursor-move transition-transform duration-150",
                          dragIndex === idx && "scale-[1.03] border-[var(--color-ink)]"
                        )}
                        draggable
                        onDragStart={() => setDragIndex(idx)}
                        onDragOver={(e) => {
                          e.preventDefault();
                          if (dragIndex === null || dragIndex === idx) return;
                          setImageFiles((prev) => {
                            const next = [...prev];
                            const [moved] = next.splice(dragIndex, 1);
                            next.splice(idx, 0, moved);
                            return next;
                          });
                          setDragIndex(idx);
                        }}
                        onDrop={() => setDragIndex(null)}
                      >
                        <img
                          src={url}
                          alt={imageFiles[idx]?.name ?? "Preview"}
                          className="h-full w-full object-cover"
                        />
                        {idx === 0 && (
                          <div className="absolute left-1 top-1 rounded-full bg-[var(--color-ink)] px-2 py-0.5 text-[10px] font-medium text-white">
                            Thumbnail
                          </div>
                        )}
                      </div>
                    ))
                  : Array.from({ length: 6 }).map((_, idx) => (
                      <div
                        key={idx}
                        className="aspect-square rounded border border-dashed border-[var(--color-line)] bg-white"
                      />
                    ))}
              </div>
            )}
            {!editingProductId && imagePreviews.length > 0 && (
              <p className="text-[11px] text-[var(--color-muted)]">
                {imageFiles.length} image
                {imageFiles.length === 1 ? "" : "s"} selected.
              </p>
            )}
            {editingProductId && (existingProductImages.length > 0 || imagePreviews.length > 0) && (
              <p className="text-[11px] text-[var(--color-muted)]">
                {existingProductImages.length + imagePreviews.length} image
                {existingProductImages.length + imagePreviews.length === 1 ? "" : "s"} - drag to
                reorder.
              </p>
            )}
            <div className="flex justify-end gap-2 pt-2">
              <Button
                type="button"
                variant="outline"
                className="rounded-full border-[var(--color-line)] px-4"
                onClick={() => {
                  setImageDialogOpen(false);
                  setReviewDragIndex(null);
                }}
              >
                Cancel
              </Button>
              <Button
                type="button"
                className="rounded-full bg-[var(--color-ink)] px-4 text-white hover:bg-[var(--color-ink)]/90"
                onClick={() => {
                  if (editingProductId && (existingProductImages.length > 0 || imagePreviews.length > 0)) {
                    const listToApply =
                      reviewImagesList.length > 0
                        ? reviewImagesList
                        : ([
                            ...existingProductImages.map((image) => ({
                              type: "existing" as const,
                              image,
                            })),
                            ...imagePreviews.map((url, i) => ({
                              type: "new" as const,
                              file: imageFiles[i],
                              previewUrl: url,
                            })),
                          ] as AdminReorderableImage[]);
                    setOrderedProductImages(listToApply);
                    const existing = listToApply
                      .filter(
                        (x): x is { type: "existing"; image: ProductImageListItem } =>
                          x.type === "existing"
                      )
                      .map((x) => x.image);
                    const newFiles = listToApply
                      .filter(
                        (x): x is { type: "new"; file: File; previewUrl: string } =>
                          x.type === "new"
                      )
                      .map((x) => x.file);
                    setExistingProductImages(existing);
                    setImageFiles(newFiles);
                    setReviewDragIndex(null);
                  }
                  setImageDialogOpen(false);
                }}
              >
                Confirm
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
      <Dialog open={reorderImagesOpen} onOpenChange={setReorderImagesOpen}>
        <DialogContent
          title="Reorder product images"
          showClose
          onEscapeKeyDown={() => setReorderImagesOpen(false)}
          onPointerDownOutside={() => setReorderImagesOpen(false)}
        >
          <p className="mb-3 text-xs text-[var(--color-muted)]">
            Drag to change order. First image is the thumbnail.
          </p>
          <div className="grid max-h-[60vh] grid-cols-3 gap-2 overflow-y-auto">
            {reorderableImages.map((item, idx) => (
              <div
                key={
                  item.type === "existing"
                    ? `existing-${item.image.imageId ?? item.image.url ?? idx}`
                    : `new-${idx}-${item.previewUrl}`
                }
                className={cn(
                  "relative aspect-square cursor-move overflow-hidden rounded border bg-[var(--color-ivory)] transition-transform duration-150",
                  item.type === "existing"
                    ? "border border-[var(--color-line)]"
                    : "border border-dashed border-[var(--color-line)]",
                  reorderDragIndex === idx &&
                    "scale-[1.03] border-[var(--color-ink)] ring-1 ring-[var(--color-ink)]"
                )}
                draggable
                onDragStart={() => setReorderDragIndex(idx)}
                onDragOver={(e) => {
                  e.preventDefault();
                  if (reorderDragIndex === null || reorderDragIndex === idx) return;
                  setReorderableImages((prev) => {
                    const next = [...prev];
                    const [moved] = next.splice(reorderDragIndex, 1);
                    next.splice(idx, 0, moved);
                    return next;
                  });
                  setReorderDragIndex(idx);
                }}
                onDragEnd={() => setReorderDragIndex(null)}
                onDrop={() => setReorderDragIndex(null)}
              >
                {item.type === "existing" ? (
                  <img
                    src={getImageUrlWithCacheBuster(item.image, productImagesLoadKey)}
                    alt=""
                    className="h-full w-full object-cover"
                  />
                ) : (
                  <img src={item.previewUrl} alt="" className="h-full w-full object-cover" />
                )}
                <span className="absolute bottom-0 left-0 right-0 bg-black/50 px-1 py-0.5 text-[10px] text-white">
                  {item.type === "existing" ? "Existing" : "New"}
                </span>
                {idx === 0 && (
                  <span className="absolute left-1 top-1 rounded-full bg-[var(--color-ink)] px-2 py-0.5 text-[10px] font-medium text-white">
                    Thumbnail
                  </span>
                )}
              </div>
            ))}
          </div>
          <div className="mt-4 flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              className="rounded-full border-[var(--color-line)] px-4"
              onClick={() => setReorderImagesOpen(false)}
            >
              Cancel
            </Button>
            <Button
              type="button"
              className="rounded-full bg-[var(--color-ink)] px-4 text-white hover:bg-[var(--color-ink)]/90"
              onClick={() => {
                setOrderedProductImages(reorderableImages);
                const existing = reorderableImages
                  .filter(
                    (x): x is { type: "existing"; image: ProductImageListItem } =>
                      x.type === "existing"
                  )
                  .map((x) => x.image);
                const newFiles = reorderableImages
                  .filter(
                    (x): x is { type: "new"; file: File; previewUrl: string } => x.type === "new"
                  )
                  .map((x) => x.file);
                setExistingProductImages(existing);
                setImageFiles(newFiles);
                setReorderImagesOpen(false);
                setReorderDragIndex(null);
              }}
            >
              Apply order
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
}
