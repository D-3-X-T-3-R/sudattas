"use client";

import { Button } from "@/components/ui/button";
import type { ProductImageListItem } from "@/lib/admin-queries";
import type { AdminReorderableImage } from "@/domains/admin/products/components/product-images-dialogs";
import type { Dispatch, RefObject, SetStateAction } from "react";
import { X } from "lucide-react";

type ProductImagesSectionProps = {
  imageError: string;
  imageMessage: string;
  fileInputRef: RefObject<HTMLInputElement | null>;
  setOrderedProductImages: Dispatch<SetStateAction<AdminReorderableImage[] | null>>;
  setImageFiles: Dispatch<SetStateAction<File[]>>;
  setImageError: Dispatch<SetStateAction<string>>;
  setImageMessage: Dispatch<SetStateAction<string>>;
  setImageDialogOpen: Dispatch<SetStateAction<boolean>>;
  editingProductId: string | null;
  orderedProductImages: AdminReorderableImage[] | null;
  existingProductImages: ProductImageListItem[];
  imagePreviews: string[];
  imageFiles: File[];
  setReorderableImages: Dispatch<SetStateAction<AdminReorderableImage[]>>;
  setReorderImagesOpen: Dispatch<SetStateAction<boolean>>;
  productImagesLoadKey: string;
  getImageUrlWithCacheBuster: (img: ProductImageListItem | undefined, loadKey?: string) => string;
  setExistingProductImages: Dispatch<SetStateAction<ProductImageListItem[]>>;
};

export function ProductImagesSection({
  imageError,
  imageMessage,
  fileInputRef,
  setOrderedProductImages,
  setImageFiles,
  setImageError,
  setImageMessage,
  setImageDialogOpen,
  editingProductId,
  orderedProductImages,
  existingProductImages,
  imagePreviews,
  imageFiles,
  setReorderableImages,
  setReorderImagesOpen,
  productImagesLoadKey,
  getImageUrlWithCacheBuster,
  setExistingProductImages,
}: ProductImagesSectionProps) {
  return (
    <div className="mt-8 border-t border-[var(--color-line)] pt-4">
      <h3 className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">
        Images *
      </h3>
      <p className="mt-2 text-xs text-[var(--color-muted)]">
        Select at least one image. All selected images will be uploaded and linked after the product is
        created.
      </p>
      {imageError && (
        <div
          className="mt-3 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-800"
          role="alert"
        >
          {imageError}
        </div>
      )}
      {imageMessage && (
        <div
          className="mt-3 rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-2 text-xs text-emerald-800"
          role="status"
        >
          {imageMessage}
        </div>
      )}
      <div className="mt-3 space-y-3 text-xs text-[var(--color-muted)]">
        <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
          <input
            ref={fileInputRef}
            type="file"
            accept="image/*"
            multiple
            onChange={(e) => {
              const files = Array.from(e.target.files ?? []);
              setOrderedProductImages(null);
              setImageFiles(files);
              setImageError("");
              setImageMessage("");
              if (files.length > 0) {
                setImageDialogOpen(true);
              }
            }}
            className="hidden"
          />
          <Button
            type="button"
            variant="outline"
            className="h-9 rounded-full border-[var(--color-line)] px-4 text-xs"
            onClick={() => fileInputRef.current?.click()}
          >
            Choose images...
          </Button>
        </div>
        <p className="text-[11px] text-[var(--color-muted)]">
          {editingProductId
            ? "Add more images below; they will upload when you click Update product."
            : "All selected images will be uploaded when you click Add product."}
        </p>
        {(orderedProductImages !== null || existingProductImages.length > 0 || imagePreviews.length > 0) && (
          <div className="mt-4">
            <div className="mb-2 flex items-center justify-between gap-2">
              <p className="text-xs font-medium text-[var(--color-ink)]">
                Added images
                {orderedProductImages != null
                  ? ` (${orderedProductImages.length})`
                  : existingProductImages.length > 0 && imagePreviews.length > 0
                    ? ` (${existingProductImages.length} existing, ${imagePreviews.length} new)`
                    : existingProductImages.length > 0
                      ? ` (${existingProductImages.length})`
                      : ` (${imagePreviews.length} selected)`}
              </p>
              {editingProductId && (existingProductImages.length > 0 || imagePreviews.length > 0) && (
                <Button
                  type="button"
                  variant="outline"
                  className="h-8 rounded-full border-[var(--color-line)] px-3 text-xs"
                  onClick={() => {
                    const list =
                      orderedProductImages ??
                      [
                        ...existingProductImages.map((image) => ({
                          type: "existing" as const,
                          image,
                        })),
                        ...imagePreviews.map((previewUrl, i) => ({
                          type: "new" as const,
                          file: imageFiles[i],
                          previewUrl,
                        })),
                      ];
                    setReorderableImages(list);
                    setReorderImagesOpen(true);
                  }}
                >
                  Reorder images
                </Button>
              )}
            </div>
            <div className="flex flex-wrap gap-3">
              {orderedProductImages != null ? (
                orderedProductImages.map((item, idx) => (
                  <div
                    key={
                      item.type === "existing"
                        ? `existing-${idx}-${item.image.imageId ?? item.image.url ?? ""}`
                        : `new-${idx}-${item.previewUrl}`
                    }
                    className={`relative aspect-square w-24 shrink-0 overflow-hidden rounded-lg border bg-[var(--color-ivory)] ${
                      item.type === "existing"
                        ? "border-[var(--color-line)]"
                        : "border-dashed border-[var(--color-line)]"
                    }`}
                  >
                    {item.type === "existing" ? (
                      (() => {
                        const src = getImageUrlWithCacheBuster(item.image, productImagesLoadKey);
                        return src ? (
                          <img src={src} alt="" className="h-full w-full object-cover" />
                        ) : (
                          <div className="flex h-full w-full items-center justify-center bg-[var(--color-line)]/30 text-[10px] text-[var(--color-muted)]">
                            No image
                          </div>
                        );
                      })()
                    ) : (
                      <img src={item.previewUrl} alt="" className="h-full w-full object-cover" />
                    )}
                    <span className="absolute bottom-0 left-0 right-0 bg-black/50 px-1 py-0.5 text-[10px] text-white">
                      {item.type === "existing" ? "Existing" : "New"}
                    </span>
                    {editingProductId && (
                      <button
                        type="button"
                        aria-label="Remove image (saved when you click Update product)"
                        className="absolute right-1 top-1 flex h-6 w-6 items-center justify-center rounded-full bg-red-500/90 text-white hover:bg-red-600"
                        onClick={() => {
                          setOrderedProductImages((prev) => (prev ? prev.filter((_, i) => i !== idx) : prev));
                          setImageMessage("Image will be removed when you click Update product.");
                        }}
                      >
                        <X className="h-3.5 w-3.5" />
                      </button>
                    )}
                  </div>
                ))
              ) : (
                <>
                  {existingProductImages.map((img, idx) => (
                    <div
                      key={`existing-${idx}-${img.imageId ?? img.url ?? img.thumbnailUrl ?? ""}`}
                      className="relative aspect-square w-24 shrink-0 overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-ivory)]"
                    >
                      {getImageUrlWithCacheBuster(img, productImagesLoadKey) ? (
                        <img
                          src={getImageUrlWithCacheBuster(img, productImagesLoadKey)}
                          alt=""
                          className="h-full w-full object-cover"
                        />
                      ) : (
                        <div className="flex h-full w-full items-center justify-center bg-[var(--color-line)]/30 text-[10px] text-[var(--color-muted)]">
                          No image
                        </div>
                      )}
                      {editingProductId && (
                        <span className="absolute bottom-0 left-0 right-0 bg-black/50 px-1 py-0.5 text-[10px] text-white">
                          Existing
                        </span>
                      )}
                      {editingProductId && (
                        <button
                          type="button"
                          aria-label="Remove image (saved when you click Update product)"
                          className="absolute right-1 top-1 flex h-6 w-6 items-center justify-center rounded-full bg-red-500/90 text-white hover:bg-red-600"
                          onClick={() => {
                            const toRemove = img;
                            setExistingProductImages((prev) => {
                              const i = prev.findIndex((im) => im === toRemove);
                              if (i === -1) return prev;
                              return prev.filter((_, j) => j !== i);
                            });
                            setImageMessage("Image will be removed when you click Update product.");
                          }}
                        >
                          <X className="h-3.5 w-3.5" />
                        </button>
                      )}
                    </div>
                  ))}
                  {imagePreviews.map((url, idx) => (
                    <div
                      key={`new-${idx}-${url}`}
                      className="relative aspect-square w-24 shrink-0 overflow-hidden rounded-lg border border-dashed border-[var(--color-line)] bg-[var(--color-ivory)]"
                    >
                      <img src={url} alt="" className="h-full w-full object-cover" />
                      <span className="absolute bottom-0 left-0 right-0 bg-black/50 px-1 py-0.5 text-[10px] text-white">
                        New
                      </span>
                      <button
                        type="button"
                        aria-label="Remove image"
                        className="absolute right-1 top-1 flex h-6 w-6 items-center justify-center rounded-full bg-red-500/90 text-white hover:bg-red-600"
                        onClick={() => {
                          setImageFiles((prev) => prev.filter((_, i) => i !== idx));
                          setImageError("");
                          setImageMessage("");
                        }}
                      >
                        <X className="h-3.5 w-3.5" />
                      </button>
                    </div>
                  ))}
                </>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
