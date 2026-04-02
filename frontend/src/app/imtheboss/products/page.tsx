"use client";

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
  deleteProductImage,
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
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { SectionHeading } from "@/components/ui/typography";
import { cn } from "@/lib/utils";
import { useToast } from "@/components/ui/toast";
import { Spinner } from "@/components/ui/loading";
import { Pencil, Trash2, Filter, Package, Plus, X } from "lucide-react";
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

function getCategoryName(categoryId: string | null | undefined, categories: CategoryRow[]): string {
  if (!categoryId) return "—";
  const c = categories.find((x) => x.categoryId === categoryId);
  return c ? c.name : categoryId;
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

function getProductThumbnail(p: ProductListRow): string | null {
  const images = p.images ?? [];
  for (let i = 0; i < images.length; i++) {
    const u = getImageUrl(images[i]);
    if (u) return u;
  }
  return null;
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
  });
  const { data: colors = [] } = useQuery<ColorRow[], Error>({
    queryKey: ["admin", "colors"],
    queryFn: fetchColors,
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
  const [selectedImageIndex, setSelectedImageIndex] = useState(0);
  const [touchStartX, setTouchStartX] = useState<number | null>(null);

  const [activeTab, setActiveTab] = useState<"view" | "add">("view");
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

  const [lastCreatedProduct, setLastCreatedProduct] = useState<{
    id: string;
    name: string;
  } | null>(null);
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
  type ReorderableImage =
    | { type: "existing"; image: ProductImageListItem }
    | { type: "new"; file: File; previewUrl: string };
  const [reorderImagesOpen, setReorderImagesOpen] = useState(false);
  const [reorderableImages, setReorderableImages] = useState<ReorderableImage[]>([]);
  const [reorderDragIndex, setReorderDragIndex] = useState<number | null>(null);
  /** When editing, Review images dialog shows all images (existing + new); this is the combined list for that dialog. */
  const [reviewImagesList, setReviewImagesList] = useState<ReorderableImage[]>([]);
  const [reviewDragIndex, setReviewDragIndex] = useState<number | null>(null);
  /** When set, this is the confirmed order (existing + new interleaved). Used for display and sync so order matches Review. */
  const [orderedProductImages, setOrderedProductImages] = useState<ReorderableImage[] | null>(null);
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
  }, [categories]);

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
        setLastCreatedProduct({ id: created.productId, name: created.name });
        if (imageFiles.length > 0) {
          imageFiles.forEach((file, index) => {
            uploadImageMutation.mutate({
              file,
              productId: created.productId,
              order: index,
            });
          });
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
          }
        }
        // Link moods
        for (const moodId of selectedMoodIds) {
          if (!moodId?.trim()) continue;
          try {
            await createProductMoodMapping(created.productId, moodId.trim());
          } catch (err) {
            console.error("Failed to link mood:", err);
          }
        }
        setVariants([]);
        setSelectedMoodIds([]);
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
    setForm((prev) => {
      const next = { ...prev, [name]: type === "checkbox" ? checked : value };
      if (typeof window !== "undefined") {
        window.sessionStorage.setItem(DRAFT_KEY, JSON.stringify(next));
      }
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

    if (!editingProductId && imageFiles.length === 0) {
      setImageError("At least one product image is required.");
      return;
    }
    const parsed = adminProductFormSchema.safeParse(form);
    if (!parsed.success) {
      const first = parsed.error.flatten().fieldErrors;
      const msg =
        first.name?.[0] ??
        first.priceRupees?.[0] ??
        first.categoryId?.[0] ??
        first.sku?.[0] ??
        first.slug?.[0] ??
        parsed.error.message;
      setError(msg);
      return;
    }
    const { name, description, priceRupees, categoryId, sku, slug, fabric, weave, occasion, hasBlousePiece, careInstructions, productStatusId } =
      parsed.data;
    const pricePaise = rupeesInputToPaise(priceRupees || "0");
    if (pricePaise <= 0) {
      setError("Price must be greater than 0.");
      return;
    }
    if (pricePaise > MAX_MONEY_PAISE) {
      setError("Price exceeds supported maximum.");
      return;
    }
    if (variants.length === 0) {
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

  useEffect(() => {
    setSelectedImageIndex(0);
  }, [selectedProduct?.productId]);

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

  return (
    <div className="mx-auto max-w-6xl w-full">
      <div className="mb-8">
        <p className="text-sm text-[var(--color-muted)]">Products</p>
        <SectionHeading size="default" className="mt-1">
          Product catalog
        </SectionHeading>
        <p className="mt-1 text-sm leading-relaxed text-[var(--color-muted)]">
          View, search, and add products. Filter by category, status, and price.
        </p>
      </div>

      <div className="mb-6 flex flex-wrap gap-3">
        {activeTab === "view" && (
          <span className="inline-flex items-center gap-2 rounded-full bg-violet-500/12 px-4 py-2 text-sm font-medium text-violet-700">
            <Package className="h-4 w-4" />
            {products.length} product{products.length !== 1 ? "s" : ""}
          </span>
        )}
      </div>

      <div className="mt-6 inline-flex rounded-full border border-[var(--color-line)] bg-white shadow-sm p-1 text-xs">
        <button
          type="button"
          onClick={() => setActiveTab("view")}
          className={cn(
            "rounded-full px-4 py-1.5 font-medium transition-colors",
            activeTab === "view"
              ? "bg-[var(--color-ink)] text-white"
              : "text-[var(--color-muted)] hover:bg-[var(--color-line)]/40"
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
          }}
          className={cn(
            "rounded-full px-4 py-1.5 font-medium transition-colors",
            activeTab === "add"
              ? "bg-[var(--color-ink)] text-white"
              : "text-[var(--color-muted)] hover:bg-[var(--color-line)]/40"
          )}
        >
          Add product
        </button>
      </div>

      {activeTab === "view" && (
        <>
          <Card className="mt-6 rounded-xl border-[var(--color-line)] border-l-4 border-l-blue-500 bg-white shadow-[var(--admin-card-shadow)]">
            <CardTitle className="flex items-center gap-2 text-[var(--color-muted)]">
              <Filter className="h-4 w-4 text-blue-500" />
              Filters
            </CardTitle>
            <CardContent className="mt-3">
              <form
                onSubmit={handleSearchSubmit}
                className="flex flex-wrap items-end gap-3"
              >
                <div>
                  <label htmlFor="products-name" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Name
                  </label>
                  <Input
                    id="products-name"
                    type="text"
                    value={searchName}
                    onChange={(e) => setSearchName(e.target.value)}
                    placeholder="e.g. silk"
                    className="h-9 w-40 rounded-md"
                  />
                </div>
                <div>
                  <label htmlFor="products-category" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Category
                  </label>
                  <select
                    id="products-category"
                    className={cn(
                      "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                      "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
                    )}
                    value={searchCategoryId}
                    onChange={(e) => setSearchCategoryId(e.target.value)}
                  >
                    <option value="">All categories</option>
                    {categories.map((c) => (
                      <option key={c.categoryId} value={c.categoryId}>
                        {c.name}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label htmlFor="products-status" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Status
                  </label>
                  <select
                    id="products-status"
                    className={cn(
                      "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                      "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
                    )}
                    value={searchProductStatusId}
                    onChange={(e) => setSearchProductStatusId(e.target.value)}
                  >
                    <option value="">All statuses</option>
                    <option value="1">Draft</option>
                    <option value="2">Active</option>
                    <option value="3">Archived</option>
                  </select>
                </div>
                <div>
                  <label htmlFor="products-mood" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Mood
                  </label>
                  <select
                    id="products-mood"
                    className={cn(
                      "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                      "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
                    )}
                    value={searchMoodId}
                    onChange={(e) => setSearchMoodId(e.target.value)}
                  >
                    <option value="">All moods</option>
                    {existingMoods.map((m) => (
                      <option key={m.moodId} value={m.moodId}>
                        {m.moodName}
                      </option>
                    ))}
                  </select>
                </div>
                <div className="min-w-[14rem]">
                  <label htmlFor="products-price-min" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Price range (₹)
                  </label>
                  <div className="flex flex-col gap-2">
                    <div className="flex items-center gap-2">
                      <Input
                        id="products-price-min"
                        type="number"
                        min={0}
                        step={0.01}
                        placeholder="Min"
                        value={searchPriceMinRupees}
                        onChange={(e) => setSearchPriceMinRupees(e.target.value)}
                        className="h-9 w-24 rounded-md"
                      />
                      <span className="text-xs text-[var(--color-muted)]">–</span>
                      <Input
                        id="products-price-max"
                        type="number"
                        min={0}
                        step={0.01}
                        placeholder="Max"
                        value={searchPriceMaxRupees}
                        onChange={(e) => setSearchPriceMaxRupees(e.target.value)}
                        className="h-9 w-24 rounded-md"
                      />
                    </div>
                    <div className="flex items-center gap-2">
                      <input
                        type="range"
                        min={0}
                        max={50000}
                        step={100}
                        value={Math.min(Number(searchPriceMinRupees) || 0, 50000)}
                        onChange={(e) => setSearchPriceMinRupees(e.target.value)}
                        className="h-2 w-24 flex-1 cursor-pointer appearance-none rounded-lg bg-[var(--color-line)] accent-[var(--color-accent-brown)]"
                        aria-label="Min price (₹)"
                      />
                      <input
                        type="range"
                        min={0}
                        max={50000}
                        step={100}
                        value={searchPriceMaxRupees === "" ? 50000 : Math.min(Number(searchPriceMaxRupees), 50000)}
                        onChange={(e) => setSearchPriceMaxRupees(e.target.value)}
                        className="h-2 w-24 flex-1 cursor-pointer appearance-none rounded-lg bg-[var(--color-line)] accent-[var(--color-accent-brown)]"
                        aria-label="Max price (₹)"
                      />
                    </div>
                  </div>
                </div>
                <div>
                  <label htmlFor="products-limit" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Limit
                  </label>
                  <Input
                    id="products-limit"
                    type="number"
                    min={1}
                    max={100}
                    value={searchLimit}
                    onChange={(e) => setSearchLimit(e.target.value)}
                    className="h-9 w-20 rounded-md"
                  />
                </div>
                <div>
                  <label htmlFor="products-fabric" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Fabric
                  </label>
                  <select
                    id="products-fabric"
                    className={cn(
                      "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                      "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
                    )}
                    value={searchFabric}
                    onChange={(e) => setSearchFabric(e.target.value)}
                  >
                    <option value="">All fabrics</option>
                    {fabrics.map((f) => (
                      <option key={f.fabricId} value={f.fabricName}>
                        {f.fabricName}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label htmlFor="products-weave" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Weave
                  </label>
                  <select
                    id="products-weave"
                    className={cn(
                      "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                      "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
                    )}
                    value={searchWeave}
                    onChange={(e) => setSearchWeave(e.target.value)}
                  >
                    <option value="">All weaves</option>
                    {weaves.map((w) => (
                      <option key={w.weaveId} value={w.weaveName}>
                        {w.weaveName}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label htmlFor="products-occasion" className="mb-1 block text-xs text-[var(--color-muted)]">
                    Occasion
                  </label>
                  <select
                    id="products-occasion"
                    className={cn(
                      "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                      "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
                    )}
                    value={searchOccasion}
                    onChange={(e) => setSearchOccasion(e.target.value)}
                  >
                    <option value="">All occasions</option>
                    {occasions.map((o) => (
                      <option key={o.occasionId} value={o.occasionName}>
                        {o.occasionName}
                      </option>
                    ))}
                  </select>
                </div>
                <div className="flex gap-2">
                  <Button type="submit" size="sm">
                    Apply
                  </Button>
                  <Button type="button" variant="outline" size="sm" onClick={handleSearchClear}>
                    Clear
                  </Button>
                  <Button type="button" variant="outline" size="sm" onClick={() => refetchProducts()}>
                    Refresh
                  </Button>
                </div>
              </form>
            </CardContent>
          </Card>

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
                  <Button variant="outline" size="sm" className="mt-2" onClick={() => refetchProducts()}>
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
                    const thumb = getProductThumbnailWithCacheBuster(p);
                    return (
                      <div
                        key={p.productId}
                        className="overflow-hidden rounded-xl border border-[var(--color-line)] bg-white"
                        onClick={() => setSelectedProduct(p)}
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
                            {getCategoryName(p.categoryId, categories)}
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
                                  beginEditProduct(p);
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
                                  setArchiveConfirm(p);
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

          {archiveConfirm && (
            <Dialog open={!!archiveConfirm} onOpenChange={(open) => !open && setArchiveConfirm(null)}>
              <DialogContent className="sm:max-w-md">
                <p className="text-sm text-[var(--color-ink)]">
                  Archive product <strong>{archiveConfirm.name}</strong> (ID: {archiveConfirm.productId})? Its status will be set to Archived; it will not be deleted.
                </p>
                <div className="mt-4 flex justify-end gap-2">
                  <Button variant="outline" onClick={() => setArchiveConfirm(null)}>
                    Cancel
                  </Button>
                  <Button
                    variant="outline"
                    className="border-red-200 text-red-600 hover:bg-red-50"
                    onClick={handleArchiveConfirm}
                    disabled={updateProductMutation.isPending}
                  >
                    {updateProductMutation.isPending ? "Archiving…" : "Archive"}
                  </Button>
                </div>
              </DialogContent>
            </Dialog>
          )}

          {selectedProduct && (
            (() => {
              const imageUrls = (selectedProduct.images ?? [])
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
              const hasImages = imageUrls.length > 0;
              const activeImage = hasImages
                ? imageUrls[Math.min(selectedImageIndex, imageUrls.length - 1)]
                : null;
              return (
            <Dialog
              open={!!selectedProduct}
              onOpenChange={(open) => !open && setSelectedProduct(null)}
            >
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
                      <img
                        src={activeImage}
                        alt={selectedProduct.name}
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
                  <div className="space-y-2 text-sm">
                    <h3 className="text-base font-semibold text-[var(--color-ink)]">
                      {selectedProduct.name}
                    </h3>
                    <p className="text-[var(--color-muted)]">
                      {selectedProduct.description || "No description"}
                    </p>
                    <div className="rounded-md border border-[var(--color-line)] p-3">
                      <p>
                        <span className="font-medium">Product ID:</span>{" "}
                        <span className="font-mono">{selectedProduct.productId}</span>
                      </p>
                      <p>
                        <span className="font-medium">Category:</span>{" "}
                        {getCategoryName(selectedProduct.categoryId, categories)}
                      </p>
                      <p>
                        <span className="font-medium">Price:</span>{" "}
                        {selectedProduct.formatted}
                      </p>
                      <p>
                        <span className="font-medium">Stock:</span>{" "}
                        {selectedProduct.stockQuantity ?? "—"}
                      </p>
                      <p>
                        <span className="font-medium">SKU:</span>{" "}
                        {selectedProduct.sku ?? "—"}
                      </p>
                      <p>
                        <span className="font-medium">Slug:</span>{" "}
                        {selectedProduct.slug ?? "—"}
                      </p>
                      <p>
                        <span className="font-medium">Fabric:</span>{" "}
                        {selectedProduct.fabric ?? "—"}
                      </p>
                      <p>
                        <span className="font-medium">Weave:</span>{" "}
                        {selectedProduct.weave ?? "—"}
                      </p>
                      <p>
                        <span className="font-medium">Occasion:</span>{" "}
                        {selectedProduct.occasion ?? "—"}
                      </p>
                      <p>
                        <span className="font-medium">Has blouse piece:</span>{" "}
                        {selectedProduct.hasBlousePiece == null
                          ? "—"
                          : selectedProduct.hasBlousePiece
                            ? "Yes"
                            : "No"}
                      </p>
                      <p>
                        <span className="font-medium">Care instructions:</span>{" "}
                        {selectedProduct.careInstructions ?? "—"}
                      </p>
                      <p>
                        <span className="font-medium">Product status:</span>{" "}
                        {getProductStatusLabel(selectedProduct.productStatusId)}
                      </p>
                    </div>
                    {imageUrls.length > 1 && (
                      <div className="flex gap-2 pt-2">
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={() =>
                            setSelectedImageIndex((prev) =>
                              prev === 0 ? imageUrls.length - 1 : prev - 1
                            )
                          }
                        >
                          Prev
                        </Button>
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          onClick={() =>
                            setSelectedImageIndex((prev) =>
                              prev === imageUrls.length - 1 ? 0 : prev + 1
                            )
                          }
                        >
                          Next
                        </Button>
                      </div>
                    )}
                  </div>
                </div>
              </DialogContent>
            </Dialog>
              );
            })()
          )}
        </>
      )}

      {activeTab === "add" && (
        <Card className="mt-6 rounded-xl border-[var(--color-line)] border-l-4 border-l-emerald-500 bg-white shadow-[var(--admin-card-shadow)]">
          <CardTitle className="flex items-center gap-2 text-[var(--color-muted)]">
            {editingProductId ? (
              <Pencil className="h-4 w-4 text-emerald-500" />
            ) : (
              <Plus className="h-4 w-4 text-emerald-500" />
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
                  className="mt-2 inline-flex items-center rounded-full border border-red-300 bg-white px-3 py-1 text-xs font-medium text-red-800 hover:bg-red-50"
                >
                  Try again
                </button>
              </div>
            )}
            {error && (
              <div
                className="mb-4 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800"
                role="alert"
              >
                {error}
              </div>
            )}
            {message && (
              <div
                className="mb-4 rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-800"
                role="status"
              >
                {message}
              </div>
            )}
            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <label className="mb-1 block text-sm font-medium text-[var(--color-ink)]">
                  Name *
                </label>
                <Input
                  type="text"
                  name="name"
                  value={form.name}
                  onChange={handleChange}
                  placeholder="e.g. Ivory Silk Saree"
                  className="rounded-lg"
                  autoFocus
                />
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-[var(--color-ink)]">
                  Description
                </label>
                <textarea
                  name="description"
                  value={form.description}
                  onChange={handleChange}
                  placeholder="Short description"
                  rows={3}
                  className={cn(
                    "w-full resize-y rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-2.5 text-sm outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20"
                  )}
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="mb-1 block text-sm font-medium text-[var(--color-ink)]">
                    Price (₹) *
                  </label>
                  <Input
                    type="text"
                    name="priceRupees"
                    value={form.priceRupees}
                    onChange={handleChange}
                    placeholder="e.g. 499.00"
                    className="rounded-lg"
                  />
                </div>
              </div>
              <div>
                <label className="mb-1 block text-sm font-medium text-[var(--color-ink)]">
                  Category *
                </label>
                <select
                  name="categoryId"
                  value={form.categoryId}
                  onChange={handleChange}
                  className={cn(
                    "w-full rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-2.5 text-sm outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20"
                  )}
                  disabled={categoriesLoading || categoriesError}
                  required
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
                      <label className="sr-only">New category name</label>
                      <Input
                        type="text"
                        value={newCategoryName}
                        onChange={(e) => {
                          setNewCategoryName(e.target.value);
                          setCategoryError("");
                        }}
                        placeholder="e.g. Silk Sarees"
                        className="rounded-lg"
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
              <div className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2">
                <div>
                  <label className="mb-1 block text-sm font-medium text-[var(--color-ink)]">SKU</label>
                  <Input
                    type="text"
                    name="sku"
                    value={form.sku}
                    onChange={handleChange}
                    placeholder="Optional unique code"
                    className="rounded-lg"
                  />
                </div>
                <div>
                  <label className="mb-1 block text-sm font-medium text-[var(--color-ink)]">Slug</label>
                  <Input
                    type="text"
                    name="slug"
                    value={form.slug}
                    onChange={handleChange}
                    placeholder="Optional URL slug"
                    className="rounded-lg"
                  />
                </div>
              </div>
              <div className="mt-4 grid grid-cols-1 gap-4 sm:grid-cols-3">
                <div>
                  <label className="mb-1 block text-sm font-medium text-[var(--color-ink)]">Fabric</label>
                  <select
                    name="fabric"
                    value={form.fabric}
                    onChange={handleChange}
                    className={cn(
                      "w-full rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-2.5 text-sm outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20"
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
                  <label className="mb-1 block text-sm font-medium text-[var(--color-ink)]">Weave</label>
                  <select
                    name="weave"
                    value={form.weave}
                    onChange={handleChange}
                    className={cn(
                      "w-full rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-2.5 text-sm outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20"
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
                  <label className="mb-1 block text-sm font-medium text-[var(--color-ink)]">Occasion</label>
                  <select
                    name="occasion"
                    value={form.occasion}
                    onChange={handleChange}
                    className={cn(
                      "w-full rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-2.5 text-sm outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20"
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
                <label className="mb-1 block text-sm font-medium text-[var(--color-ink)]">Care instructions</label>
                <textarea
                  name="careInstructions"
                  value={form.careInstructions}
                  onChange={handleChange}
                  placeholder="Optional care instructions"
                  rows={2}
                  className={cn(
                    "w-full resize-y rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-2.5 text-sm outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20"
                  )}
                />
              </div>
              <div className="mt-4">
                <label className="mb-1 block text-sm font-medium text-[var(--color-ink)]">Product status</label>
                <select
                  name="productStatusId"
                  value={form.productStatusId}
                  onChange={handleChange}
                  className={cn(
                    "w-full max-w-xs rounded-lg border border-[var(--color-line)] bg-white/60 px-4 py-2.5 text-sm outline-none focus:bg-white focus:ring-2 focus:ring-[var(--color-ink)]/20"
                  )}
                >
                  <option value="">— Not set —</option>
                  <option value="1">Draft</option>
                  <option value="2">Active</option>
                  <option value="3">Archived</option>
                </select>
              </div>
              <div className="mt-8 border-t border-[var(--color-line)] pt-4">
                <h3 className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">
                  Variants *
                </h3>
                <p className="mt-2 text-xs text-[var(--color-muted)]">
                  Add size/color combinations. Each variant can have an extra price (paise) and initial stock.
                </p>
                <div className="mt-3 space-y-2">
                  {variants.map((v, idx) => (
                    <div
                      key={idx}
                      className="flex flex-wrap items-end gap-2 rounded-lg border border-[var(--color-line)] bg-white/40 p-3"
                    >
                      <select
                        className={cn(
                          "h-9 min-w-[6rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                          "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
                        )}
                        value={v.sizeId}
                        onChange={(e) =>
                          setVariants((prev) => {
                            const next = [...prev];
                            next[idx] = { ...next[idx], sizeId: e.target.value };
                            return next;
                          })
                        }
                      >
                        <option value="">Select size</option>
                        {sizes.map((s) => (
                          <option key={s.sizeId} value={s.sizeId}>{s.sizeName}</option>
                        ))}
                      </select>
                      <select
                        className={cn(
                          "h-9 min-w-[6rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                          "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
                        )}
                        value={v.colorId}
                        onChange={(e) =>
                          setVariants((prev) => {
                            const next = [...prev];
                            next[idx] = { ...next[idx], colorId: e.target.value };
                            return next;
                          })
                        }
                      >
                        <option value="">Select color</option>
                        {colors.map((c) => (
                          <option key={c.colorId} value={c.colorId}>{c.colorName}</option>
                        ))}
                      </select>
                      <Input
                        type="text"
                        placeholder="Extra price (paise)"
                        value={v.additionalPricePaise}
                        onChange={(e) =>
                          setVariants((prev) => {
                            const next = [...prev];
                            next[idx] = { ...next[idx], additionalPricePaise: e.target.value };
                            return next;
                          })
                        }
                        className="h-9 w-28 rounded-md"
                      />
                      <Input
                        type="text"
                        placeholder="Qty"
                        value={v.quantityAvailable}
                        onChange={(e) =>
                          setVariants((prev) => {
                            const next = [...prev];
                            next[idx] = { ...next[idx], quantityAvailable: e.target.value };
                            return next;
                          })
                        }
                        className="h-9 w-20 rounded-md"
                      />
                      <Input
                        type="text"
                        placeholder="Reorder"
                        value={v.reorderLevel}
                        onChange={(e) =>
                          setVariants((prev) => {
                            const next = [...prev];
                            next[idx] = { ...next[idx], reorderLevel: e.target.value };
                            return next;
                          })
                        }
                        className="h-9 w-20 rounded-md"
                      />
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        className="h-9 text-red-600"
                        onClick={() => setVariants((prev) => prev.filter((_, i) => i !== idx))}
                      >
                        Remove
                      </Button>
                    </div>
                  ))}
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    className="rounded-lg border-[var(--color-line)]"
                    onClick={() =>
                      setVariants((prev) => [
                        ...prev,
                        {
                          sizeId: "",
                          colorId: "",
                          additionalPricePaise: "",
                          quantityAvailable: "0",
                          reorderLevel: "",
                        },
                      ])
                    }
                  >
                    + Add variant
                  </Button>
                </div>
              </div>
              <div className="mt-6 border-t border-[var(--color-line)] pt-4">
                <h3 className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">
                  Moods (optional)
                </h3>
                <p className="mt-2 text-xs text-[var(--color-muted)]">
                  Select one or more moods to link to this product.
                </p>
                <div className="mt-3 grid grid-cols-4 gap-x-6 gap-y-2">
                  {existingMoods.map((m) => (
                    <label
                      key={m.moodId}
                      className={cn(
                        "flex cursor-pointer items-center gap-2 text-sm",
                        "focus-within:outline-none focus-within:ring-2 focus-within:ring-[var(--color-accent-gold)]/50 focus-within:ring-offset-1 rounded"
                      )}
                    >
                      <input
                        type="checkbox"
                        checked={selectedMoodIds.includes(m.moodId)}
                        onChange={() => {
                          setSelectedMoodIds((prev) =>
                            prev.includes(m.moodId)
                              ? prev.filter((id) => id !== m.moodId)
                              : [...prev, m.moodId]
                          );
                        }}
                        className="h-4 w-4 rounded border-[var(--color-line)] text-[var(--color-accent-gold)] focus:ring-[var(--color-accent-gold)]/50"
                      />
                      <span>{m.moodName}</span>
                    </label>
                  ))}
                </div>
                <div className="mt-3 flex flex-wrap items-center gap-2">
                  <Input
                    placeholder="New mood name"
                    value={newMoodName}
                    onChange={(e) => {
                      setNewMoodName(e.target.value);
                      setMoodCreateError("");
                    }}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                        const name = newMoodName.trim();
                        if (name) createMoodMutation.mutate(name);
                      }
                    }}
                    className="max-w-[14rem] rounded-md border border-[var(--color-line)] bg-white px-3 py-1.5 text-sm"
                  />
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    disabled={!newMoodName.trim() || createMoodMutation.isPending}
                    onClick={() => {
                      const name = newMoodName.trim();
                      if (name) createMoodMutation.mutate(name);
                    }}
                  >
                    {createMoodMutation.isPending ? "Adding…" : "Add mood"}
                  </Button>
                </div>
                {moodCreateError && (
                  <p className="mt-2 text-xs text-red-600" role="alert">{moodCreateError}</p>
                )}
                {existingMoods.length === 0 && !newMoodName && (
                  <p className="mt-2 text-xs text-[var(--color-muted)]">No moods in the system yet. Add one above or add moods from seed data.</p>
                )}
              </div>
              <div className="mt-8 border-t border-[var(--color-line)] pt-4">
                <h3 className="text-xs font-semibold uppercase tracking-[0.2em] text-[var(--color-muted)]">
                  Images *
                </h3>
                <p className="mt-2 text-xs text-[var(--color-muted)]">
                  Select at least one image. All selected images will be uploaded
                  and linked after the product is created.
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
                      Choose images…
                    </Button>
                  </div>
                  <p className="text-[11px] text-[var(--color-muted)]">
                    {editingProductId
                      ? "Add more images below; they will upload when you click Update product."
                      : "All selected images will be uploaded when you click Add product."}
                  </p>
                  {(orderedProductImages !== null ||
                    existingProductImages.length > 0 ||
                    imagePreviews.length > 0) && (
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
                        {editingProductId &&
                          (existingProductImages.length > 0 || imagePreviews.length > 0) && (
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
                        {orderedProductImages != null
                          ? orderedProductImages.map((item, idx) => (
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
                                    const src = getImageUrlWithCacheBuster(
                                      item.image,
                                      productImagesLoadKey
                                    );
                                    return src ? (
                                      <img
                                        src={src}
                                        alt=""
                                        className="h-full w-full object-cover"
                                      />
                                    ) : (
                                      <div className="flex h-full w-full items-center justify-center bg-[var(--color-line)]/30 text-[10px] text-[var(--color-muted)]">
                                        No image
                                      </div>
                                    );
                                  })()
                                ) : (
                                  <img
                                    src={item.previewUrl}
                                    alt=""
                                    className="h-full w-full object-cover"
                                  />
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
                                      setOrderedProductImages((prev) =>
                                        prev ? prev.filter((_, i) => i !== idx) : prev
                                      );
                                      setImageMessage(
                                        "Image will be removed when you click Update product."
                                      );
                                    }}
                                  >
                                    <X className="h-3.5 w-3.5" />
                                  </button>
                                )}
                              </div>
                            ))
                          : (
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
                                        setImageMessage(
                                          "Image will be removed when you click Update product."
                                        );
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
                                  <img
                                    src={url}
                                    alt=""
                                    className="h-full w-full object-cover"
                                  />
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
              <div className="mt-4 flex items-center gap-2">
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
                ? "All product images — drag to reorder. First is the thumbnail."
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
                {existingProductImages.length + imagePreviews.length === 1 ? "" : "s"} — drag to
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
                  if (
                    editingProductId &&
                    (existingProductImages.length > 0 || imagePreviews.length > 0)
                  ) {
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
                          ] as ReorderableImage[]);
                    setOrderedProductImages(listToApply);
                    const existing = listToApply
                      .filter(
                        (x): x is { type: "existing"; image: ProductImageListItem } =>
                          x.type === "existing"
                      )
                      .map((x) => x.image);
                    const newFiles = listToApply
                      .filter(
                        (x): x is { type: "new"; file: File; previewUrl: string } => x.type === "new"
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
                  reorderDragIndex === idx && "scale-[1.03] border-[var(--color-ink)] ring-1 ring-[var(--color-ink)]"
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
                  <img
                    src={item.previewUrl}
                    alt=""
                    className="h-full w-full object-cover"
                  />
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
                  .filter((x): x is { type: "existing"; image: ProductImageListItem } => x.type === "existing")
                  .map((x) => x.image);
                const newFiles = reorderableImages
                  .filter((x): x is { type: "new"; file: File; previewUrl: string } => x.type === "new")
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
    </div>
  );
}
