"use client";
/* eslint-disable @next/next/no-img-element */

import { useState } from "react";
import { Button } from "@/components/ui/button";
import type { ProductImageListItem } from "@/lib/admin-queries";
import type { AdminReorderableImage } from "@/domains/admin/products/components/product-images-dialogs";
import type { Dispatch, RefObject, SetStateAction } from "react";
import { X, UploadCloud, FolderInput } from "lucide-react";
import { cn } from "@/lib/utils";

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
  onRequestMoveImage: (img: ProductImageListItem) => void;
};

function ImagesToolbar({
  fileInputRef,
  setOrderedProductImages,
  setImageFiles,
  setImageError,
  setImageMessage,
  setImageDialogOpen,
}: Pick<
  ProductImagesSectionProps,
  | "fileInputRef"
  | "setOrderedProductImages"
  | "setImageFiles"
  | "setImageError"
  | "setImageMessage"
  | "setImageDialogOpen"
>) {
  const [isDragOver, setIsDragOver] = useState(false);

  const applyFiles = (fileList: FileList | null) => {
    const files = Array.from(fileList ?? []).filter((f) => f.type.startsWith("image/"));
    if (files.length === 0) return;
    setOrderedProductImages(null);
    setImageFiles(files);
    setImageError("");
    setImageMessage("");
    setImageDialogOpen(true);
  };

  return (
    <div
      onDragOver={(e) => {
        e.preventDefault();
        setIsDragOver(true);
      }}
      onDragLeave={() => setIsDragOver(false)}
      onDrop={(e) => {
        e.preventDefault();
        setIsDragOver(false);
        applyFiles(e.dataTransfer.files);
      }}
      className={cn(
        "flex flex-col items-center gap-2 rounded-xl border-2 border-dashed px-6 py-8 text-center transition-colors",
        isDragOver ? "border-[var(--color-green)] bg-[var(--color-surface-soft)]" : "border-[var(--color-line)]"
      )}
    >
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        multiple
        onChange={(e) => applyFiles(e.target.files)}
        className="hidden"
      />
      <UploadCloud className="h-6 w-6 text-[var(--color-muted)]" aria-hidden="true" />
      <p className="text-sm text-[var(--color-muted)]">Drag photos here, or</p>
      <Button
        type="button"
        variant="outline"
        className="rounded-full border-[var(--color-line)] px-5"
        onClick={() => fileInputRef.current?.click()}
      >
        Choose photos…
      </Button>
    </div>
  );
}

function ExistingOrNewImageGrid({
  editingProductId,
  orderedProductImages,
  existingProductImages,
  imagePreviews,
  setOrderedProductImages,
  setImageMessage,
  setExistingProductImages,
  setImageFiles,
  setImageError,
  productImagesLoadKey,
  getImageUrlWithCacheBuster,
  onRequestMoveImage,
}: Pick<
  ProductImagesSectionProps,
  | "editingProductId"
  | "orderedProductImages"
  | "existingProductImages"
  | "imagePreviews"
  | "setOrderedProductImages"
  | "setImageMessage"
  | "setExistingProductImages"
  | "setImageFiles"
  | "setImageError"
  | "productImagesLoadKey"
  | "getImageUrlWithCacheBuster"
  | "onRequestMoveImage"
>) {
  if (orderedProductImages != null) {
    return (
      <>
        {orderedProductImages.map((item, idx) => (
          <div
            key={item.type === "existing" ? `existing-${idx}-${item.image.imageId ?? item.image.url ?? ""}` : `new-${idx}-${item.previewUrl}`}
            className={`relative aspect-[2/3] w-28 shrink-0 overflow-hidden rounded-lg border bg-[var(--color-ivory)] ${
              item.type === "existing" ? "border-[var(--color-line)]" : "border-dashed border-[var(--color-line)]"
            }`}
          >
            {item.type === "existing" ? (
              (() => {
                const src = getImageUrlWithCacheBuster(item.image, productImagesLoadKey);
                return src ? <img src={src} alt="" className="h-full w-full object-cover" /> : <div className="flex h-full w-full items-center justify-center bg-[var(--color-line)]/30 text-[10px] text-[var(--color-muted)]">No image</div>;
              })()
            ) : (
              <img src={item.previewUrl} alt="" className="h-full w-full object-cover" />
            )}
            <span className="absolute bottom-0 left-0 right-0 bg-black/50 px-1 py-0.5 text-[10px] text-white">{item.type === "existing" ? "Existing" : "New"}</span>
            {editingProductId && (
              <button
                type="button"
                aria-label="Remove image (saved when you click Update product)"
                className="absolute right-1 top-1 flex h-8 w-8 items-center justify-center rounded-full bg-red-500/90 text-white hover:bg-red-600"
                onClick={() => {
                  setOrderedProductImages((prev) => (prev ? prev.filter((_, i) => i !== idx) : prev));
                  setImageMessage("Image will be removed when you click Update product.");
                }}
              >
                <X className="h-4 w-4" />
              </button>
            )}
          </div>
        ))}
      </>
    );
  }

  return (
    <>
      {existingProductImages.map((img, idx) => (
        <div
          key={`existing-${idx}-${img.imageId ?? img.url ?? img.thumbnailUrl ?? ""}`}
          className="relative aspect-[2/3] w-28 shrink-0 overflow-hidden rounded-lg border border-[var(--color-line)] bg-[var(--color-ivory)]"
        >
          {getImageUrlWithCacheBuster(img, productImagesLoadKey) ? <img src={getImageUrlWithCacheBuster(img, productImagesLoadKey)} alt="" className="h-full w-full object-cover" /> : <div className="flex h-full w-full items-center justify-center bg-[var(--color-line)]/30 text-[10px] text-[var(--color-muted)]">No image</div>}
          {editingProductId && <span className="absolute bottom-0 left-0 right-0 bg-black/50 px-1 py-0.5 text-[10px] text-white">Existing</span>}
          {editingProductId && (
            <button
              type="button"
              aria-label="Remove image (saved when you click Update product)"
              className="absolute right-1 top-1 flex h-8 w-8 items-center justify-center rounded-full bg-red-500/90 text-white hover:bg-red-600"
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
          {editingProductId && img.imageId && (
            <button
              type="button"
              aria-label="Move this image to a different product"
              className="absolute left-1 top-1 flex h-8 w-8 items-center justify-center rounded-full bg-black/60 text-white hover:bg-black/80"
              onClick={() => onRequestMoveImage(img)}
            >
              <FolderInput className="h-3.5 w-3.5" />
            </button>
          )}
        </div>
      ))}
      {imagePreviews.map((url, idx) => (
        <div key={`new-${idx}-${url}`} className="relative aspect-[2/3] w-28 shrink-0 overflow-hidden rounded-lg border border-dashed border-[var(--color-line)] bg-[var(--color-ivory)]">
          <img src={url} alt="" className="h-full w-full object-cover" />
          <span className="absolute bottom-0 left-0 right-0 bg-black/50 px-1 py-0.5 text-[10px] text-white">New</span>
          <button
            type="button"
            aria-label="Remove image"
            className="absolute right-1 top-1 flex h-8 w-8 items-center justify-center rounded-full bg-red-500/90 text-white hover:bg-red-600"
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
  );
}

export function ProductImagesSection(props: ProductImagesSectionProps) {
  const {
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
  } = props;

  return (
    <div>
      <h3 className="text-[15px] font-semibold text-[var(--color-ink)]">Photos *</h3>
      <p className="mt-1.5 text-sm text-[var(--color-muted)]">Add at least one photo. Photos are uploaded once you save the product.</p>

      {imageError && <div className="mt-3 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800" role="alert">{imageError}</div>}
      {imageMessage && <div className="mt-3 rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-600" role="status">{imageMessage}</div>}

      <div className="mt-3 space-y-3 text-sm text-[var(--color-muted)]">
        <ImagesToolbar
          fileInputRef={fileInputRef}
          setOrderedProductImages={setOrderedProductImages}
          setImageFiles={setImageFiles}
          setImageError={setImageError}
          setImageMessage={setImageMessage}
          setImageDialogOpen={setImageDialogOpen}
        />

        <p className="text-sm text-[var(--color-muted)]">
          {editingProductId ? "Add more photos below; they upload when you click Update product." : "All selected photos upload when you click Add product."}
        </p>

        {(orderedProductImages !== null || existingProductImages.length > 0 || imagePreviews.length > 0) && (
          <div className="mt-4">
            <div className="mb-2 flex items-center justify-between gap-2">
              <p className="text-sm font-medium text-[var(--color-ink)]">
                Added photos
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
                  className="h-9 rounded-full border-[var(--color-line)] px-4 text-sm"
                  onClick={() => {
                    const list = orderedProductImages ?? [
                      ...existingProductImages.map((image) => ({ type: "existing" as const, image })),
                      ...imagePreviews.map((previewUrl, i) => ({ type: "new" as const, file: imageFiles[i], previewUrl })),
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
              <ExistingOrNewImageGrid {...props} />
            </div>
          </div>
        )}
      </div>
    </div>
  );
}


