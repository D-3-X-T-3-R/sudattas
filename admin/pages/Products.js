import React, { useState, useEffect } from "react";
import { AdminLayout } from "../Layout";

const LIGHT = {
  surface: "#FFFFFF",
  border: "#E7E1D6",
  muted: "#78716c",
  text: "#1c1917",
};

const GRAPHQL_URL =
  (typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_URL) ||
  "http://localhost:8080/v2";

function getAuthHeaders() {
  const headers = { "Content-Type": "application/json" };
  const token =
    typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_TOKEN;
  const sessionId =
    typeof process !== "undefined" && process.env?.REACT_APP_GRAPHQL_SESSION_ID;
  const adminKey =
    typeof process !== "undefined" && process.env?.REACT_APP_ADMIN_API_KEY;
  if (token) headers["Authorization"] = token.startsWith("Bearer ") ? token : `Bearer ${token}`;
  else if (sessionId) headers["X-Session-Id"] = sessionId;
  else if (adminKey) headers["X-Admin-Key"] = adminKey;
  return headers;
}

async function gql(query, variables = {}) {
  const res = await fetch(GRAPHQL_URL, {
    method: "POST",
    headers: getAuthHeaders(),
    body: JSON.stringify({ query, variables }),
  });
  const text = await res.text();
  if (res.status === 401) {
    throw new Error(
      "Unauthorized. Set REACT_APP_ADMIN_API_KEY in the frontend .env to match ADMIN_API_KEY in the backend .env (or use REACT_APP_GRAPHQL_TOKEN / REACT_APP_GRAPHQL_SESSION_ID)."
    );
  }
  if (res.status === 403) {
    throw new Error("Forbidden. Request was rejected (e.g. CSRF or origin check).");
  }
  if (!res.ok) {
    try {
      const json = JSON.parse(text);
      throw new Error(json.message || json.errors?.[0]?.message || `HTTP ${res.status}`);
    } catch (e) {
      if (e instanceof Error && e.message !== "HTTP " + res.status) throw e;
      throw new Error(text || `HTTP ${res.status}`);
    }
  }
  let json;
  try {
    json = JSON.parse(text);
  } catch {
    throw new Error(text || "Invalid response from server.");
  }
  if (json.errors?.length) throw new Error(json.errors.map((e) => e.message).join("; "));
  return json.data;
}

function fetchCategories() {
  return gql(`query { searchCategory(search: {}) { categoryId name } }`).then(
    (data) => data?.searchCategory ?? []
  );
}

export function Products() {
  const [categories, setCategories] = useState([]);
  const [loading, setLoading] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const [showNewCategory, setShowNewCategory] = useState(false);
  const [newCategoryName, setNewCategoryName] = useState("");
  const [addingCategory, setAddingCategory] = useState(false);
  const [categoryError, setCategoryError] = useState("");
  const [form, setForm] = useState({
    name: "",
    description: "",
    priceRupees: "",
    stockQuantity: "0",
    categoryId: "",
  });

  useEffect(() => {
    let cancelled = false;
    fetchCategories()
      .then((list) => {
        if (!cancelled) {
          setCategories(list);
          if (list.length && !form.categoryId) {
            setForm((f) => ({ ...f, categoryId: list[0].categoryId || "" }));
          }
        }
      })
      .catch(() => {});
    return () => { cancelled = true; };
  }, []);

  const handleChange = (e) => {
    const { name, value } = e.target;
    setForm((prev) => ({ ...prev, [name]: value }));
    setError("");
    setMessage("");
  };

  const handleAddCategory = async (e) => {
    e.preventDefault();
    const name = newCategoryName.trim();
    if (!name) {
      setCategoryError("Category name is required.");
      return;
    }
    setCategoryError("");
    setAddingCategory(true);
    try {
      const data = await gql(
        `mutation CreateCategory($category: NewCategory!) {
          createCategory(category: $category) { categoryId name }
        }`,
        { category: { name } }
      );
      const created = data?.createCategory?.[0];
      if (created) {
        const list = await fetchCategories();
        setCategories(list);
        setForm((prev) => ({ ...prev, categoryId: created.categoryId }));
        setNewCategoryName("");
        setShowNewCategory(false);
      }
    } catch (err) {
      setCategoryError(err.message || "Failed to create category.");
    } finally {
      setAddingCategory(false);
    }
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError("");
    setMessage("");
    const name = form.name.trim();
    const description = form.description.trim();
    const priceRupees = form.priceRupees.trim();
    const stockQuantity = form.stockQuantity.trim();
    const categoryId = form.categoryId.trim();

    if (!name) {
      setError("Name is required.");
      return;
    }
    if (!categoryId) {
      setError("Please select a category.");
      return;
    }

    const pricePaise = Math.round(parseFloat(priceRupees || "0") * 100);
    if (isNaN(pricePaise) || pricePaise < 0) {
      setError("Enter a valid price (e.g. 499.00).");
      return;
    }

    setLoading(true);
    try {
      const data = await gql(
        `mutation CreateProduct($product: NewProduct!) {
          createProduct(product: $product) { productId name formatted }
        }`,
        {
          product: {
            name,
            description: description || "",
            pricePaise: String(pricePaise),
            stockQuantity: stockQuantity || "0",
            categoryId,
          },
        }
      );
      const created = data?.createProduct?.[0];
      setMessage(created ? `Created: ${created.name} (${created.formatted})` : "Product created.");
      setForm((prev) => ({
        ...prev,
        name: "",
        description: "",
        priceRupees: "",
        stockQuantity: "0",
      }));
    } catch (err) {
      setError(err.message || "Failed to create product.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <AdminLayout title="Products">
      <div
        className="rounded-xl border p-6 shadow-sm max-w-xl"
        style={{ borderColor: LIGHT.border, background: LIGHT.surface }}
      >
        <h2 className="text-sm font-semibold uppercase tracking-wider" style={{ color: LIGHT.muted }}>
          Add new product
        </h2>
        <form onSubmit={handleSubmit} className="mt-6 space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1" style={{ color: LIGHT.text }}>
              Name *
            </label>
            <input
              type="text"
              name="name"
              value={form.name}
              onChange={handleChange}
              placeholder="e.g. Ivory Silk Saree"
              className="w-full rounded-lg border px-4 py-2.5 text-sm outline-none focus:ring-2"
              style={{ borderColor: LIGHT.border }}
              required
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1" style={{ color: LIGHT.text }}>
              Description
            </label>
            <textarea
              name="description"
              value={form.description}
              onChange={handleChange}
              placeholder="Short description"
              rows={3}
              className="w-full rounded-lg border px-4 py-2.5 text-sm outline-none focus:ring-2 resize-y"
              style={{ borderColor: LIGHT.border }}
            />
          </div>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium mb-1" style={{ color: LIGHT.text }}>
                Price (₹) *
              </label>
              <input
                type="text"
                name="priceRupees"
                value={form.priceRupees}
                onChange={handleChange}
                placeholder="e.g. 499.00"
                className="w-full rounded-lg border px-4 py-2.5 text-sm outline-none focus:ring-2"
                style={{ borderColor: LIGHT.border }}
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1" style={{ color: LIGHT.text }}>
                Stock quantity
              </label>
              <input
                type="number"
                name="stockQuantity"
                value={form.stockQuantity}
                onChange={handleChange}
                min="0"
                className="w-full rounded-lg border px-4 py-2.5 text-sm outline-none focus:ring-2"
                style={{ borderColor: LIGHT.border }}
              />
            </div>
          </div>
          <div>
            <label className="block text-sm font-medium mb-1" style={{ color: LIGHT.text }}>
              Category *
            </label>
            <select
              name="categoryId"
              value={form.categoryId}
              onChange={handleChange}
              className="w-full rounded-lg border px-4 py-2.5 text-sm outline-none focus:ring-2"
              style={{ borderColor: LIGHT.border }}
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
                className="text-sm font-medium underline focus:outline-none"
                style={{ color: "#6B3F1A" }}
              >
                {showNewCategory ? "Cancel" : "+ Add new category"}
              </button>
            </div>
            {showNewCategory && (
              <div
                className="mt-3 flex flex-wrap items-end gap-2 rounded-lg border p-3"
                style={{ borderColor: LIGHT.border }}
              >
                <div className="min-w-0 flex-1">
                  <label className="sr-only">New category name</label>
                  <input
                    type="text"
                    value={newCategoryName}
                    onChange={(e) => {
                      setNewCategoryName(e.target.value);
                      setCategoryError("");
                    }}
                    placeholder="e.g. Silk Sarees"
                    className="w-full rounded-lg border px-3 py-2 text-sm outline-none focus:ring-2"
                    style={{ borderColor: LIGHT.border }}
                    autoFocus
                  />
                </div>
                <button
                  type="button"
                  onClick={handleAddCategory}
                  disabled={addingCategory}
                  className="rounded-lg px-4 py-2 text-sm font-semibold text-white disabled:opacity-50"
                  style={{ background: "#6B3F1A" }}
                >
                  {addingCategory ? "Adding…" : "Add category"}
                </button>
                {categoryError && (
                  <p className="w-full text-sm text-red-600" role="alert">
                    {categoryError}
                  </p>
                )}
              </div>
            )}
            {categories.length === 0 && !showNewCategory && (
              <p className="mt-1 text-xs" style={{ color: LIGHT.muted }}>
                No categories yet. Use "Add new category" above to create one.
              </p>
            )}
          </div>
          {error && (
            <p className="text-sm text-red-600" role="alert">{error}</p>
          )}
          {message && (
            <p className="text-sm" style={{ color: LIGHT.muted }} role="status">{message}</p>
          )}
          <button
            type="submit"
            disabled={loading}
            className="rounded-lg px-5 py-2.5 text-sm font-semibold text-white disabled:opacity-50"
            style={{ background: "#6B3F1A" }}
          >
            {loading ? "Creating…" : "Add product"}
          </button>
        </form>
      </div>
    </AdminLayout>
  );
}
