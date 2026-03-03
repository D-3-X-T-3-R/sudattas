"use client";

import { useState, useEffect } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { gqlAdmin } from "@/lib/graphqlAdmin";
import { adminProductFormSchema } from "@/lib/schemas";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

async function fetchCategories() {
  const data = await gqlAdmin<{ searchCategory?: Array<{ categoryId: string; name: string }> }>(
    `query { searchCategory(search: {}) { categoryId name } }`
  );
  return data?.searchCategory ?? [];
}

export default function AdminProductsPage() {
  const queryClient = useQueryClient();
  const { data: categories = [] } = useQuery({
    queryKey: ["admin", "categories"],
    queryFn: fetchCategories,
  });

  const [form, setForm] = useState({
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
      setMessage(created ? `Created: ${created.name}${created.formatted ? ` (${created.formatted})` : ""}` : "Product created.");
      setForm({
        name: "",
        description: "",
        priceRupees: "",
        stockQuantity: "0",
        categoryId: form.categoryId,
      });
      setError("");
    },
    onError: (err: Error) => setError(err.message || "Failed to create product."),
  });

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => {
    const { name, value } = e.target;
    setForm((prev) => ({ ...prev, [name]: value }));
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
    const parsed = adminProductFormSchema.safeParse(form);
    if (!parsed.success) {
      const first = parsed.error.flatten().fieldErrors;
      const msg = first.name?.[0] ?? first.categoryId?.[0] ?? parsed.error.message;
      setError(msg);
      return;
    }
    const { name, description, priceRupees, stockQuantity, categoryId } = parsed.data;
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

  return (
    <Card className="max-w-xl">
      <CardTitle>Add new product</CardTitle>
      <CardContent className="mt-6">
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
              required
            >
              <option value="">Select category</option>
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
                No categories yet. Use &quot;Add new category&quot; above to create one.
              </p>
            )}
          </div>
          {error && (
            <p className="text-sm text-red-600" role="alert">{error}</p>
          )}
          {message && (
            <p className="text-sm text-[var(--color-muted)]" role="status">
              {message}
            </p>
          )}
          <Button
            type="submit"
            disabled={createProductMutation.isPending}
            className="rounded-lg bg-[var(--color-accent-brown)] hover:bg-[var(--color-accent-brown)]/90"
          >
            {createProductMutation.isPending ? "Creating…" : "Add product"}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
