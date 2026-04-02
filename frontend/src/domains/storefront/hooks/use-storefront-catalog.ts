import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ensureGuestSession, getGuestSessionId, clearGuestSession } from "@/lib/session";
import { toRouteFailureUi } from "@/lib/route-state";
import { PRODUCTS_SEED } from "@/lib/seed-data";
import type { Product } from "@/lib/schemas";

type ProductsResponse = { products: Product[]; error: string | null };

type StorefrontFiltersResponse = {
  categories: { categoryId: string; name: string; thumbnailUrl?: string }[];
  occasions: { occasionId: string; occasionName: string }[];
  moods: { moodId: string; moodName: string; thumbnailUrl?: string }[];
  error: string | null;
};

type ToastArgs = { title: string; description: string };

type UseStorefrontCatalogProps = {
  showToast: (args: ToastArgs) => void;
};

async function fetchStorefrontProducts(
  sessionId: string | null,
  moodId?: string | null
): Promise<ProductsResponse> {
  try {
    const headers: Record<string, string> = {};
    if (sessionId) headers["X-Session-Id"] = sessionId;
    const query = moodId && moodId.trim() !== "" ? `?moodId=${encodeURIComponent(moodId.trim())}` : "";
    const res = await fetch(`/api/products${query}`, { headers });
    const data = (await res.json()) as ProductsResponse | Product[];
    if (Array.isArray(data)) {
      return { products: data, error: null };
    }
    return {
      products: data.products ?? [],
      error: data.error ?? (res.ok ? null : "Request failed"),
    };
  } catch {
    return { products: [], error: "Network error" };
  }
}

async function fetchStorefrontFilters(sessionId: string | null): Promise<StorefrontFiltersResponse> {
  try {
    const headers: Record<string, string> = {};
    if (sessionId) headers["X-Session-Id"] = sessionId;
    const res = await fetch("/api/storefront-filters", { headers });
    const data = (await res.json()) as StorefrontFiltersResponse;
    return {
      categories: data.categories ?? [],
      occasions: data.occasions ?? [],
      moods: data.moods ?? [],
      error: data.error ?? null,
    };
  } catch {
    return { categories: [], occasions: [], moods: [], error: "Network error" };
  }
}

function looksLikeBadSession(msg: string | null): boolean {
  return !!msg && (msg.includes("Session invalid") || msg.includes("Session not found") || msg.includes("expired") || msg.includes("Unauthorized"));
}

export function useStorefrontCatalog({ showToast }: UseStorefrontCatalogProps) {
  const [query, setQuery] = useState("");
  const [collection, setCollection] = useState("All");
  const [occasion, setOccasion] = useState("All");
  const [sort, setSort] = useState("Featured");
  const [products, setProducts] = useState<Product[]>(PRODUCTS_SEED);
  const [productsError, setProductsError] = useState<string | null>(null);
  const [productsBannerDismissed, setProductsBannerDismissed] = useState(false);
  const [categories, setCategories] = useState<{ categoryId: string; name: string; thumbnailUrl?: string }[]>([]);
  const [occasions, setOccasions] = useState<{ occasionId: string; occasionName: string }[]>([]);
  const [moods, setMoods] = useState<{ moodId: string; moodName: string; thumbnailUrl?: string }[]>([]);
  const [shopMoodId, setShopMoodId] = useState<string | null>(null);
  const [loadingProducts, setLoadingProducts] = useState(true);
  const didInitialLoadRef = useRef(false);

  useEffect(() => {
    if (didInitialLoadRef.current) return;
    didInitialLoadRef.current = true;

    async function loadProducts() {
      setLoadingProducts(true);
      await ensureGuestSession();
      let sessionId = getGuestSessionId();

      const loadCatalog = async (sid: string | null, mood: string | null) => {
        const [productsResponse, filtersResponse] = await Promise.all([
          fetchStorefrontProducts(sid, mood),
          fetchStorefrontFilters(sid),
        ]);
        return { productsResponse, filtersResponse };
      };

      let { productsResponse, filtersResponse } = await loadCatalog(sessionId, null);

      if ((productsResponse.error || filtersResponse.error) && (looksLikeBadSession(productsResponse.error) || looksLikeBadSession(filtersResponse.error) || !sessionId)) {
        clearGuestSession();
        await ensureGuestSession();
        sessionId = getGuestSessionId();
        if (sessionId) {
          ({ productsResponse, filtersResponse } = await loadCatalog(sessionId, null));
        }
      }

      if (productsResponse.products.length > 0) {
        setProducts(productsResponse.products);
        setProductsError(null);
      } else if (productsResponse.error) {
        setProductsError(toRouteFailureUi("public", new Error(productsResponse.error)).message);
        showToast({
          title: "Catalog",
          description: "Having trouble connecting. Your bag is saved on this device.",
        });
      } else {
        setProducts([]);
        setProductsError(null);
      }

      setCategories(filtersResponse.categories);
      setOccasions(filtersResponse.occasions);
      setMoods(filtersResponse.moods);
      setLoadingProducts(false);
    }

    void loadProducts();
  }, [showToast]);

  const applyShopMoodFilter = useCallback(
    async (nextMoodId: string | null) => {
      setShopMoodId(nextMoodId);
      setLoadingProducts(true);
      await ensureGuestSession();
      let sid = getGuestSessionId();
      let productsResponse = await fetchStorefrontProducts(sid, nextMoodId);
      if (sid && looksLikeBadSession(productsResponse.error)) {
        clearGuestSession();
        await ensureGuestSession();
        sid = getGuestSessionId();
        productsResponse = await fetchStorefrontProducts(sid, nextMoodId);
      }

      if (productsResponse.products.length > 0) {
        setProducts(productsResponse.products);
        setProductsError(null);
      } else if (productsResponse.error) {
        setProductsError(toRouteFailureUi("public", new Error(productsResponse.error)).message);
        showToast({
          title: "Catalog",
          description: "Having trouble connecting. Your bag is saved on this device.",
        });
      } else {
        setProducts([]);
        setProductsError(null);
      }

      setLoadingProducts(false);
    },
    [showToast]
  );

  const occasionOptions = useMemo(() => {
    const fromProducts = new Set(products.map((product) => product.occasion).filter(Boolean));
    if (occasions.length > 0) {
      return ["All", ...occasions.map((item) => item.occasionName).filter((name) => fromProducts.has(name))];
    }
    return ["All", ...Array.from(fromProducts)];
  }, [occasions, products]);

  const collectionOptions = useMemo(() => {
    const fromProducts = new Set(products.map((product) => product.collection).filter(Boolean));
    if (categories.length > 0) {
      return ["All", ...categories.map((item) => item.name).filter((name) => fromProducts.has(name))];
    }
    return ["All", ...Array.from(fromProducts)];
  }, [categories, products]);

  const filtered = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    let filteredProducts = products.filter((product) => {
      const matchesQuery =
        !normalizedQuery ||
        [product.name, product.collection, product.fabric, product.occasion].join(" ").toLowerCase().includes(normalizedQuery);
      const matchesCollection = collection === "All" || product.collection === collection;
      const matchesOccasion = occasion === "All" || product.occasion === occasion;
      return matchesQuery && matchesCollection && matchesOccasion;
    });

    if (sort === "Price: Low") filteredProducts = [...filteredProducts].sort((a, b) => a.price - b.price);
    if (sort === "Price: High") filteredProducts = [...filteredProducts].sort((a, b) => b.price - a.price);
    if (sort === "Rating") filteredProducts = [...filteredProducts].sort((a, b) => b.rating - a.rating);
    if (sort === "Latest") {
      filteredProducts = [...filteredProducts].sort(
        (a, b) => parseInt(b.id, 10) - parseInt(a.id, 10) || b.id.localeCompare(a.id)
      );
    }
    return filteredProducts;
  }, [products, query, collection, occasion, sort]);

  return {
    query,
    setQuery,
    collection,
    setCollection,
    occasion,
    setOccasion,
    sort,
    setSort,
    products,
    productsError,
    productsBannerDismissed,
    setProductsBannerDismissed,
    categories,
    occasions,
    moods,
    shopMoodId,
    loadingProducts,
    filtered,
    collectionOptions,
    occasionOptions,
    applyShopMoodFilter,
  };
}
