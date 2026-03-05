"use client";

import { useState, useEffect, useRef } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { gqlAdmin } from "@/lib/graphqlAdmin";
import { adminProductFormSchema } from "@/lib/schemas";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Section } from "@/components/ui/section";
import { Kicker, SectionHeading } from "@/components/ui/typography";
import { cn } from "@/lib/utils";

type ProductFormState = {
  name: string;
  description: string;
  priceRupees: string;
  stockQuantity: string;
  categoryId: string;
};

const DRAFT_KEY = "sudattas_admin_product_draft";

type AdminProductRow = {
  productId: string;
  name: string;
  formatted: string;
  stockQuantity?: string | null;
  categoryId?: string | null;
};

async function fetchCategories(): Promise<
  Array<{ categoryId: string; name: string }>
> {
  const data = await gqlAdmin<{
    searchCategory?: Array<{ categoryId: string; name: string }>;
  }>(`query { searchCategory(search: {}) { categoryId name } }`);
  return data?.searchCategory ?? [];
}

async function fetchProducts(search: {
  name?: string;
  limit?: string;
}): Promise<AdminProductRow[]> {
  const input: Record<string, string> = {};
  if (search.name) input.name = search.name;
  if (search.limit) input.limit = search.limit;

  const data = await gqlAdmin<{
    searchProduct?: AdminProductRow[];
  }>(
    `query SearchProducts($search: SearchProduct!) {
      searchProduct(search: $search) {
        productId
        name
        formatted
        stockQuantity
        categoryId
      }
    }`,
    { search: input }
  );
  return data?.searchProduct ?? [];
}

export default function AdminProductsPage() {
  const queryClient = useQueryClient();
  const {
    data: categories = [],
    isLoading: categoriesLoading,
    isError: categoriesError,
    error: categoriesErrorObj,
    refetch: refetchCategories,
  } = useQuery<Array<{ categoryId: string; name: string }>, Error>({
    queryKey: ["admin", "categories"],
    queryFn: fetchCategories,
  });

  const [searchName, setSearchName] = useState("");
  const [searchLimit, setSearchLimit] = useState("20");
  const [appliedSearch, setAppliedSearch] = useState<{
    name?: string;
    limit?: string;
  }>({ limit: "20" });

  const {
    data: products = [],
    isLoading: productsLoading,
    isError: productsError,
    error: productsErrorObj,
    refetch: refetchProducts,
  } = useQuery<AdminProductRow[], Error>({
    queryKey: ["admin", "products", appliedSearch],
    queryFn: () => fetchProducts(appliedSearch || {}),
    enabled: !!appliedSearch,
  });

  const [form, setForm] = useState<ProductFormState>({
    name: "",
    description: "",
    priceRupees: "",
    stockQuantity: "0",
    categoryId: categories[0]?.categoryId ?? "",
  });
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

  // Load draft from sessionStorage on first mount
  useEffect(() => {
    if (typeof window === "undefined") return;
    try {
      const raw = window.sessionStorage.getItem(DRAFT_KEY);
      if (!raw) return;
      const parsed = JSON.parse(raw) as Partial<ProductFormState>;
      setForm((prev) => ({
        ...prev,
        ...parsed,
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
    }) => {
      const data = await gqlAdmin<{ createProduct?: Array<{ productId: string; name: string; formatted?: string }> }>(
        `mutation CreateProduct($product: NewProduct!) {
          createProduct(product: $product) { productId name formatted }
        }`,
        {
          product: {
            name: payload.name,
            description: payload.description,
            pricePaise: String(payload.pricePaise),
            stockQuantity: payload.stockQuantity,
            categoryId: payload.categoryId,
          },
        }
      );
      return data?.createProduct?.[0];
    },
    onSuccess: (created) => {
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
      }
      setForm((prev) => ({
        name: "",
        description: "",
        priceRupees: "",
        stockQuantity: "0",
        categoryId: prev.categoryId,
      }));
      setError("");
      if (typeof window !== "undefined") {
        window.sessionStorage.removeItem(DRAFT_KEY);
      }
    },
    onError: (err: Error) => setError(err.message || "Failed to create product."),
  });

  const handleChange = (
    e: React.ChangeEvent<
      HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement
    >
  ) => {
    const { name, value } = e.target;
    setForm((prev) => {
      const next = { ...prev, [name]: value };
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

    if (imageFiles.length === 0) {
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
    const { name, description, priceRupees, stockQuantity, categoryId } =
      parsed.data;
    const pricePaise = Math.round(parseFloat(priceRupees || "0") * 100);
    if (isNaN(pricePaise) || pricePaise < 0) {
      setError("Enter a valid price (e.g. 499.00).");
      return;
    }
    createProductMutation.mutate({
      name,
      description: description || "",
      pricePaise,
      stockQuantity: stockQuantity || "0",
      categoryId,
    });
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

  const categoriesAuthLike =
    categoriesError &&
    categoriesErrorObj &&
    /unauthorized|forbidden|admin/i.test(categoriesErrorObj.message);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const next: { name?: string; limit?: string } = {};
    const trimmedName = searchName.trim();
    const trimmedLimit = searchLimit.trim();
    if (trimmedName) next.name = trimmedName;
    if (trimmedLimit) next.limit = trimmedLimit;
    setAppliedSearch(next);
  };

  const handleSearchClear = () => {
    setSearchName("");
    setSearchLimit("20");
    setAppliedSearch({ limit: "20" });
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

  const [activeTab, setActiveTab] = useState<"view" | "add">("view");

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
        <Card className="mt-6 border-[var(--color-line)]">
          <CardTitle className="text-[var(--color-muted)]">View products</CardTitle>
          <CardContent className="mt-4 space-y-4 text-sm text-[var(--color-muted)]">
            <form
              onSubmit={handleSearchSubmit}
              className="flex flex-col gap-3 rounded-lg border border-[var(--color-line)] bg-white/70 p-3 sm:flex-row sm:items-end"
            >
              <div className="flex-1">
                <label className="mb-1 block text-xs font-semibold tracking-[0.18em] text-[var(--color-muted)]">
                  NAME CONTAINS
                </label>
                <Input
                  type="text"
                  value={searchName}
                  onChange={(e) => setSearchName(e.target.value)}
                  placeholder="e.g. silk, ivory"
                  className="h-9 rounded-lg"
                />
              </div>
              <div>
                <label className="mb-1 block text-xs font-semibold tracking-[0.18em] text-[var(--color-muted)]">
                  LIMIT
                </label>
                <Input
                  type="number"
                  min={1}
                  max={50}
                  value={searchLimit}
                  onChange={(e) => setSearchLimit(e.target.value)}
                  className="h-9 w-24 rounded-lg"
                />
              </div>
              <div className="flex gap-2">
                <Button
                  type="submit"
                  className="h-9 rounded-full bg-[var(--color-ink)] px-4 text-xs hover:bg-[var(--color-ink)]/90"
                >
                  Search
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleSearchClear}
                  className="h-9 rounded-full border-[var(--color-line)] px-4 text-xs"
                >
                  Clear
                </Button>
              </div>
            </form>

            {productsError && (
              <div className="rounded-lg border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-800">
                <p className="font-medium">Can&apos;t load products.</p>
                <p className="mt-1 text-xs text-red-800/80">
                  {productsErrorObj?.message ?? "Failed to load products."}
                </p>
                <button
                  type="button"
                  onClick={() => refetchProducts()}
                  className="mt-2 inline-flex items-center rounded-full border border-red-300 bg-white px-3 py-1 text-xs font-medium text-red-800 hover:bg-red-50"
                >
                  Try again
                </button>
              </div>
            )}

            {productsLoading && !productsError && (
              <p className="text-xs text-[var(--color-muted)]">
                Loading products…
              </p>
            )}

            {!productsLoading && !productsError && products.length === 0 && (
              <p className="text-xs text-[var(--color-muted)]">
                No products match your search yet. Once you create products in
                the <strong>Add product</strong> tab, they will appear here.
              </p>
            )}

            {!productsLoading && !productsError && products.length > 0 && (
              <div className="overflow-hidden rounded-lg border border-[var(--color-line)] bg-white">
                <div className="grid grid-cols-[2fr,1fr,1fr] gap-3 border-b border-[var(--color-line)] bg-[var(--color-ivory)] px-3 py-2 text-[10px] font-semibold tracking-[0.16em] text-[var(--color-muted)]">
                  <span>NAME</span>
                  <span className="text-right">PRICE</span>
                  <span className="text-right">STOCK</span>
                </div>
                <ul className="divide-y divide-[var(--color-line)]">
                  {products.map((p) => (
                    <li key={p.productId} className="px-3 py-2.5">
                      <div className="grid grid-cols-[2fr,1fr,1fr] items-baseline gap-3">
                        <div>
                          <div className="text-sm font-medium text-[var(--color-ink)]">
                            {p.name}
                          </div>
                          <div className="mt-0.5 text-[11px] text-[var(--color-muted)]">
                            ID:{" "}
                            <span className="font-mono text-[10px]">
                              {p.productId}
                            </span>
                            {p.categoryId && (
                              <>
                                {" "}
                                • Category:{" "}
                                <span className="font-mono text-[10px]">
                                  {p.categoryId}
                                </span>
                              </>
                            )}
                          </div>
                        </div>
                        <div className="text-right text-sm font-semibold text-[var(--color-ink)]">
                          {p.formatted}
                        </div>
                        <div className="text-right text-xs text-[var(--color-muted)]">
                          {p.stockQuantity ?? "—"}
                        </div>
                      </div>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {activeTab === "add" && (
        <Card className="mt-6 border-[var(--color-line)]">
          <CardTitle className="text-[var(--color-muted)]">Add new product</CardTitle>
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
                <div>
                  <label className="mb-1 block text-sm font-medium text-[var(--color-ink)]">
                    Stock quantity
                  </label>
                  <Input
                    type="number"
                    name="stockQuantity"
                    value={form.stockQuantity}
                    onChange={handleChange}
                    min={0}
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
              <Button
                type="submit"
                disabled={createProductMutation.isPending}
                className="mt-4 rounded-lg bg-[var(--color-accent-brown)] hover:bg-[var(--color-accent-brown)]/90"
              >
                {createProductMutation.isPending ? "Creating…" : "Add product"}
              </Button>
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
