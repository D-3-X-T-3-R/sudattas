"use client";
/* eslint-disable max-lines */
/* eslint-disable max-lines-per-function */

import { useState, useEffect, useRef } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { gqlAdmin } from "@/lib/graphqlAdmin";
import { adminProductFormSchema } from "@/lib/schemas";
import {
  fetchCategories,
  fetchProductsList,
  fetchProductById,
  fetchSizes,
  fetchColors,
  fetchFabrics,
  fetchWeaves,
  fetchOccasions,
  searchProductMoods,
  searchProductMoodMappingsByProduct,
  createProductMood,
  createProductVariant,
  updateProductVariant,
  deleteProductVariant,
  createProductMoodMapping,
  deleteProductMoodMapping,
  createInventoryItem,
  searchInventoryByVariantId,
  updateInventoryItem,
  type ProductListRow,
  type ProductListRowWithVariantStock,
  type ProductImageListItem,
  type CategoryRow,
  type SizeRow,
  type ColorRow,
  type ProductMoodRow,
  type FabricRow,
  type WeaveRow,
  type OccasionRow,
} from "@/lib/admin-queries";
import type { AdminProductVariantRow } from "@/lib/schemas";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ProductsFiltersCard } from "@/domains/admin/products/components/products-filters-card";
import { ProductsGridCard } from "@/domains/admin/products/components/products-grid-card";
import { ProductPreviewDialog } from "@/domains/admin/products/components/product-preview-dialog";
import { ArchiveProductDialog } from "@/domains/admin/products/components/archive-product-dialog";
import {
  ProductImagesDialogs,
  type AdminReorderableImage,
} from "@/domains/admin/products/components/product-images-dialogs";
import { ProductVariantsSection } from "@/domains/admin/products/components/product-variants-section";
import { ProductMoodsSection } from "@/domains/admin/products/components/product-moods-section";
import { ProductImagesSection } from "@/domains/admin/products/components/product-images-section";
import { ProductFormPreview } from "@/domains/admin/products/components/product-form-preview";
import { AdminPageShell } from "@/components/admin/admin-page-shell";
import { cn } from "@/lib/utils";
import { useToast } from "@/components/ui/toast";
import { Spinner } from "@/components/ui/loading";
import { Pencil, Package, Plus } from "lucide-react";
import {
  MAX_MONEY_PAISE,
  optionalRupeesInputToPaise,
  paiseToRupeesInput,
  rupeesInputToPaise,
} from "@/lib/money";
import { toRouteFailureUi } from "@/lib/route-state";

type ProductFormState = {
  name: string;
  description: string;
  priceRupees: string;
  stockQuantity: string;
  categoryId: string;
  sku: string;
  slug: string;
  fabric: string;
  weave: string;
  occasion: string;
  hasBlousePiece: boolean;
  careInstructions: string;
  productStatusId: string;
};

const DRAFT_KEY = "sudattas_admin_product_draft";

type FormSection = "basics" | "details" | "stock" | "moods" | "photos";

const SECTION_FOR_FIELD: Partial<Record<keyof ProductFormState, FormSection>> = {
  name: "basics",
  description: "basics",
  priceRupees: "basics",
  categoryId: "basics",
  sku: "details",
  slug: "details",
  fabric: "details",
  weave: "details",
  occasion: "details",
  careInstructions: "details",
  productStatusId: "details",
};

/** Derive a URL-safe slug from a product name (lowercase, hyphenated, no repeats). */
function slugify(name: string): string {
  return name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 100);
}

/** Get first non-empty URL from an image (supports camelCase and snake_case from API). */
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

/** URL with cache-buster so the browser doesn't show stale cached image content. Pass loadKey (e.g. from beginEdit) to force refetch when opening edit. */
function getImageUrlWithCacheBuster(
  img: ProductImageListItem | undefined,
  loadKey?: string
): string {
  const u = getImageUrl(img);
  if (!u) return "";
  const raw = img as Record<string, unknown>;
  const id =
    (img as ProductImageListItem & { image_id?: string }).imageId ??
    (raw.image_id as string) ??
    "";
  const v = loadKey ? `${id}-${loadKey}` : id;
  const sep = u.includes("?") ? "&" : "?";
  return `${u}${sep}v=${v}`;
}

/** Thumbnail URL with cache-buster so list card shows updated image after sync. */
function getProductThumbnailWithCacheBuster(p: ProductListRow): string | null {
  const images = p.images ?? [];
  for (let i = 0; i < images.length; i++) {
    const img = images[i];
    const u = getImageUrl(img);
    if (!u) continue;
    const sep = u.includes("?") ? "&" : "?";
    const id =
      (img as ProductImageListItem & { image_id?: string }).imageId ??
      (img as ProductImageListItem & { image_id?: string }).image_id ??
      "";
    return `${u}${sep}v=${id}`;
  }
  return null;
}

function getProductStatusLabel(statusId?: string | null): string {
  if (!statusId) return "—";
  if (statusId === "1") return "Draft";
  if (statusId === "2") return "Active";
  if (statusId === "3") return "Archived";
  return statusId;
}

export default function AdminProductsPage() {
  const queryClient = useQueryClient();
  const { showToast } = useToast();
  const {
    data: categories = [],
    isLoading: categoriesLoading,
    isError: categoriesError,
    error: categoriesErrorObj,
    refetch: refetchCategories,
  } = useQuery<CategoryRow[], Error>({
    queryKey: ["admin", "categories"],
    queryFn: fetchCategories,
  });

  const [searchName, setSearchName] = useState("");
  const [searchCategoryId, setSearchCategoryId] = useState("");
  const [searchMoodId, setSearchMoodId] = useState("");
  const [searchFabric, setSearchFabric] = useState("");
  const [searchWeave, setSearchWeave] = useState("");
  const [searchOccasion, setSearchOccasion] = useState("");
  const [searchProductStatusId, setSearchProductStatusId] = useState("");
  const [searchPriceMinRupees, setSearchPriceMinRupees] = useState("");
  const [searchPriceMaxRupees, setSearchPriceMaxRupees] = useState("");
  const [searchLimit, setSearchLimit] = useState("20");
  const [activeTab, setActiveTab] = useState<"view" | "add">("view");
  const [appliedSearch, setAppliedSearch] = useState<{
    name?: string;
    categoryId?: string;
    moodId?: string;
    fabric?: string;
    weave?: string;
    occasion?: string;
    limit?: string;
    productStatusId?: string;
    startingPricePaise?: string;
    endingPricePaise?: string;
  }>({ limit: "20" });

  const {
    data: products = [],
    isLoading: productsLoading,
    isError: productsError,
    error: productsErrorObj,
    refetch: refetchProducts,
  } = useQuery<ProductListRow[], Error>({
    queryKey: ["admin", "products", appliedSearch],
    queryFn: () => fetchProductsList(appliedSearch),
    enabled: true,
  });

  const { data: sizes = [] } = useQuery<SizeRow[], Error>({
    queryKey: ["admin", "sizes"],
    queryFn: fetchSizes,
    enabled: activeTab === "add",
  });
  const { data: colors = [] } = useQuery<ColorRow[], Error>({
    queryKey: ["admin", "colors"],
    queryFn: fetchColors,
    enabled: activeTab === "add",
  });
  const { data: weaves = [] } = useQuery<WeaveRow[], Error>({
    queryKey: ["admin", "weaves"],
    queryFn: fetchWeaves,
  });
  const { data: occasions = [] } = useQuery<OccasionRow[], Error>({
    queryKey: ["admin", "occasions"],
    queryFn: fetchOccasions,
  });
  const { data: existingMoods = [] } = useQuery<ProductMoodRow[], Error>({
    queryKey: ["admin", "productMoods"],
    queryFn: () => searchProductMoods({}),
  });
  const createMoodMutation = useMutation({
    mutationFn: (name: string) => createProductMood(name),
    onSuccess: (created) => {
      if (created) {
        queryClient.invalidateQueries({ queryKey: ["admin", "productMoods"] });
        setSelectedMoodIds((prev) => (prev.includes(created.moodId) ? prev : [...prev, created.moodId]));
        setNewMoodName("");
        setMoodCreateError("");
      }
    },
    onError: (err: Error) => setMoodCreateError(err.message || "Failed to create mood"),
  });

  const { data: fabrics = [] } = useQuery<FabricRow[], Error>({
    queryKey: ["admin", "fabrics"],
    queryFn: fetchFabrics,
  });

  const [archiveConfirm, setArchiveConfirm] = useState<ProductListRow | null>(null);
  const [selectedProduct, setSelectedProduct] = useState<ProductListRow | null>(null);

  const [editingProductId, setEditingProductId] = useState<string | null>(null);

  const [form, setForm] = useState<ProductFormState>({
    name: "",
    description: "",
    priceRupees: "",
    stockQuantity: "0",
    categoryId: categories[0]?.categoryId ?? "",
    sku: "",
    slug: "",
    fabric: "",
    weave: "",
    occasion: "",
    hasBlousePiece: true,
    careInstructions: "",
    productStatusId: "",
  });
  const [variants, setVariants] = useState<AdminProductVariantRow[]>([]);
  const [selectedMoodIds, setSelectedMoodIds] = useState<string[]>([]);
  const [newMoodName, setNewMoodName] = useState("");
  const [moodCreateError, setMoodCreateError] = useState("");
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [showNewCategory, setShowNewCategory] = useState(false);
  const [newCategoryName, setNewCategoryName] = useState("");
  const [categoryError, setCategoryError] = useState("");
  const [activeSection, setActiveSection] = useState<FormSection>("basics");
  const [fieldErrors, setFieldErrors] = useState<Partial<Record<keyof ProductFormState, string>>>({});
  const [slugTouched, setSlugTouched] = useState(false);

  const [imageFiles, setImageFiles] = useState<File[]>([]);
  const [imagePreviews, setImagePreviews] = useState<string[]>([]);
  /** When editing, images already linked to the product (from list row). */
  const [existingProductImages, setExistingProductImages] = useState<ProductImageListItem[]>([]);
  const [imageError, setImageError] = useState("");
  const [imageMessage, setImageMessage] = useState("");
  const [imageDialogOpen, setImageDialogOpen] = useState(false);
  /** When editing, image IDs that were present when we opened edit (so we can delete removed ones on Update). */
  const [initialExistingImageIdsWhenEdit, setInitialExistingImageIdsWhenEdit] = useState<string[]>([]);
  const [initialVariantIdsWhenEdit, setInitialVariantIdsWhenEdit] = useState<string[]>([]);
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  /** True while uploads/refetch run after Update product (loading overlay shown). */
  const [isUpdateReflecting, setIsUpdateReflecting] = useState(false);
  /** Reorder images dialog: combined list (existing + new) for drag reorder while editing. */
  const [reorderImagesOpen, setReorderImagesOpen] = useState(false);
  const [reorderableImages, setReorderableImages] = useState<AdminReorderableImage[]>([]);
  const [reorderDragIndex, setReorderDragIndex] = useState<number | null>(null);
  /** When editing, Review images dialog shows all images (existing + new); this is the combined list for that dialog. */
  const [reviewImagesList, setReviewImagesList] = useState<AdminReorderableImage[]>([]);
  const [reviewDragIndex, setReviewDragIndex] = useState<number | null>(null);
  /** When set, this is the confirmed order (existing + new interleaved). Used for display and sync so order matches Review. */
  const [orderedProductImages, setOrderedProductImages] = useState<AdminReorderableImage[] | null>(
    null
  );
  /** Key set when loading product for edit so image URLs get a fresh cache-buster and browser refetches. */
  const [productImagesLoadKey, setProductImagesLoadKey] = useState<string>("");
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  // Load draft from sessionStorage on first mount (product fields only; variants/moods not persisted)
  useEffect(() => {
    if (typeof window === "undefined") return;
    try {
      const raw = window.sessionStorage.getItem(DRAFT_KEY);
      if (!raw) return;
      const parsed = JSON.parse(raw) as Partial<ProductFormState>;
      setForm((prev) => ({
        ...prev,
        ...parsed,
        hasBlousePiece: parsed.hasBlousePiece ?? prev.hasBlousePiece,
      }));
    } catch {
      // ignore malformed drafts
    }
  }, []);

  useEffect(() => {
    if (categories.length > 0 && !form.categoryId) {
      setForm((prev) => ({ ...prev, categoryId: categories[0].categoryId }));
    }
  }, [categories, form.categoryId]);

  const createCategoryMutation = useMutation({
    mutationFn: async (name: string) => {
      const data = await gqlAdmin<{ createCategory?: Array<{ categoryId: string; name: string }> }>(
        `mutation CreateCategory($category: NewCategory!) {
          createCategory(category: $category) { categoryId name }
        }`,
        { category: { name: name.trim() } }
      );
      return data?.createCategory?.[0];
    },
    onSuccess: (created) => {
      if (created) {
        queryClient.invalidateQueries({ queryKey: ["admin", "categories"] });
        setForm((prev) => ({ ...prev, categoryId: created.categoryId }));
        setNewCategoryName("");
        setShowNewCategory(false);
        setCategoryError("");
      }
    },
    onError: (err: Error) => setCategoryError(err.message || "Failed to create category."),
  });

  const createProductMutation = useMutation({
    mutationFn: async (payload: {
      name: string;
      description: string;
      pricePaise: number;
      stockQuantity: string;
      categoryId: string;
      sku?: string;
      slug?: string;
      fabric?: string;
      weave?: string;
      occasion?: string;
      hasBlousePiece?: boolean;
      careInstructions?: string;
      productStatusId?: string;
    }) => {
      const product: Record<string, unknown> = {
        name: payload.name,
        description: payload.description,
        pricePaise: String(payload.pricePaise),
        stockQuantity: payload.stockQuantity,
        categoryId: payload.categoryId,
      };
      if (payload.sku?.trim()) product.sku = payload.sku.trim();
      if (payload.slug?.trim()) product.slug = payload.slug.trim();
      if (payload.fabric?.trim()) product.fabric = payload.fabric.trim();
      if (payload.weave?.trim()) product.weave = payload.weave.trim();
      if (payload.occasion?.trim()) product.occasion = payload.occasion.trim();
      if (payload.hasBlousePiece !== undefined) product.hasBlousePiece = payload.hasBlousePiece;
      if (payload.careInstructions?.trim()) product.careInstructions = payload.careInstructions.trim();
      if (payload.productStatusId?.trim()) product.productStatusId = payload.productStatusId.trim();
      const data = await gqlAdmin<{ createProduct?: Array<{ productId: string; name: string; formatted?: string }> }>(
        `mutation CreateProduct($product: NewProduct!) {
          createProduct(product: $product) { productId name formatted }
        }`,
        { product }
      );
      return data?.createProduct?.[0];
    },
    onSuccess: async (created) => {
      const text =
        created && created.name
          ? `Created: ${created.name}${created.formatted ? ` (${created.formatted})` : ""}`
          : "Product created.";
      setMessage(text);
      showToast({
        title: "Product",
        description: "Product created.",
      });
      if (created?.productId) {
        const failures: string[] = [];

        // Upload images — awaited (via mutateAsync, not fire-and-forget mutate) so the grid
        // refetch below actually happens after they land, and so a failure here is caught
        // instead of vanishing silently.
        for (let index = 0; index < imageFiles.length; index += 1) {
          try {
            await uploadImageMutation.mutateAsync({
              file: imageFiles[index],
              productId: created.productId,
              order: index,
            });
          } catch (err) {
            console.error("Failed to upload image:", err);
            failures.push(`image "${imageFiles[index].name}"`);
          }
        }
        // Create variants and inventory
        for (const v of variants) {
          try {
            const sizeId = v.sizeId?.trim() || undefined;
            const colorId = v.colorId?.trim() || undefined;
            const additionalPricePaise = v.additionalPricePaise?.trim() || undefined;
            const variant = await createProductVariant({
              productId: created.productId,
              sizeId: sizeId || undefined,
              colorId: colorId || undefined,
              additionalPricePaise,
            });
            if (variant?.variantId) {
              await createInventoryItem({
                variantId: variant.variantId,
                quantityAvailable: (v.quantityAvailable?.trim() || "0").replace(/^$/, "0"),
                reorderLevel: v.reorderLevel?.trim() || undefined,
              });
            }
          } catch (err) {
            console.error("Failed to create variant/inventory:", err);
            failures.push("a size/stock variant");
          }
        }
        // Link moods
        for (const moodId of selectedMoodIds) {
          if (!moodId?.trim()) continue;
          try {
            await createProductMoodMapping(created.productId, moodId.trim());
          } catch (err) {
            console.error("Failed to link mood:", err);
            failures.push("a mood tag");
          }
        }
        setVariants([]);
        setSelectedMoodIds([]);

        if (failures.length > 0) {
          showToast({
            title: "Product created, but incomplete",
            description: `"${created.name}" was created, but failed to save: ${failures.join(", ")}. Edit the product to retry.`,
          });
        }

        // The product row above was created before its images/variants existed, so refetch now
        // that they're in — otherwise the grid keeps showing the bare row (no image, "Stock: -")
        // until something else happens to trigger a refetch.
        await queryClient.refetchQueries({ queryKey: ["admin", "products"] });
      }
      setForm((prev) => ({
        ...prev,
        name: "",
        description: "",
        priceRupees: "",
        stockQuantity: "0",
        categoryId: prev.categoryId,
        sku: "",
        slug: "",
        fabric: "",
        weave: "",
        occasion: "",
        hasBlousePiece: true,
        careInstructions: "",
        productStatusId: "",
      }));
      setError("");
      if (typeof window !== "undefined") {
        window.sessionStorage.removeItem(DRAFT_KEY);
      }
    },
    onError: (err: Error) => {
      setError(err.message || "Failed to create product.");
      showToast({
        title: "Product",
        description: "Failed to create product.",
      });
    },
  });

  const updateProductMutation = useMutation({
    mutationFn: async (payload: {
      productId: string;
      name: string;
      description: string;
      pricePaise: number;
      categoryId: string;
      sku?: string;
      slug?: string;
      fabric?: string;
      weave?: string;
      occasion?: string;
      hasBlousePiece?: boolean;
      careInstructions?: string;
      productStatusId?: string;
      selectedMoodIds?: string[];
      variants?: AdminProductVariantRow[];
      initialVariantIdsWhenEdit?: string[];
      /** Current existing images (after user may have removed some); used to compute which to delete. */
      currentExistingImages?: ProductImageListItem[];
      /** Image IDs when edit was opened; images not in currentExistingImages will be deleted. */
      initialExistingImageIdsWhenEdit?: string[];
    }) => {
      const product: Record<string, unknown> = {
        productId: payload.productId,
        name: payload.name,
        description: payload.description,
        pricePaise: String(payload.pricePaise),
        categoryId: payload.categoryId,
      };
      if (payload.sku?.trim()) product.sku = payload.sku.trim();
      if (payload.slug?.trim()) product.slug = payload.slug.trim();
      if (payload.fabric?.trim()) product.fabric = payload.fabric.trim();
      if (payload.weave?.trim()) product.weave = payload.weave.trim();
      if (payload.occasion?.trim()) product.occasion = payload.occasion.trim();
      if (payload.hasBlousePiece !== undefined) product.hasBlousePiece = payload.hasBlousePiece;
      if (payload.careInstructions?.trim()) product.careInstructions = payload.careInstructions.trim();
      if (payload.productStatusId?.trim()) product.productStatusId = payload.productStatusId.trim();

      const data = await gqlAdmin<{ updateProduct?: Array<{ productId: string; name: string; formatted?: string }> }>(
        `mutation UpdateProduct($product: ProductMutation!) {
          updateProduct(product: $product) { productId name formatted }
        }`,
        { product }
      );
      const updated = data?.updateProduct?.[0];
      if (!updated?.productId) return updated ?? null;

      // Sync mood mappings to match selectedMoodIds
      const selected = new Set((payload.selectedMoodIds ?? []).map((id) => id?.trim()).filter(Boolean));
      const current = await searchProductMoodMappingsByProduct(updated.productId);
      const currentMoodIds = new Set(current.map((m) => m.moodId));

      for (const moodId of selected) {
        if (!currentMoodIds.has(moodId)) {
          try {
            await createProductMoodMapping(updated.productId, moodId);
          } catch (err) {
            console.error("Failed to add mood mapping:", err);
          }
        }
      }
      for (const m of current) {
        if (!selected.has(m.moodId)) {
          try {
            await deleteProductMoodMapping(updated.productId, m.moodId);
          } catch (err) {
            console.error("Failed to remove mood mapping:", err);
          }
        }
      }

      // Sync variants + inventory (persist stock edits and new variants on update)
      const incomingVariants = payload.variants ?? [];
      const keptVariantIds = new Set<string>();
      for (const v of incomingVariants) {
        try {
          const sizeId = v.sizeId?.trim() || undefined;
          const colorId = v.colorId?.trim() || undefined;
          const additionalPricePaise = v.additionalPricePaise?.trim() || undefined;
          const quantityAvailable = (v.quantityAvailable?.trim() || "0").replace(/^$/, "0");
          const reorderLevel = v.reorderLevel?.trim() || undefined;

          let variantId = v.variantId?.trim() || "";
          if (!variantId) {
            const createdVariant = await createProductVariant({
              productId: updated.productId,
              sizeId,
              colorId,
              additionalPricePaise,
            });
            variantId = createdVariant?.variantId ?? "";
          } else {
            await updateProductVariant({
              variantId,
              productId: updated.productId,
              sizeId,
              colorId,
              additionalPricePaise,
            });
          }
          if (!variantId) continue;
          keptVariantIds.add(variantId);

          const inventoryRows = await searchInventoryByVariantId(variantId);
          if (inventoryRows.length > 0) {
            await updateInventoryItem({
              inventoryId: inventoryRows[0].inventoryId,
              quantityAvailable,
              reorderLevel,
            });
          } else {
            await createInventoryItem({
              variantId,
              quantityAvailable,
              reorderLevel,
            });
          }
        } catch (err) {
          console.error("Failed to sync variant/inventory:", err);
        }
      }
      // Delete variants removed from the edit form
      for (const oldVariantId of payload.initialVariantIdsWhenEdit ?? []) {
        if (!keptVariantIds.has(oldVariantId)) {
          try {
            await deleteProductVariant(oldVariantId);
          } catch (err) {
            console.error("Failed to delete removed variant:", err);
          }
        }
      }

      return updated;
    },
    onSuccess: async (updated) => {
      try {
        const successText = updated
          ? `Updated: ${updated.name}${updated.formatted ? ` (${updated.formatted})` : ""}`
          : "Product updated.";
        setMessage(successText);
        showToast({ title: "Product updated", description: successText });
        // Sync product images: update order for kept, bulk insert new, delete removed (1 row per image)
        setIsUpdateReflecting(true);
        const productId = updated?.productId;
        const hasExisting = existingProductImages.length > 0;
        const hasNew = imageFiles.length > 0;
        const hasOrdered = orderedProductImages != null && orderedProductImages.length > 0;
        if (productId && (hasOrdered || hasExisting || hasNew)) {
          try {
            let items: Array<{ imageId?: string; key?: string; url?: string }>;
            if (hasOrdered && orderedProductImages) {
              // Preserve confirmed order: upload new in order, then build items in same order
              const newKeys: string[] = [];
              for (let i = 0; i < orderedProductImages.length; i++) {
                const item = orderedProductImages[i];
                if (item.type !== "new") continue;
                const file = item.file;
                try {
                  const presigned = await gqlAdmin<{
                    getPresignedUploadUrl?: Array<{ uploadUrl: string; key: string }>;
                  }>(
                    `query GetPresignedUploadUrl($input: GetPresignedUploadUrl!) {
                      getPresignedUploadUrl(input: $input) { uploadUrl key }
                    }`,
                    {
                      input: {
                        productId,
                        filename: file.name,
                        contentType: file.type || "application/octet-stream",
                        displayOrder: i,
                      },
                    }
                  );
                  const info = presigned.getPresignedUploadUrl?.[0];
                  if (!info) throw new Error("Did not receive upload URL.");
                  await fetch(info.uploadUrl, {
                    method: "PUT",
                    headers: { "Content-Type": file.type || "application/octet-stream" },
                    body: file,
                  });
                  newKeys.push(info.key);
                } catch (uploadErr) {
                  const reason =
                    uploadErr instanceof Error ? uploadErr.message : "upload failed";
                  throw new Error(
                    `Image "${file.name}" (${newKeys.length + 1} of the new images) failed to upload: ${reason}. Product images were not changed — remove or replace this file and try again.`
                  );
                }
              }
              let newIndex = 0;
              items = orderedProductImages.map((item) =>
                item.type === "existing"
                  ? { imageId: item.image.imageId ?? undefined, key: undefined, url: undefined }
                  : { imageId: undefined, key: newKeys[newIndex++], url: undefined }
              );
            } else {
              // No confirmed order: existing first, then new
              const newKeys: string[] = [];
              for (let i = 0; i < imageFiles.length; i++) {
                const file = imageFiles[i];
                try {
                  const presigned = await gqlAdmin<{
                    getPresignedUploadUrl?: Array<{ uploadUrl: string; key: string }>;
                  }>(
                    `query GetPresignedUploadUrl($input: GetPresignedUploadUrl!) {
                      getPresignedUploadUrl(input: $input) { uploadUrl key }
                    }`,
                    {
                      input: {
                        productId,
                        filename: file.name,
                        contentType: file.type || "application/octet-stream",
                        displayOrder: i,
                      },
                    }
                  );
                  const info = presigned.getPresignedUploadUrl?.[0];
                  if (!info) throw new Error("Did not receive upload URL.");
                  await fetch(info.uploadUrl, {
                    method: "PUT",
                    headers: { "Content-Type": file.type || "application/octet-stream" },
                    body: file,
                  });
                  newKeys.push(info.key);
                } catch (uploadErr) {
                  const reason =
                    uploadErr instanceof Error ? uploadErr.message : "upload failed";
                  throw new Error(
                    `Image "${file.name}" (${newKeys.length + 1} of ${imageFiles.length} new images) failed to upload: ${reason}. Product images were not changed — remove or replace this file and try again.`
                  );
                }
              }
              const existingItems = existingProductImages.map((im) => ({
                imageId: im.imageId ?? undefined,
                key: undefined,
                url: undefined,
              }));
              const newItems = newKeys.map((key) => ({
                imageId: undefined,
                key,
                url: undefined,
              }));
              items = [...existingItems, ...newItems];
            }
            const syncResult = await gqlAdmin<{
              syncProductImages?: Array<{ imageId: string; productId: string; url?: string | null }>;
            }>(
              `mutation SyncProductImages($input: SyncProductImagesInput!) {
                syncProductImages(input: $input) { imageId productId url }
              }`,
              {
                input: {
                  productId,
                  items,
                },
              }
            );
            const updatedImages = syncResult.syncProductImages ?? [];
            const mappedImages: ProductImageListItem[] = updatedImages.map((im) => ({
              imageId: im.imageId,
              thumbnailUrl: im.url ?? null,
              url: im.url ?? null,
            }));
            if (productId) {
              setSelectedProduct((prev) =>
                prev?.productId === productId
                  ? { ...prev, images: mappedImages }
                  : prev
              );
              queryClient.setQueryData<ProductListRow[]>(
                ["admin", "products", appliedSearch],
                (old) => {
                  if (!old) return old;
                  return old.map((prod) =>
                    prod.productId === productId ? { ...prod, images: mappedImages } : prod
                  );
                }
              );
            }
          } catch (err) {
            setImageError(err instanceof Error ? err.message : "Image update failed.");
          }
        }
        await queryClient.refetchQueries({ queryKey: ["admin", "products"] });
        if (productId) {
          const list = queryClient.getQueryData<ProductListRow[]>([
            "admin",
            "products",
            appliedSearch,
          ]);
          const fresh = list?.find((p) => p.productId === productId);
          if (fresh) setSelectedProduct(fresh);
        }
        setEditingProductId(null);
        setImageFiles([]);
        setExistingProductImages([]);
        setOrderedProductImages(null);
        setInitialExistingImageIdsWhenEdit([]);
        setInitialVariantIdsWhenEdit([]);
      } finally {
        setIsUpdateReflecting(false);
      }
    },
    onError: (err: Error) => setError(err.message || "Failed to update product."),
  });

  const handleChange = (
    e: React.ChangeEvent<
      HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement
    >
  ) => {
    const { name, value, type } = e.target;
    const checked = (e.target as HTMLInputElement).checked;
    if (name === "slug") setSlugTouched(true);
    setForm((prev) => {
      const next = { ...prev, [name]: type === "checkbox" ? checked : value };
      if (name === "name" && !slugTouched) {
        next.slug = slugify(value);
      }
      if (typeof window !== "undefined") {
        window.sessionStorage.setItem(DRAFT_KEY, JSON.stringify(next));
      }
      return next;
    });
    setFieldErrors((prev) => {
      if (!prev[name as keyof ProductFormState]) return prev;
      const next = { ...prev };
      delete next[name as keyof ProductFormState];
      return next;
    });
    setError("");
    setMessage("");
  };

  const handleAddCategory = (e: React.FormEvent) => {
    e.preventDefault();
    const name = newCategoryName.trim();
    if (!name) {
      setCategoryError("Category name is required.");
      return;
    }
    setCategoryError("");
    createCategoryMutation.mutate(name);
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setMessage("");
    setImageError("");
    setImageMessage("");
    setFieldErrors({});

    if (!editingProductId && imageFiles.length === 0) {
      setActiveSection("photos");
      setImageError("At least one product image is required.");
      return;
    }
    const parsed = adminProductFormSchema.safeParse(form);
    if (!parsed.success) {
      const flat = parsed.error.flatten().fieldErrors;
      const nextFieldErrors: Partial<Record<keyof ProductFormState, string>> = {};
      let firstSection: FormSection | null = null;
      for (const [key, messages] of Object.entries(flat)) {
        const msg = messages?.[0];
        if (!msg) continue;
        const field = key as keyof ProductFormState;
        nextFieldErrors[field] = msg;
        if (!firstSection) firstSection = SECTION_FOR_FIELD[field] ?? "basics";
      }
      setFieldErrors(nextFieldErrors);
      setActiveSection(firstSection ?? "basics");
      return;
    }
    const { name, description, priceRupees, categoryId, sku, slug, fabric, weave, occasion, hasBlousePiece, careInstructions, productStatusId } =
      parsed.data;
    const pricePaise = rupeesInputToPaise(priceRupees || "0");
    if (pricePaise <= 0) {
      setActiveSection("basics");
      setError("Price must be greater than 0.");
      return;
    }
    if (pricePaise > MAX_MONEY_PAISE) {
      setActiveSection("basics");
      setError("Price exceeds supported maximum.");
      return;
    }
    if (variants.length === 0) {
      setActiveSection("stock");
      setError("Add at least one variant (size) with stock.");
      return;
    }
    const invalidVariant = variants.find(
      (v) =>
        !v.sizeId ||
        v.quantityAvailable.trim() === "" ||
        Number.isNaN(Number(v.quantityAvailable)) ||
        Number(v.quantityAvailable) < 0
    );
    if (invalidVariant && variants.length > 0) {
      setActiveSection("stock");
      setError("Each variant must have a size and non-negative stock quantity.");
      return;
    }
    if (editingProductId) {
      updateProductMutation.mutate({
        productId: editingProductId,
        name,
        description: description || "",
        pricePaise,
        categoryId,
        sku: sku || undefined,
        slug: slug || undefined,
        fabric: fabric || undefined,
        weave: weave || undefined,
        occasion: occasion || undefined,
        hasBlousePiece,
        careInstructions: careInstructions || undefined,
        productStatusId: productStatusId || undefined,
        selectedMoodIds,
        variants,
        initialVariantIdsWhenEdit,
        currentExistingImages: existingProductImages,
        initialExistingImageIdsWhenEdit,
      });
    } else {
      createProductMutation.mutate({
        name,
        description: description || "",
        pricePaise,
        stockQuantity: form.stockQuantity ?? "0",
        categoryId,
        sku: sku || undefined,
        slug: slug || undefined,
        fabric: fabric || undefined,
        weave: weave || undefined,
        occasion: occasion || undefined,
        hasBlousePiece,
        careInstructions: careInstructions || undefined,
        productStatusId: productStatusId || undefined,
      });
    }
  };

  // Auto-clear success message after a short delay
  useEffect(() => {
    if (!message) return;
    const t = setTimeout(() => setMessage(""), 4000);
    return () => clearTimeout(t);
  }, [message]);

  useEffect(() => {
    if (!imageMessage) return;
    const t = setTimeout(() => setImageMessage(""), 4000);
    return () => clearTimeout(t);
  }, [imageMessage]);

  // Build and clean up object URLs for image previews
  useEffect(() => {
    if (imageFiles.length === 0) {
      setImagePreviews([]);
      return;
    }
    const urls = imageFiles.map((file) => URL.createObjectURL(file));
    setImagePreviews(urls);
    return () => {
      urls.forEach((url) => URL.revokeObjectURL(url));
    };
  }, [imageFiles]);

  // When Review images dialog is open and editing, show all images (preserve confirmed order when set)
  useEffect(() => {
    if (!imageDialogOpen || !editingProductId) return;
    if (orderedProductImages != null && orderedProductImages.length > 0) {
      setReviewImagesList(orderedProductImages);
    } else {
      const existing = existingProductImages.map((image) => ({ type: "existing" as const, image }));
      const newItems = imagePreviews.map((url, i) => ({
        type: "new" as const,
        file: imageFiles[i],
        previewUrl: url,
      }));
      setReviewImagesList([...existing, ...newItems]);
    }
  }, [imageDialogOpen, editingProductId, existingProductImages, imagePreviews, imageFiles, orderedProductImages]);

  const categoriesErrorUi =
    categoriesError && categoriesErrorObj
      ? toRouteFailureUi("admin", categoriesErrorObj)
      : null;
  const productsErrorUi =
    productsError && productsErrorObj
      ? toRouteFailureUi("admin", productsErrorObj)
      : null;
  const categoryNameById = Object.fromEntries(categories.map((c) => [c.categoryId, c.name]));

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const next: {
      name?: string;
      categoryId?: string;
      moodId?: string;
      fabric?: string;
      weave?: string;
      occasion?: string;
      limit?: string;
      productStatusId?: string;
      startingPricePaise?: string;
      endingPricePaise?: string;
    } = {};
    const trimmedName = searchName.trim();
    const trimmedLimit = searchLimit.trim();
    if (trimmedName) next.name = trimmedName;
    if (searchCategoryId) next.categoryId = searchCategoryId;
    if (searchMoodId) next.moodId = searchMoodId;
    if (searchFabric) next.fabric = searchFabric;
    if (searchWeave) next.weave = searchWeave;
    if (searchOccasion) next.occasion = searchOccasion;
    if (searchProductStatusId) next.productStatusId = searchProductStatusId;
    const minPaise = optionalRupeesInputToPaise(searchPriceMinRupees);
    const maxPaise = optionalRupeesInputToPaise(searchPriceMaxRupees);
    if (typeof minPaise === "number" && minPaise >= 0) {
      next.startingPricePaise = String(Math.min(minPaise, MAX_MONEY_PAISE));
    }
    if (typeof maxPaise === "number" && maxPaise >= 0) {
      next.endingPricePaise = String(Math.min(maxPaise, MAX_MONEY_PAISE));
    }
    if (trimmedLimit) next.limit = trimmedLimit;
    setAppliedSearch(next);
  };

  const handleSearchClear = () => {
    setSearchName("");
    setSearchCategoryId("");
    setSearchMoodId("");
    setSearchFabric("");
    setSearchWeave("");
    setSearchOccasion("");
    setSearchProductStatusId("");
    setSearchPriceMinRupees("");
    setSearchPriceMaxRupees("");
    setSearchLimit("20");
    setAppliedSearch({ limit: "20" });
  };

  const handleArchiveConfirm = () => {
    if (!archiveConfirm) return;
    const pricePaise = parseInt(archiveConfirm.amountPaise ?? "0", 10) || 0;
    updateProductMutation.mutate(
      {
        productId: archiveConfirm.productId,
        name: archiveConfirm.name,
        description: archiveConfirm.description ?? "",
        pricePaise,
        categoryId: archiveConfirm.categoryId ?? "",
        sku: archiveConfirm.sku ?? undefined,
        slug: archiveConfirm.slug ?? undefined,
        fabric: archiveConfirm.fabric ?? undefined,
        weave: archiveConfirm.weave ?? undefined,
        occasion: archiveConfirm.occasion ?? undefined,
        hasBlousePiece: archiveConfirm.hasBlousePiece ?? undefined,
        careInstructions: archiveConfirm.careInstructions ?? undefined,
        productStatusId: "3",
      },
      {
        onSettled: () => setArchiveConfirm(null),
        onSuccess: () => setMessage(`${archiveConfirm.name} archived.`),
      }
    );
  };

  const loadProductEditData = async (productId: string): Promise<string[]> => {
    try {
      const mappings = await searchProductMoodMappingsByProduct(productId);
      return mappings.map((m) => m.moodId);
    } catch (err) {
      console.error("Failed to load product moods for edit:", err);
      return [];
    }
  };

  const beginEditProduct = async (p: ProductListRow) => {
    setActiveTab("add");
    setEditingProductId(p.productId);
    setError("");
    setFieldErrors({});
    setActiveSection("basics");
    setSlugTouched(true);
    setMessage(`Loading product…`);
    let product: ProductListRow = p;
    let moodIds: string[] = [];
    try {
      const [fresh, loadedMoodIds] = await Promise.all([
        fetchProductById(p.productId),
        loadProductEditData(p.productId),
      ]);
      if (fresh) product = fresh;
      moodIds = loadedMoodIds;
    } catch (err) {
      console.error("Failed to load product for edit:", err);
      setError("Failed to load product. Using list data.");
    }
    setForm((prev) => ({
      ...prev,
      name: product.name ?? "",
      description: product.description ?? "",
      priceRupees: paiseToRupeesInput(product.amountPaise),
      stockQuantity: product.stockQuantity ?? "",
      categoryId: product.categoryId ?? prev.categoryId ?? "",
      sku: product.sku ?? "",
      slug: product.slug ?? "",
      fabric: product.fabric ?? "",
      weave: product.weave ?? "",
      occasion: product.occasion ?? "",
      hasBlousePiece: product.hasBlousePiece ?? true,
      careInstructions: product.careInstructions ?? "",
      productStatusId: product.productStatusId ?? "",
    }));
    const variantRows = (product as ProductListRowWithVariantStock).variantStock ?? [];
    setVariants(
      variantRows.map((v) => ({
        variantId: v.variantId,
        sizeId: v.sizeId ?? "",
        colorId: undefined,
        additionalPricePaise: "0",
        quantityAvailable: String(v.quantity ?? 0),
        reorderLevel: undefined,
      }))
    );
    setInitialVariantIdsWhenEdit(
      variantRows
        .map((v) => v.variantId)
        .filter((id): id is string => !!id)
    );
    setSelectedMoodIds(moodIds);
    setImageFiles([]);
    setImageError("");
    setImageMessage("");
    const images = product.images ?? [];
    setExistingProductImages(images);
    setOrderedProductImages(null);
    setProductImagesLoadKey(String(Date.now()));
    setInitialExistingImageIdsWhenEdit(
      images
        .map(
          (im) =>
            (im as ProductImageListItem & { image_id?: string }).imageId ??
            (im as ProductImageListItem & { image_id?: string }).image_id
        )
        .filter((id): id is string => !!id)
    );
    setMessage(`Editing product: ${product.name}`);
  };

  const uploadImageMutation = useMutation({
    mutationFn: async ({
      file,
      productId,
      order,
    }: {
      file: File;
      productId: string;
      order: number;
    }) => {
      const presigned = await gqlAdmin<{
        getPresignedUploadUrl?: Array<{
          uploadUrl: string;
          key: string;
          cdnUrl: string;
        }>;
      }>(
        `query GetPresignedUploadUrl($input: GetPresignedUploadUrl!) {
          getPresignedUploadUrl(input: $input) {
            uploadUrl
            key
            cdnUrl
          }
        }`,
        {
          input: {
            productId,
            filename: file.name,
            contentType: file.type || "application/octet-stream",
            displayOrder: order,
          },
        }
      );

      const info = presigned.getPresignedUploadUrl?.[0];
      if (!info) {
        throw new Error("Did not receive upload URL from backend.");
      }

      await fetch(info.uploadUrl, {
        method: "PUT",
        headers: {
          "Content-Type": file.type || "application/octet-stream",
        },
        body: file,
      });

      const confirmed = await gqlAdmin<{
        confirmImageUpload?: Array<{
          imageId: string;
          url?: string | null;
          thumbnailUrl?: string | null;
        }>;
      }>(
        `mutation ConfirmImageUpload($input: ConfirmImageUpload!) {
          confirmImageUpload(input: $input) {
            imageId
            url
            thumbnailUrl
          }
        }`,
        {
          input: {
            productId,
            key: info.key,
            altText: null,
            displayOrder: order,
          },
        }
      );

      return confirmed.confirmImageUpload?.[0];
    },
    onSuccess: (img) => {
      setImageFiles([]);
      setImageError("");
      setImageMessage(
        img?.imageId
          ? "Image uploaded and linked to product."
          : "Image upload confirmed."
      );
    },
    onError: (err: Error) => {
      setImageError(err.message || "Failed to upload image.");
      setImageMessage("");
    },
  });

  const totalPhotos = existingProductImages.length + imageFiles.length;
  const FORM_SECTIONS: { id: FormSection; label: string; fields: (keyof ProductFormState)[]; completion?: string }[] = [
    { id: "basics", label: "Basics", fields: ["name", "description", "priceRupees", "categoryId"] },
    { id: "details", label: "Details", fields: ["sku", "slug", "fabric", "weave", "occasion", "careInstructions", "productStatusId"] },
    { id: "stock", label: "Sizes & stock", fields: [], completion: variants.length > 0 ? `(${variants.length})` : undefined },
    { id: "moods", label: "Moods", fields: [], completion: selectedMoodIds.length > 0 ? `(${selectedMoodIds.length})` : undefined },
    { id: "photos", label: "Photos", fields: [], completion: totalPhotos > 0 ? `(${totalPhotos})` : undefined },
  ];

  return (
    <AdminPageShell
      label="Products"
      title="Product catalog"
      description="View, search, and add products. Filter by category, status, and price."
      action={
        activeTab === "view" ? (
          <span className="inline-flex items-center gap-2 rounded-full border border-[var(--color-line)] bg-[var(--color-surface-soft)] px-4 py-2 text-sm font-semibold text-[var(--color-ink)]">
            <Package className="h-4 w-4" />
            {products.length} products
          </span>
        ) : null
      }
    >
      <div className="inline-flex rounded-xl border border-[var(--color-line)] bg-[var(--color-surface)] p-1.5 shadow-[var(--shadow-subtle)]">
        <button
          type="button"
          onClick={() => setActiveTab("view")}
          className={cn(
            "rounded-lg px-5 py-2.5 text-[15px] font-semibold transition-colors",
            activeTab === "view"
              ? "bg-[var(--color-green)] text-white"
              : "text-[var(--color-muted)] hover:bg-[var(--color-surface-soft)]"
          )}
        >
          View products
        </button>
        <button
          type="button"
          onClick={() => {
            setActiveTab("add");
            setEditingProductId(null);
            setExistingProductImages([]);
            setOrderedProductImages(null);
            setInitialExistingImageIdsWhenEdit([]);
            setInitialVariantIdsWhenEdit([]);
            setFieldErrors({});
            setActiveSection("basics");
            setSlugTouched(false);
          }}
          className={cn(
            "rounded-lg px-5 py-2.5 text-[15px] font-semibold transition-colors",
            activeTab === "add"
              ? "bg-[var(--color-green)] text-white"
              : "text-[var(--color-muted)] hover:bg-[var(--color-surface-soft)]"
          )}
        >
          Add product
        </button>
      </div>

      {activeTab === "view" && (
        <>
          <ProductsFiltersCard
            filters={{
              searchName,
              searchCategoryId,
              searchProductStatusId,
              searchMoodId,
              searchPriceMinRupees,
              searchPriceMaxRupees,
              searchLimit,
              searchFabric,
              searchWeave,
              searchOccasion,
            }}
            categories={categories.map((c) => ({ id: c.categoryId, name: c.name }))}
            moods={existingMoods.map((m) => ({ id: m.moodId, name: m.moodName }))}
            fabrics={fabrics.map((f) => ({ id: f.fabricId, name: f.fabricName }))}
            weaves={weaves.map((w) => ({ id: w.weaveId, name: w.weaveName }))}
            occasions={occasions.map((o) => ({ id: o.occasionId, name: o.occasionName }))}
            onFiltersChange={(next) => {
              setSearchName(next.searchName);
              setSearchCategoryId(next.searchCategoryId);
              setSearchProductStatusId(next.searchProductStatusId);
              setSearchMoodId(next.searchMoodId);
              setSearchPriceMinRupees(next.searchPriceMinRupees);
              setSearchPriceMaxRupees(next.searchPriceMaxRupees);
              setSearchLimit(next.searchLimit);
              setSearchFabric(next.searchFabric);
              setSearchWeave(next.searchWeave);
              setSearchOccasion(next.searchOccasion);
            }}
            onApply={handleSearchSubmit}
            onClear={handleSearchClear}
            onRefresh={() => {
              void refetchProducts();
            }}
          />
          <ProductsGridCard
            products={products}
            productsLoading={productsLoading}
            productsError={productsError}
            productsErrorUi={productsErrorUi}
            categoryNameById={categoryNameById}
            getThumbnail={getProductThumbnailWithCacheBuster}
            onRetry={() => {
              void refetchProducts();
            }}
            onOpenProduct={setSelectedProduct}
            onEditProduct={beginEditProduct}
            onArchiveProduct={setArchiveConfirm}
          />

          <ArchiveProductDialog
            product={archiveConfirm}
            isPending={updateProductMutation.isPending}
            onClose={() => setArchiveConfirm(null)}
            onConfirm={handleArchiveConfirm}
          />

          <ProductPreviewDialog
            key={selectedProduct?.productId ?? "preview-none"}
            product={selectedProduct}
            open={!!selectedProduct}
            onClose={() => setSelectedProduct(null)}
            categoryNameById={categoryNameById}
            getProductStatusLabel={getProductStatusLabel}
          />
        </>
      )}

      {activeTab === "add" && (
        <Card className="mt-6 rounded-2xl border-[var(--color-line)] bg-[var(--admin-surface-muted)] shadow-[var(--admin-card-shadow)]">
          <CardTitle className="flex items-center gap-2.5 text-sm font-semibold normal-case tracking-normal text-[var(--color-ink)] md:text-[15px]">
            {editingProductId ? (
              <Pencil className="h-4 w-4 text-emerald-600" />
            ) : (
              <Plus className="h-4 w-4 text-emerald-600" />
            )}
            {editingProductId ? "Edit product" : "Add new product"}
          </CardTitle>
          <CardContent className="mt-6 relative">
            {isUpdateReflecting && (
              <div
                className="absolute inset-0 z-10 flex flex-col items-center justify-center gap-3 rounded-xl bg-[var(--color-ivory)]/95"
                aria-live="polite"
                aria-busy="true"
              >
                <Spinner />
                <p className="text-sm font-medium text-[var(--color-ink)]">
                  Updating images &amp; refreshing…
                </p>
              </div>
            )}
            {categoriesError && (
              <div className="mb-4 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800">
                <p className="font-medium">Can&apos;t load categories.</p>
                <p className="mt-1 text-xs text-red-800/80">
                  {categoriesErrorUi?.message ?? "Please try again."}
                </p>
                <button
                  type="button"
                  onClick={() => refetchCategories()}
                  className="mt-2 inline-flex items-center rounded-md border border-red-300 bg-white px-3 py-1 text-xs font-medium text-red-800 hover:bg-red-50"
                >
                  Try again
                </button>
              </div>
            )}
            {error && (
              <div
                id="admin-product-form-error"
                className="mb-4 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800"
                role="alert"
              >
                {error}
              </div>
            )}
            {message && (
              <div
                className="mb-4 rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-600"
                role="status"
              >
                {message}
              </div>
            )}
            <form onSubmit={handleSubmit}>
              <div className="lg:grid lg:grid-cols-[minmax(0,1fr)_320px] lg:items-start lg:gap-8">
                <div className="min-w-0 space-y-4">
                  <div
                    role="tablist"
                    aria-label="Product form sections"
                    className="flex flex-wrap gap-2 rounded-xl border border-[var(--color-line)] bg-[var(--color-surface)] p-1.5 shadow-[var(--shadow-subtle)]"
                  >
                    {FORM_SECTIONS.map((s) => {
                      const isActive = activeSection === s.id;
                      const hasError = s.fields.some((f) => Boolean(fieldErrors[f]));
                      return (
                        <button
                          key={s.id}
                          type="button"
                          role="tab"
                          aria-selected={isActive}
                          onClick={() => setActiveSection(s.id)}
                          className={cn(
                            "flex items-center gap-1.5 rounded-lg px-4 py-2 text-sm font-semibold transition-colors",
                            isActive
                              ? "bg-[var(--color-green)] text-white"
                              : "text-[var(--color-muted)] hover:bg-[var(--color-surface-soft)]",
                            hasError && !isActive && "text-rose-600"
                          )}
                        >
                          {s.label}
                          {s.completion ? <span className="text-xs opacity-80">{s.completion}</span> : null}
                          {hasError ? <span className="h-1.5 w-1.5 rounded-full bg-rose-500" aria-hidden="true" /> : null}
                        </button>
                      );
                    })}
                  </div>

              <div className={cn(activeSection === "basics" ? "block space-y-4" : "hidden")}>
              <div>
                <label htmlFor="admin-product-name" className="mb-1.5 block text-[15px] font-medium text-[var(--color-ink)]">
                  Name *
                </label>
                <Input
                  id="admin-product-name"
                  type="text"
                  name="name"
                  value={form.name}
                  onChange={handleChange}
                  placeholder="e.g. Ivory Silk Saree"
                  className={cn("rounded-lg text-[15px]", fieldErrors.name && "border-rose-400 focus:ring-rose-200")}
                  autoFocus
                  aria-invalid={Boolean(fieldErrors.name)}
                  aria-describedby={fieldErrors.name ? "admin-product-name-error" : undefined}
                />
                {fieldErrors.name ? (
                  <p id="admin-product-name-error" className="mt-1.5 text-sm text-rose-600" role="alert">
                    {fieldErrors.name}
                  </p>
                ) : null}
              </div>
              <div>
                <label htmlFor="admin-product-description" className="mb-1.5 block text-[15px] font-medium text-[var(--color-ink)]">
                  Description
                </label>
                <textarea
                  id="admin-product-description"
                  name="description"
                  value={form.description}
                  onChange={handleChange}
                  placeholder="Short description"
                  rows={3}
                  className={cn(
                    "w-full resize-y rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-3 text-[15px] outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20"
                  )}
                  aria-invalid={Boolean(error)}
                  aria-describedby={error ? "admin-product-form-error" : undefined}
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label htmlFor="admin-product-price" className="mb-1.5 block text-[15px] font-medium text-[var(--color-ink)]">
                    Price (₹) *
                  </label>
                  <Input
                    id="admin-product-price"
                    type="text"
                    name="priceRupees"
                    value={form.priceRupees}
                    onChange={handleChange}
                    placeholder="e.g. 499.00"
                    className={cn("rounded-lg text-[15px]", fieldErrors.priceRupees && "border-rose-400 focus:ring-rose-200")}
                    aria-invalid={Boolean(fieldErrors.priceRupees)}
                    aria-describedby={fieldErrors.priceRupees ? "admin-product-price-error" : undefined}
                  />
                  {fieldErrors.priceRupees ? (
                    <p id="admin-product-price-error" className="mt-1.5 text-sm text-rose-600" role="alert">
                      {fieldErrors.priceRupees}
                    </p>
                  ) : null}
                </div>
              </div>
              <div>
                <label htmlFor="admin-product-category" className="mb-1.5 block text-[15px] font-medium text-[var(--color-ink)]">
                  Category *
                </label>
                <select
                  id="admin-product-category"
                  name="categoryId"
                  value={form.categoryId}
                  onChange={handleChange}
                  className={cn(
                    "w-full rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-3 text-[15px] outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20",
                    fieldErrors.categoryId && "border-rose-400 focus:ring-rose-200"
                  )}
                  disabled={categoriesLoading || categoriesError}
                  aria-invalid={Boolean(fieldErrors.categoryId)}
                  aria-describedby={fieldErrors.categoryId ? "admin-product-category-error" : undefined}
                >
                  <option value="">
                    {categoriesLoading ? "Loading categories…" : "Select category"}
                  </option>
                  {categories.map((c) => (
                    <option key={c.categoryId} value={c.categoryId}>
                      {c.name || `Category ${c.categoryId}`}
                    </option>
                  ))}
                </select>
                {fieldErrors.categoryId ? (
                  <p id="admin-product-category-error" className="mt-1.5 text-sm text-rose-600" role="alert">
                    {fieldErrors.categoryId}
                  </p>
                ) : null}
                <div className="mt-2 flex items-center gap-2">
                  <button
                    type="button"
                    onClick={() => {
                      setShowNewCategory((s) => !s);
                      setCategoryError("");
                      setNewCategoryName("");
                    }}
                    className="text-sm font-medium text-[var(--color-accent-brown)] underline focus:outline-none"
                  >
                    {showNewCategory ? "Cancel" : "+ Add new category"}
                  </button>
                </div>
                {showNewCategory && (
                  <div className="mt-3 flex flex-wrap items-end gap-2 rounded-lg border border-[var(--color-line)] p-3">
                    <div className="min-w-0 flex-1">
                      <label htmlFor="admin-product-new-category" className="sr-only">New category name</label>
                      <Input
                        id="admin-product-new-category"
                        type="text"
                        value={newCategoryName}
                        onChange={(e) => {
                          setNewCategoryName(e.target.value);
                          setCategoryError("");
                        }}
                        placeholder="e.g. Silk Sarees"
                        className="rounded-lg text-[15px]"
                        autoFocus
                      />
                    </div>
                    <Button
                      type="button"
                      onClick={handleAddCategory}
                      disabled={createCategoryMutation.isPending}
                      className="rounded-lg bg-[var(--color-accent-brown)] hover:bg-[var(--color-accent-brown)]/90"
                    >
                      {createCategoryMutation.isPending ? "Adding…" : "Add category"}
                    </Button>
                    {categoryError && (
                      <p className="w-full text-sm text-red-600" role="alert">
                        {categoryError}
                      </p>
                    )}
                  </div>
                )}
                {categories.length === 0 && !showNewCategory && (
                  <p className="mt-1 text-xs text-[var(--color-muted)]">
                    No categories yet. Use &quot;Add new category&quot; above to
                    create one.
                  </p>
                )}
              </div>
              </div>

              <div className={cn(activeSection === "details" ? "block space-y-4" : "hidden")}>
              <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
                <div>
                  <label htmlFor="admin-product-sku" className="mb-1.5 block text-[15px] font-medium text-[var(--color-ink)]">SKU</label>
                  <Input
                    id="admin-product-sku"
                    type="text"
                    name="sku"
                    value={form.sku}
                    onChange={handleChange}
                    placeholder="Optional unique code"
                    className={cn("rounded-lg text-[15px]", fieldErrors.sku && "border-rose-400 focus:ring-rose-200")}
                    aria-invalid={Boolean(fieldErrors.sku)}
                  />
                  {fieldErrors.sku ? <p className="mt-1.5 text-sm text-rose-600" role="alert">{fieldErrors.sku}</p> : null}
                </div>
                <div>
                  <label htmlFor="admin-product-slug" className="mb-1.5 block text-[15px] font-medium text-[var(--color-ink)]">Slug</label>
                  <Input
                    id="admin-product-slug"
                    type="text"
                    name="slug"
                    value={form.slug}
                    onChange={handleChange}
                    placeholder="Auto-filled from the name"
                    className={cn("rounded-lg text-[15px]", fieldErrors.slug && "border-rose-400 focus:ring-rose-200")}
                    aria-invalid={Boolean(fieldErrors.slug)}
                  />
                  {fieldErrors.slug ? (
                    <p className="mt-1.5 text-sm text-rose-600" role="alert">{fieldErrors.slug}</p>
                  ) : (
                    <p className="mt-1.5 text-sm text-[var(--color-muted)]">Used in the product&apos;s web address.</p>
                  )}
                </div>
              </div>
              <div className="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-3">
                <div>
                  <label htmlFor="admin-product-fabric" className="mb-1.5 block text-[15px] font-medium text-[var(--color-ink)]">Fabric</label>
                  <select
                    id="admin-product-fabric"
                    name="fabric"
                    value={form.fabric}
                    onChange={handleChange}
                    className={cn(
                      "w-full rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-3 text-[15px] outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20"
                    )}
                  >
                    <option value="">Select fabric</option>
                    {fabrics.map((f) => (
                      <option key={f.fabricId} value={f.fabricName}>
                        {f.fabricName}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label htmlFor="admin-product-weave" className="mb-1.5 block text-[15px] font-medium text-[var(--color-ink)]">Weave</label>
                  <select
                    id="admin-product-weave"
                    name="weave"
                    value={form.weave}
                    onChange={handleChange}
                    className={cn(
                      "w-full rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-3 text-[15px] outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20"
                    )}
                  >
                    <option value="">Select weave</option>
                    {weaves.map((w) => (
                      <option key={w.weaveId} value={w.weaveName}>
                        {w.weaveName}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label htmlFor="admin-product-occasion" className="mb-1.5 block text-[15px] font-medium text-[var(--color-ink)]">Occasion</label>
                  <select
                    id="admin-product-occasion"
                    name="occasion"
                    value={form.occasion}
                    onChange={handleChange}
                    className={cn(
                      "w-full rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-3 text-[15px] outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20"
                    )}
                  >
                    <option value="">Select occasion</option>
                    {occasions.map((o) => (
                      <option key={o.occasionId} value={o.occasionName}>
                        {o.occasionName}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
              <div className="mt-4 flex items-center gap-2">
                <input
                  type="checkbox"
                  id="hasBlousePiece"
                  name="hasBlousePiece"
                  checked={form.hasBlousePiece}
                  onChange={handleChange}
                  className="h-4 w-4 rounded border-[var(--color-line)]"
                />
                <label htmlFor="hasBlousePiece" className="text-sm font-medium text-[var(--color-ink)]">
                  Has blouse piece
                </label>
              </div>
              <div className="mt-4">
                <label htmlFor="admin-product-care-instructions" className="mb-1.5 block text-[15px] font-medium text-[var(--color-ink)]">Care instructions</label>
                <textarea
                  id="admin-product-care-instructions"
                  name="careInstructions"
                  value={form.careInstructions}
                  onChange={handleChange}
                  placeholder="Optional care instructions"
                  rows={2}
                  className={cn(
                    "w-full resize-y rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-3 text-[15px] outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20"
                  )}
                />
              </div>
              <div className="mt-4">
                <label htmlFor="admin-product-status" className="mb-1.5 block text-[15px] font-medium text-[var(--color-ink)]">Product status</label>
                <select
                  id="admin-product-status"
                  name="productStatusId"
                  value={form.productStatusId}
                  onChange={handleChange}
                  className={cn(
                    "w-full max-w-xs rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-3 text-[15px] outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20"
                  )}
                >
                  <option value="">— Not set —</option>
                  <option value="1">Draft</option>
                  <option value="2">Active</option>
                  <option value="3">Archived</option>
                </select>
              </div>
              </div>

              <div className={cn(activeSection === "stock" ? "block" : "hidden")}>
              <ProductVariantsSection
                variants={variants}
                setVariants={setVariants}
                sizes={sizes}
                colors={colors}
              />
              </div>

              <div className={cn(activeSection === "moods" ? "block" : "hidden")}>
              <ProductMoodsSection
                existingMoods={existingMoods}
                selectedMoodIds={selectedMoodIds}
                setSelectedMoodIds={setSelectedMoodIds}
                newMoodName={newMoodName}
                setNewMoodName={setNewMoodName}
                moodCreateError={moodCreateError}
                setMoodCreateError={setMoodCreateError}
                createMoodMutation={createMoodMutation}
              />
              </div>

              <div className={cn(activeSection === "photos" ? "block" : "hidden")}>
              <ProductImagesSection
                imageError={imageError}
                imageMessage={imageMessage}
                fileInputRef={fileInputRef}
                setOrderedProductImages={setOrderedProductImages}
                setImageFiles={setImageFiles}
                setImageError={setImageError}
                setImageMessage={setImageMessage}
                setImageDialogOpen={setImageDialogOpen}
                editingProductId={editingProductId}
                orderedProductImages={orderedProductImages}
                existingProductImages={existingProductImages}
                imagePreviews={imagePreviews}
                imageFiles={imageFiles}
                setReorderableImages={setReorderableImages}
                setReorderImagesOpen={setReorderImagesOpen}
                productImagesLoadKey={productImagesLoadKey}
                getImageUrlWithCacheBuster={getImageUrlWithCacheBuster}
                setExistingProductImages={setExistingProductImages}
              />
              </div>
                </div>

                <div className="mt-6 lg:mt-0 lg:sticky lg:top-24">
                  <ProductFormPreview
                    name={form.name}
                    priceRupees={form.priceRupees}
                    categoryName={categoryNameById[form.categoryId] ?? ""}
                    statusLabel={getProductStatusLabel(form.productStatusId)}
                    imageUrl={
                      imagePreviews[0] ||
                      getImageUrlWithCacheBuster(existingProductImages[0], productImagesLoadKey) ||
                      undefined
                    }
                  />
                </div>
              </div>

              <div className="sticky bottom-4 z-10 mt-6 flex items-center gap-2 rounded-xl border border-[var(--color-line)] bg-white/95 p-3 shadow-[0_8px_24px_rgba(45,42,38,0.12)] backdrop-blur">
                <Button
                  type="submit"
                  disabled={createProductMutation.isPending || updateProductMutation.isPending || isUpdateReflecting}
                  className="rounded-lg bg-[var(--color-accent-brown)] hover:bg-[var(--color-accent-brown)]/90"
                >
                  {editingProductId
                    ? updateProductMutation.isPending || isUpdateReflecting
                      ? "Updating…"
                      : "Update product"
                    : createProductMutation.isPending
                      ? "Creating…"
                      : "Add product"}
                </Button>
                {editingProductId && (
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => {
                      setEditingProductId(null);
                      setForm((prev) => ({
                        ...prev,
                        name: "",
                        description: "",
                        priceRupees: "",
                        sku: "",
                        slug: "",
                        fabric: "",
                        weave: "",
                        occasion: "",
                        hasBlousePiece: true,
                        careInstructions: "",
                        productStatusId: "",
                      }));
                      setVariants([]);
                      setSelectedMoodIds([]);
                      setImageFiles([]);
                      setExistingProductImages([]);
                      setOrderedProductImages(null);
                      setInitialExistingImageIdsWhenEdit([]);
                      setInitialVariantIdsWhenEdit([]);
                      setImageError("");
                      setImageMessage("");
                      setSlugTouched(false);
                      setFieldErrors({});
                      setActiveSection("basics");
                    }}
                  >
                    Cancel edit
                  </Button>
                )}
              </div>
            </form>
          </CardContent>
        </Card>
      )}
      <ProductImagesDialogs
        imageDialogOpen={imageDialogOpen}
        setImageDialogOpen={setImageDialogOpen}
        reorderImagesOpen={reorderImagesOpen}
        setReorderImagesOpen={setReorderImagesOpen}
        editingProductId={editingProductId}
        existingProductImages={existingProductImages}
        imagePreviews={imagePreviews}
        imageFiles={imageFiles}
        reviewImagesList={reviewImagesList}
        setReviewImagesList={setReviewImagesList}
        reviewDragIndex={reviewDragIndex}
        setReviewDragIndex={setReviewDragIndex}
        dragIndex={dragIndex}
        setDragIndex={setDragIndex}
        setImageFiles={setImageFiles}
        setOrderedProductImages={setOrderedProductImages}
        setExistingProductImages={setExistingProductImages}
        reorderableImages={reorderableImages}
        setReorderableImages={setReorderableImages}
        reorderDragIndex={reorderDragIndex}
        setReorderDragIndex={setReorderDragIndex}
        productImagesLoadKey={productImagesLoadKey}
        getImageUrlWithCacheBuster={getImageUrlWithCacheBuster}
      />
    </AdminPageShell>
  );
}


