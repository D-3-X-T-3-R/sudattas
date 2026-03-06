"use client";

import { useState, useEffect, useRef } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { gqlAdmin } from "@/lib/graphqlAdmin";
import { adminProductFormSchema } from "@/lib/schemas";
import {
  fetchCategories,
  fetchProductsList,
  deleteProduct,
  fetchSizes,
  fetchColors,
  fetchFabrics,
  fetchWeaves,
  fetchOccasions,
  searchProductAttributes,
  createProductVariant,
  createProductAttribute,
  createProductAttributeMapping,
  createInventoryItem,
  type ProductListRow,
  type CategoryRow,
  type SizeRow,
  type ColorRow,
  type ProductAttributeRow,
  type FabricRow,
  type WeaveRow,
  type OccasionRow,
} from "@/lib/admin-queries";
import type { AdminProductVariantRow, AdminProductAttributeRow } from "@/lib/schemas";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Section } from "@/components/ui/section";
import { Kicker, SectionHeading } from "@/components/ui/typography";
import { cn } from "@/lib/utils";
import { Pencil, Trash2 } from "lucide-react";

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

function getProductThumbnail(p: ProductListRow): string | null {
  const first = p.images?.[0];
  return first?.thumbnailUrl ?? first?.url ?? null;
}

function paiseToRupeesInput(paise?: string): string {
  const n = Number(paise ?? "0");
  if (!Number.isFinite(n)) return "";
  return (n / 100).toFixed(2);
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
  const [searchFabric, setSearchFabric] = useState("");
  const [searchWeave, setSearchWeave] = useState("");
  const [searchOccasion, setSearchOccasion] = useState("");
  const [searchLimit, setSearchLimit] = useState("20");
  const [appliedSearch, setAppliedSearch] = useState<{
    name?: string;
    categoryId?: string;
    fabric?: string;
    weave?: string;
    occasion?: string;
    limit?: string;
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
  const { data: existingAttributes = [] } = useQuery<ProductAttributeRow[], Error>({
    queryKey: ["admin", "productAttributes"],
    queryFn: () => searchProductAttributes({}),
  });

  const { data: fabrics = [] } = useQuery<FabricRow[], Error>({
    queryKey: ["admin", "fabrics"],
    queryFn: fetchFabrics,
  });

  const deleteProductMutation = useMutation({
    mutationFn: deleteProduct,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["admin", "products"] });
    },
  });

  const [deleteConfirm, setDeleteConfirm] = useState<{ productId: string; name: string } | null>(null);
  const [selectedProduct, setSelectedProduct] = useState<ProductListRow | null>(null);
  const [selectedImageIndex, setSelectedImageIndex] = useState(0);
  const [touchStartX, setTouchStartX] = useState<number | null>(null);

  const [activeTab, setActiveTab] = useState<"view" | "add">("view");
  const [editingProductId, setEditingProductId] = useState<string | null>(null);

  const [form, setForm] = useState<ProductFormState>({
    name: "",
    description: "",
    priceRupees: "",
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
  const [attributes, setAttributes] = useState<AdminProductAttributeRow[]>([]);
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
  const [imageError, setImageError] = useState("");
  const [imageMessage, setImageMessage] = useState("");
  const [imageDialogOpen, setImageDialogOpen] = useState(false);
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  // Load draft from sessionStorage on first mount (product fields only; variants/attributes not persisted)
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
      setMessage(
        created
          ? `Created: ${created.name}${
              created.formatted ? ` (${created.formatted})` : ""
            }`
          : "Product created."
      );
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
        // Create or link attributes
        for (const attr of attributes) {
          if (!attr.attributeName?.trim() || !attr.attributeValue?.trim()) continue;
          try {
            let attributeId = attr.attributeId;
            if (!attributeId) {
              const createdAttr = await createProductAttribute(attr.attributeName.trim(), attr.attributeValue.trim());
              attributeId = createdAttr?.attributeId ?? undefined;
            }
            if (attributeId) {
              await createProductAttributeMapping(created.productId, attributeId);
            }
          } catch (err) {
            console.error("Failed to create/link attribute:", err);
          }
        }
        setVariants([]);
        setAttributes([]);
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
    onError: (err: Error) => setError(err.message || "Failed to create product."),
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
      return data?.updateProduct?.[0];
    },
    onSuccess: async (updated) => {
      if (updated?.productId && imageFiles.length > 0) {
        imageFiles.forEach((file, index) => {
          uploadImageMutation.mutate({
            file,
            productId: updated.productId,
            order: index,
          });
        });
      }
      setMessage(
        updated
          ? `Updated: ${updated.name}${updated.formatted ? ` (${updated.formatted})` : ""}`
          : "Product updated."
      );
      setEditingProductId(null);
      setImageFiles([]);
      queryClient.invalidateQueries({ queryKey: ["admin", "products"] });
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
      const msg = first.name?.[0] ?? first.categoryId?.[0] ?? parsed.error.message;
      setError(msg);
      return;
    }
    const { name, description, priceRupees, categoryId, sku, slug, fabric, weave, occasion, hasBlousePiece, careInstructions, productStatusId } =
      parsed.data;
    const pricePaise = Math.round(parseFloat(priceRupees || "0") * 100);
    if (isNaN(pricePaise) || pricePaise < 0) {
      setError("Enter a valid price (e.g. 499.00).");
      return;
    }
    if (!editingProductId && variants.length === 0) {
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
      });
    } else {
      createProductMutation.mutate({
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

  const categoriesAuthLike =
    categoriesError &&
    categoriesErrorObj &&
    /unauthorized|forbidden|admin/i.test(categoriesErrorObj.message);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const next: {
      name?: string;
      categoryId?: string;
      fabric?: string;
      weave?: string;
      occasion?: string;
      limit?: string;
    } = {};
    const trimmedName = searchName.trim();
    const trimmedLimit = searchLimit.trim();
    if (trimmedName) next.name = trimmedName;
    if (searchCategoryId) next.categoryId = searchCategoryId;
    if (searchFabric) next.fabric = searchFabric;
    if (searchWeave) next.weave = searchWeave;
    if (searchOccasion) next.occasion = searchOccasion;
    if (trimmedLimit) next.limit = trimmedLimit;
    setAppliedSearch(next);
  };

  const handleSearchClear = () => {
    setSearchName("");
    setSearchCategoryId("");
    setSearchFabric("");
    setSearchWeave("");
    setSearchOccasion("");
    setSearchLimit("20");
    setAppliedSearch({ limit: "20" });
  };

  const handleDeleteConfirm = () => {
    if (!deleteConfirm) return;
    deleteProductMutation.mutate(deleteConfirm.productId, {
      onSettled: () => setDeleteConfirm(null),
    });
  };

  const beginEditProduct = (p: ProductListRow) => {
    setActiveTab("add");
    setEditingProductId(p.productId);
    setForm((prev) => ({
      ...prev,
      name: p.name ?? "",
      description: p.description ?? "",
      priceRupees: paiseToRupeesInput(p.amountPaise),
      categoryId: p.categoryId ?? prev.categoryId ?? "",
      // Keep optional fields editable by user; existing values are not returned in product list query.
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
    setAttributes([]);
    setImageFiles([]);
    setImageError("");
    setImageMessage("");
    setError("");
    setMessage(`Editing product: ${p.name}`);
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
    <Section compact className="max-w-6xl">
      <Kicker className="text-[var(--color-muted)]">Products</Kicker>
      <SectionHeading size="default" className="mt-2">
        Product catalog
      </SectionHeading>

      <div className="mt-6 inline-flex rounded-full border border-[var(--color-line)] bg-white/70 p-1 text-xs">
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
          onClick={() => setActiveTab("add")}
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
          <Card className="mt-6 border-[var(--color-line)]">
            <CardTitle className="text-[var(--color-muted)]">Filters</CardTitle>
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

          <Card className="mt-6 border-[var(--color-line)]">
            <CardTitle className="text-[var(--color-muted)]">Products</CardTitle>
            <CardContent className="mt-3">
              {productsError && (
                <div className="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-800">
                  <p className="font-medium">Could not load products.</p>
                  <p className="mt-1 text-xs">{productsErrorObj?.message ?? "Unknown error"}</p>
                  <Button variant="outline" size="sm" className="mt-2" onClick={() => refetchProducts()}>
                    Try again
                  </Button>
                </div>
              )}
              {productsLoading && !productsError && (
                <p className="py-8 text-center text-sm text-[var(--color-muted)]">Loading products…</p>
              )}
              {!productsLoading && !productsError && products.length === 0 && (
                <p className="py-8 text-center text-sm text-[var(--color-muted)]">
                  No products match. Create some in the <strong>Add product</strong> tab.
                </p>
              )}
              {!productsLoading && !productsError && products.length > 0 && (
                <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
                  {products.map((p) => {
                    const thumb = getProductThumbnail(p);
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
                                aria-label={`Delete ${p.name}`}
                                title="Delete"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  setDeleteConfirm({ productId: p.productId, name: p.name });
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

          {deleteConfirm && (
            <Dialog open={!!deleteConfirm} onOpenChange={(open) => !open && setDeleteConfirm(null)}>
              <DialogContent className="sm:max-w-md">
                <p className="text-sm text-[var(--color-ink)]">
                  Delete product <strong>{deleteConfirm.name}</strong> (ID: {deleteConfirm.productId})? This cannot be undone.
                </p>
                <div className="mt-4 flex justify-end gap-2">
                  <Button variant="outline" onClick={() => setDeleteConfirm(null)}>
                    Cancel
                  </Button>
                  <Button
                    variant="destructive"
                    onClick={handleDeleteConfirm}
                    disabled={deleteProductMutation.isPending}
                  >
                    {deleteProductMutation.isPending ? "Deleting…" : "Delete"}
                  </Button>
                </div>
              </DialogContent>
            </Dialog>
          )}

          {selectedProduct && (
            (() => {
              const imageUrls = (selectedProduct.images ?? [])
                .map((img) => img.thumbnailUrl || img.url || "")
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
        <Card className="mt-6 border-[var(--color-line)]">
          <CardTitle className="text-[var(--color-muted)]">
            {editingProductId ? "Edit product" : "Add new product"}
          </CardTitle>
          <CardContent className="mt-6">
            {categoriesError && (
              <div className="mb-4 rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800">
                <p className="font-medium">Can&apos;t load categories.</p>
                <p className="mt-1 text-xs text-red-800/80">
                  {categoriesAuthLike
                    ? "Admin access was denied. Check NEXT_PUBLIC_ADMIN_API_KEY or your admin auth configuration."
                    : categoriesErrorObj?.message ?? "Failed to load categories."}
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
                  Variants (optional)
                </h3>
                <p className="mt-2 text-xs text-[var(--color-muted)]">
                  Add size/color combinations. Each variant can have an extra price (paise) and initial stock. If no variants are added, the product can still be created; you can add variants later.
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
                  Attributes (optional)
                </h3>
                <p className="mt-2 text-xs text-[var(--color-muted)]">
                  Link attributes (e.g. Material = Cotton). Select an existing attribute or enter name + value to create and link.
                </p>
                <div className="mt-3 space-y-2">
                  {attributes.map((attr, idx) => (
                    <div
                      key={idx}
                      className="flex flex-wrap items-end gap-2 rounded-lg border border-[var(--color-line)] bg-white/40 p-3"
                    >
                      <select
                        className={cn(
                          "h-9 min-w-[10rem] rounded-md border border-[var(--color-line)] bg-white px-2 text-sm",
                          "focus:outline-none focus:ring-2 focus:ring-[var(--color-focus)]"
                        )}
                        value={attr.attributeId ?? ""}
                        onChange={(e) => {
                          const opt = existingAttributes.find((a) => a.attributeId === e.target.value);
                          setAttributes((prev) => {
                            const next = [...prev];
                            next[idx] = opt
                              ? { attributeId: opt.attributeId, attributeName: opt.attributeName, attributeValue: opt.attributeValue }
                              : { attributeName: "", attributeValue: "" };
                            return next;
                          });
                        }}
                      >
                        <option value="">New or select…</option>
                        {existingAttributes.map((a) => (
                          <option key={a.attributeId} value={a.attributeId}>
                            {a.attributeName} = {a.attributeValue}
                          </option>
                        ))}
                      </select>
                      <Input
                        type="text"
                        placeholder="Name"
                        value={attr.attributeName}
                        onChange={(e) =>
                          setAttributes((prev) => {
                            const next = [...prev];
                            next[idx] = { ...next[idx], attributeName: e.target.value };
                            return next;
                          })
                        }
                        className="h-9 w-32 rounded-md"
                      />
                      <Input
                        type="text"
                        placeholder="Value"
                        value={attr.attributeValue}
                        onChange={(e) =>
                          setAttributes((prev) => {
                            const next = [...prev];
                            next[idx] = { ...next[idx], attributeValue: e.target.value };
                            return next;
                          })
                        }
                        className="h-9 w-32 rounded-md"
                      />
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        className="h-9 text-red-600"
                        onClick={() => setAttributes((prev) => prev.filter((_, i) => i !== idx))}
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
                    onClick={() => setAttributes((prev) => [...prev, { attributeName: "", attributeValue: "" }])}
                  >
                    + Add attribute
                  </Button>
                </div>
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
                    All selected images will be uploaded when you click{" "}
                    <span className="font-semibold">Add product</span>.
                  </p>
                  {imageFiles.length > 0 && (
                    <p className="text-[10px] text-[var(--color-muted)]">
                      {imageFiles.length} image
                      {imageFiles.length === 1 ? "" : "s"} selected.
                    </p>
                  )}
                </div>
              </div>
              <div className="mt-4 flex items-center gap-2">
                <Button
                  type="submit"
                  disabled={createProductMutation.isPending || updateProductMutation.isPending}
                  className="rounded-lg bg-[var(--color-accent-brown)] hover:bg-[var(--color-accent-brown)]/90"
                >
                  {editingProductId
                    ? updateProductMutation.isPending
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
                      setAttributes([]);
                      setImageFiles([]);
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
              Add your product images
            </p>
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
                      onDrop={() => {
                        setDragIndex(null);
                      }}
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
            {imagePreviews.length > 0 && (
              <p className="text-[11px] text-[var(--color-muted)]">
                {imageFiles.length} image
                {imageFiles.length === 1 ? "" : "s"} selected.
              </p>
            )}
            <div className="flex justify-end gap-2 pt-2">
              <Button
                type="button"
                variant="outline"
                className="rounded-full border-[var(--color-line)] px-4"
                onClick={() => {
                  setImageDialogOpen(false);
                }}
              >
                Cancel
              </Button>
              <Button
                type="button"
                className="rounded-full bg-[var(--color-ink)] px-4 text-white hover:bg-[var(--color-ink)]/90"
                onClick={() => setImageDialogOpen(false)}
              >
                Confirm
              </Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </Section>
  );
}
