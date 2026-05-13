import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import { ensureGuestSession, getGuestSessionId, clearGuestSession } from "@/lib/session";
import { toRouteFailureUi } from "@/lib/route-state";
import { PRODUCTS_SEED } from "@/lib/seed-data";
import type { Product } from "@/lib/schemas";
import {
  formatInrFromPaise,
  optionalRupeesInputToPaise,
  paiseToRupeesInput,
} from "@/lib/money";

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

export const EXPLORE_SORT_OPTIONS = [
  "Featured",
  "Price: Low to High",
  "Price: High to Low",
  "Name: A-Z",
] as const;

export type ExploreSortOption = (typeof EXPLORE_SORT_OPTIONS)[number];
export type AvailabilityValue = "in-stock" | "out-of-stock";
export type BlouseValue = "included" | "not-included";
export type FilterOption = { value: string; label: string; count: number };
export type ActiveFilterChip = { key: string; label: string; onRemove: () => void };

export type ExploreFilterVisibility = {
  category: boolean;
  price: boolean;
  fabric: boolean;
  occasion: boolean;
  size: boolean;
  craft: boolean;
  blouse: boolean;
  availability: boolean;
};

const SIZE_ORDER = ["XS", "S", "M", "L", "XL", "XXL", "3XL"] as const;

function normalizeSizeName(name: string): string {
  const normalized = name.trim().toUpperCase().replace(/\s+/g, "");
  return normalized === "XXXL" ? "3XL" : normalized;
}

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

function isPublicCatalogName(name: string): boolean {
  const normalized = name.trim().toLowerCase();
  if (!normalized) return false;
  return !/^(itest|test|e2e|mock|seed)[_-]/.test(normalized);
}

function cleanCatalogProducts(products: Product[]): Product[] {
  return products.filter((product) => isPublicCatalogName(product.collection));
}

function buildOptions(
  products: Product[],
  getValue: (product: Product) => string | null | undefined
): FilterOption[] {
  const counts = new Map<string, number>();
  for (const product of products) {
    const value = (getValue(product) ?? "").trim();
    if (!value || !isPublicCatalogName(value)) continue;
    counts.set(value, (counts.get(value) ?? 0) + 1);
  }
  return Array.from(counts.entries())
    .map(([value, count]) => ({ value, label: value, count }))
    .sort((a, b) => a.label.localeCompare(b.label));
}

function productPricePaise(product: Product): number {
  return product.pricePaise ?? Math.max(0, Math.round(product.price * 100));
}

function productKnownStock(product: Product): number | null {
  if (product.variantStock && product.variantStock.length > 0) {
    return product.variantStock.reduce((sum, row) => sum + Math.max(0, row.quantity), 0);
  }
  const stock = product.stockQuantity?.trim();
  if (!stock) return null;
  const parsed = Number.parseInt(stock, 10);
  return Number.isFinite(parsed) ? Math.max(0, parsed) : null;
}

function buildSizeOptions(products: Product[]): FilterOption[] {
  const counts = new Map<string, number>();
  for (const product of products) {
    const sizes = new Set(
      (product.variantStock ?? [])
        .map((row) => normalizeSizeName(row.sizeName))
        .filter((name) => name && name !== "FREESIZE")
    );
    for (const size of sizes) {
      counts.set(size, (counts.get(size) ?? 0) + 1);
    }
  }
  return SIZE_ORDER.filter((size) => counts.has(size)).map((size) => ({
    value: size,
    label: size,
    count: counts.get(size) ?? 0,
  }));
}

function buildAvailabilityOptions(products: Product[]): FilterOption[] {
  let inStock = 0;
  let outOfStock = 0;
  for (const product of products) {
    const stock = productKnownStock(product);
    if (stock == null) continue;
    if (stock > 0) inStock += 1;
    else outOfStock += 1;
  }
  return [
    ...(inStock > 0 ? [{ value: "in-stock", label: "In stock", count: inStock }] : []),
    ...(outOfStock > 0 ? [{ value: "out-of-stock", label: "Out of stock", count: outOfStock }] : []),
  ];
}

function buildBlouseOptions(products: Product[]): FilterOption[] {
  let included = 0;
  let notIncluded = 0;
  for (const product of products) {
    if (product.hasBlousePiece == null) continue;
    if (product.hasBlousePiece) included += 1;
    else notIncluded += 1;
  }
  return [
    ...(included > 0 ? [{ value: "included", label: "Blouse piece included", count: included }] : []),
    ...(notIncluded > 0 ? [{ value: "not-included", label: "No blouse piece", count: notIncluded }] : []),
  ];
}

function hasUsefulOptions(options: FilterOption[], totalProducts: number): boolean {
  if (options.length > 1) return true;
  return options.length === 1 && options[0].count < totalProducts;
}

function toggleValue<T extends string>(values: T[], value: T): T[] {
  return values.includes(value)
    ? values.filter((entry) => entry !== value)
    : [...values, value];
}

function productMatches(product: Product, filters: {
  query: string;
  selectedCategories: string[];
  selectedFabrics: string[];
  selectedOccasions: string[];
  selectedSizes: string[];
  selectedCrafts: string[];
  selectedBlouse: BlouseValue[];
  selectedAvailability: AvailabilityValue[];
  minPricePaise: number | null;
  maxPricePaise: number | null;
}): boolean {
  const normalizedQuery = filters.query.trim().toLowerCase();
  if (normalizedQuery) {
    const haystack = [
      product.name,
      product.collection,
      product.fabric,
      product.occasion,
      product.weave,
    ].join(" ").toLowerCase();
    if (!haystack.includes(normalizedQuery)) return false;
  }
  if (filters.selectedCategories.length > 0 && !filters.selectedCategories.includes(product.collection)) {
    return false;
  }
  if (filters.selectedFabrics.length > 0 && !filters.selectedFabrics.includes(product.fabric)) {
    return false;
  }
  if (filters.selectedOccasions.length > 0 && !filters.selectedOccasions.includes(product.occasion)) {
    return false;
  }
  if (filters.selectedCrafts.length > 0 && !filters.selectedCrafts.includes(product.weave ?? "")) {
    return false;
  }
  if (filters.selectedSizes.length > 0) {
    const productSizes = (product.variantStock ?? [])
      .map((row) => normalizeSizeName(row.sizeName))
      .filter((name) => name && name !== "FREESIZE");
    if (!productSizes.some((size) => filters.selectedSizes.includes(size))) return false;
  }
  if (filters.selectedBlouse.length > 0) {
    const blouse = product.hasBlousePiece == null ? null : product.hasBlousePiece ? "included" : "not-included";
    if (!blouse || !filters.selectedBlouse.includes(blouse)) return false;
  }
  if (filters.selectedAvailability.length > 0) {
    const stock = productKnownStock(product);
    const availability = stock == null ? null : stock > 0 ? "in-stock" : "out-of-stock";
    if (!availability || !filters.selectedAvailability.includes(availability)) return false;
  }
  const pricePaise = productPricePaise(product);
  if (filters.minPricePaise != null && pricePaise < filters.minPricePaise) return false;
  if (filters.maxPricePaise != null && pricePaise > filters.maxPricePaise) return false;
  return true;
}

function sortProducts(products: Product[], sort: ExploreSortOption): Product[] {
  if (sort === "Price: Low to High") {
    return [...products].sort((a, b) => productPricePaise(a) - productPricePaise(b) || a.name.localeCompare(b.name));
  }
  if (sort === "Price: High to Low") {
    return [...products].sort((a, b) => productPricePaise(b) - productPricePaise(a) || a.name.localeCompare(b.name));
  }
  if (sort === "Name: A-Z") {
    return [...products].sort((a, b) => a.name.localeCompare(b.name));
  }
  return products;
}

function hasSelectedValues(values: string[][], hasActivePriceFilter: boolean): boolean {
  return values.some((items) => items.length > 0) || hasActivePriceFilter;
}

function useCatalogData(showToast: (args: ToastArgs) => void) {
  const [products, setProducts] = useState<Product[]>(cleanCatalogProducts(PRODUCTS_SEED));
  const [productsError, setProductsError] = useState<string | null>(null);
  const [productsBannerDismissed, setProductsBannerDismissed] = useState(false);
  const [categories, setCategories] = useState<{ categoryId: string; name: string; thumbnailUrl?: string }[]>([]);
  const [occasions, setOccasions] = useState<{ occasionId: string; occasionName: string }[]>([]);
  const [moods, setMoods] = useState<{ moodId: string; moodName: string; thumbnailUrl?: string }[]>([]);
  const [shopMoodId, setShopMoodId] = useState<string | null>(null);
  const [loadingProducts, setLoadingProducts] = useState(true);
  const didInitialLoadRef = useRef(false);

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
        setProducts(cleanCatalogProducts(productsResponse.products));
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
        setProducts(cleanCatalogProducts(productsResponse.products));
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

      setCategories(filtersResponse.categories.filter((category) => isPublicCatalogName(category.name)));
      setOccasions(filtersResponse.occasions);
      setMoods(filtersResponse.moods);
      setLoadingProducts(false);
    }

    void loadProducts();
  }, [showToast]);

  return {
    products,
    productsError,
    productsBannerDismissed,
    setProductsBannerDismissed,
    categories,
    occasions,
    moods,
    shopMoodId,
    loadingProducts,
    applyShopMoodFilter,
  };
}

type ListSetter<T extends string> = Dispatch<SetStateAction<T[]>>;

function useActiveFilterChips({
  selectedCategories,
  setSelectedCategories,
  selectedFabrics,
  setSelectedFabrics,
  selectedOccasions,
  setSelectedOccasions,
  selectedSizes,
  setSelectedSizes,
  selectedCrafts,
  setSelectedCrafts,
  selectedBlouse,
  setSelectedBlouse,
  selectedAvailability,
  setSelectedAvailability,
  blouseOptions,
  availabilityOptions,
  minPricePaise,
  maxPricePaise,
  setMinPrice,
  setMaxPrice,
}: {
  selectedCategories: string[];
  setSelectedCategories: ListSetter<string>;
  selectedFabrics: string[];
  setSelectedFabrics: ListSetter<string>;
  selectedOccasions: string[];
  setSelectedOccasions: ListSetter<string>;
  selectedSizes: string[];
  setSelectedSizes: ListSetter<string>;
  selectedCrafts: string[];
  setSelectedCrafts: ListSetter<string>;
  selectedBlouse: BlouseValue[];
  setSelectedBlouse: ListSetter<BlouseValue>;
  selectedAvailability: AvailabilityValue[];
  setSelectedAvailability: ListSetter<AvailabilityValue>;
  blouseOptions: FilterOption[];
  availabilityOptions: FilterOption[];
  minPricePaise: number | null;
  maxPricePaise: number | null;
  setMinPrice: (value: string) => void;
  setMaxPrice: (value: string) => void;
}): ActiveFilterChip[] {
  return useMemo(
    () => [
      ...selectedCategories.map((value) => ({
        key: `category-${value}`,
        label: value,
        onRemove: () => setSelectedCategories((current) => current.filter((entry) => entry !== value)),
      })),
      ...selectedFabrics.map((value) => ({
        key: `fabric-${value}`,
        label: value,
        onRemove: () => setSelectedFabrics((current) => current.filter((entry) => entry !== value)),
      })),
      ...selectedOccasions.map((value) => ({
        key: `occasion-${value}`,
        label: value,
        onRemove: () => setSelectedOccasions((current) => current.filter((entry) => entry !== value)),
      })),
      ...selectedSizes.map((value) => ({
        key: `size-${value}`,
        label: value,
        onRemove: () => setSelectedSizes((current) => current.filter((entry) => entry !== value)),
      })),
      ...selectedCrafts.map((value) => ({
        key: `craft-${value}`,
        label: value,
        onRemove: () => setSelectedCrafts((current) => current.filter((entry) => entry !== value)),
      })),
      ...selectedBlouse.map((value) => ({
        key: `blouse-${value}`,
        label: blouseOptions.find((option) => option.value === value)?.label ?? value,
        onRemove: () => setSelectedBlouse((current) => current.filter((entry) => entry !== value)),
      })),
      ...selectedAvailability.map((value) => ({
        key: `availability-${value}`,
        label: availabilityOptions.find((option) => option.value === value)?.label ?? value,
        onRemove: () => setSelectedAvailability((current) => current.filter((entry) => entry !== value)),
      })),
      ...(minPricePaise != null || maxPricePaise != null
        ? [
            {
              key: "price",
              label: `Price: ${minPricePaise != null ? formatInrFromPaise(minPricePaise) : "Any"} - ${maxPricePaise != null ? formatInrFromPaise(maxPricePaise) : "Any"}`,
              onRemove: () => {
                setMinPrice("");
                setMaxPrice("");
              },
            },
          ]
        : []),
    ],
    [
      availabilityOptions,
      blouseOptions,
      maxPricePaise,
      minPricePaise,
      selectedAvailability,
      selectedBlouse,
      selectedCategories,
      selectedCrafts,
      selectedFabrics,
      selectedOccasions,
      selectedSizes,
      setMaxPrice,
      setMinPrice,
      setSelectedAvailability,
      setSelectedBlouse,
      setSelectedCategories,
      setSelectedCrafts,
      setSelectedFabrics,
      setSelectedOccasions,
      setSelectedSizes,
    ]
  );
}

function useExploreFilterOptions(products: Product[], selectedCategories: string[]) {
  const categoryOptions = useMemo(
    () => buildOptions(products, (product) => product.collection),
    [products]
  );
  const fabricOptions = useMemo(
    () => buildOptions(products, (product) => product.fabric),
    [products]
  );
  const occasionFilterOptions = useMemo(
    () => buildOptions(products, (product) => product.occasion),
    [products]
  );
  const craftOptions = useMemo(
    () => buildOptions(products, (product) => product.weave),
    [products]
  );
  const sizeOptions = useMemo(() => buildSizeOptions(products), [products]);
  const availabilityOptions = useMemo(() => buildAvailabilityOptions(products), [products]);
  const blouseScopeProducts = useMemo(
    () =>
      selectedCategories.length > 0
        ? products.filter((product) => selectedCategories.includes(product.collection))
        : [],
    [products, selectedCategories]
  );
  const blouseOptions = useMemo(() => buildBlouseOptions(blouseScopeProducts), [blouseScopeProducts]);
  const priceBounds = useMemo(() => {
    const prices = products.map(productPricePaise).filter((price) => price >= 0);
    return {
      min: prices.length > 0 ? Math.min(...prices) : 0,
      max: prices.length > 0 ? Math.max(...prices) : 0,
      hasPrices: prices.length > 0,
    };
  }, [products]);
  const filterVisibility: ExploreFilterVisibility = {
    category: hasUsefulOptions(categoryOptions, products.length),
    price: priceBounds.hasPrices,
    fabric: hasUsefulOptions(fabricOptions, products.length),
    occasion: hasUsefulOptions(occasionFilterOptions, products.length),
    size: hasUsefulOptions(sizeOptions, products.length),
    craft: hasUsefulOptions(craftOptions, products.length),
    blouse:
      blouseScopeProducts.some((product) => product.collection.toLowerCase().includes("saree")) &&
      hasUsefulOptions(blouseOptions, blouseScopeProducts.length),
    availability: hasUsefulOptions(availabilityOptions, products.length),
  };

  return {
    categoryOptions,
    fabricOptions,
    occasionFilterOptions,
    craftOptions,
    sizeOptions,
    availabilityOptions,
    blouseOptions,
    priceBounds,
    filterVisibility,
  };
}

function useExploreFilters(
  products: Product[],
  shopMoodId: string | null,
  applyShopMoodFilter: (nextMoodId: string | null) => Promise<void>
) {
  const [query, setQuery] = useState("");
  const [selectedCategories, setSelectedCategories] = useState<string[]>([]);
  const [selectedFabrics, setSelectedFabrics] = useState<string[]>([]);
  const [selectedOccasions, setSelectedOccasions] = useState<string[]>([]);
  const [selectedSizes, setSelectedSizes] = useState<string[]>([]);
  const [selectedCrafts, setSelectedCrafts] = useState<string[]>([]);
  const [selectedBlouse, setSelectedBlouse] = useState<BlouseValue[]>([]);
  const [selectedAvailability, setSelectedAvailability] = useState<AvailabilityValue[]>([]);
  const [minPrice, setMinPrice] = useState("");
  const [maxPrice, setMaxPrice] = useState("");
  const [sort, setSort] = useState<ExploreSortOption>("Featured");

  const setCollection = useCallback((category: string) => {
    setSelectedCategories(category === "All" ? [] : [category]);
  }, []);

  const setOccasion = useCallback((occasion: string) => {
    setSelectedOccasions(occasion === "All" ? [] : [occasion]);
  }, []);

  const {
    categoryOptions,
    fabricOptions,
    occasionFilterOptions,
    craftOptions,
    sizeOptions,
    availabilityOptions,
    blouseOptions,
    priceBounds,
    filterVisibility,
  } = useExploreFilterOptions(products, selectedCategories);

  const minPricePaise = optionalRupeesInputToPaise(minPrice);
  const maxPricePaise = optionalRupeesInputToPaise(maxPrice);
  const selectedMinPricePaise = minPricePaise ?? priceBounds.min;
  const selectedMaxPricePaise = maxPricePaise ?? priceBounds.max;
  const hasActivePriceFilter =
    (minPricePaise != null && minPricePaise > priceBounds.min) ||
    (maxPricePaise != null && maxPricePaise < priceBounds.max);

  const setPriceRangePaise = useCallback(
    (nextMinPaise: number, nextMaxPaise: number) => {
      const boundedMin = Math.max(priceBounds.min, Math.min(nextMinPaise, priceBounds.max));
      const boundedMax = Math.max(priceBounds.min, Math.min(nextMaxPaise, priceBounds.max));
      const clampedMin = Math.min(boundedMin, boundedMax);
      const clampedMax = Math.max(boundedMin, boundedMax);
      setMinPrice(clampedMin <= priceBounds.min ? "" : paiseToRupeesInput(clampedMin));
      setMaxPrice(clampedMax >= priceBounds.max ? "" : paiseToRupeesInput(clampedMax));
    },
    [priceBounds.max, priceBounds.min]
  );

  const filtered = useMemo(() => {
    const next = products.filter((product) =>
      productMatches(product, {
        query,
        selectedCategories,
        selectedFabrics,
        selectedOccasions,
        selectedSizes,
        selectedCrafts,
        selectedBlouse,
        selectedAvailability,
        minPricePaise,
        maxPricePaise,
      })
    );
    return sortProducts(next, sort);
  }, [
    maxPricePaise,
    minPricePaise,
    products,
    query,
    selectedAvailability,
    selectedBlouse,
    selectedCategories,
    selectedCrafts,
    selectedFabrics,
    selectedOccasions,
    selectedSizes,
    sort,
  ]);

  const hasActiveFilters = hasSelectedValues(
    [selectedCategories, selectedFabrics, selectedOccasions, selectedSizes, selectedCrafts, selectedBlouse, selectedAvailability],
    hasActivePriceFilter
  );

  const resetFilters = useCallback(() => {
    setSelectedCategories([]);
    setSelectedFabrics([]);
    setSelectedOccasions([]);
    setSelectedSizes([]);
    setSelectedCrafts([]);
    setSelectedBlouse([]);
    setSelectedAvailability([]);
    setMinPrice("");
    setMaxPrice("");
    setQuery("");
    if (shopMoodId) {
      void applyShopMoodFilter(null);
    }
  }, [applyShopMoodFilter, shopMoodId]);

  const activeFilterChips = useActiveFilterChips({
    selectedCategories,
    setSelectedCategories,
    selectedFabrics,
    setSelectedFabrics,
    selectedOccasions,
    setSelectedOccasions,
    selectedSizes,
    setSelectedSizes,
    selectedCrafts,
    setSelectedCrafts,
    selectedBlouse,
    setSelectedBlouse,
    selectedAvailability,
    setSelectedAvailability,
    blouseOptions,
    availabilityOptions,
    minPricePaise: hasActivePriceFilter ? selectedMinPricePaise : null,
    maxPricePaise: hasActivePriceFilter ? selectedMaxPricePaise : null,
    setMinPrice,
    setMaxPrice,
  });

  const collectionOptions = useMemo(
    () => ["All", ...categoryOptions.map((option) => option.value)],
    [categoryOptions]
  );

  const occasionOptions = useMemo(
    () => ["All", ...occasionFilterOptions.map((option) => option.value)],
    [occasionFilterOptions]
  );

  return {
    query,
    setQuery,
    collection: selectedCategories.length === 1 ? selectedCategories[0] : "All",
    setCollection,
    occasion: selectedOccasions.length === 1 ? selectedOccasions[0] : "All",
    setOccasion,
    sort,
    setSort,
    filtered,
    collectionOptions,
    occasionOptions,
    categoryOptions,
    fabricOptions,
    occasionFilterOptions,
    sizeOptions,
    craftOptions,
    blouseOptions,
    availabilityOptions,
    priceBounds,
    minPrice,
    setMinPrice,
    maxPrice,
    setMaxPrice,
    selectedMinPricePaise,
    selectedMaxPricePaise,
    setPriceRangePaise,
    selectedCategories,
    setSelectedCategories,
    selectedFabrics,
    setSelectedFabrics,
    selectedOccasions,
    setSelectedOccasions,
    selectedSizes,
    setSelectedSizes,
    selectedCrafts,
    setSelectedCrafts,
    selectedBlouse,
    setSelectedBlouse,
    selectedAvailability,
    setSelectedAvailability,
    filterVisibility,
    activeFilterChips,
    hasActiveFilters,
    resetFilters,
    toggleValue,
  };
}

export function useStorefrontCatalog({ showToast }: UseStorefrontCatalogProps) {
  const catalogData = useCatalogData(showToast);
  const filters = useExploreFilters(
    catalogData.products,
    catalogData.shopMoodId,
    catalogData.applyShopMoodFilter
  );

  return {
    ...filters,
    ...catalogData,
  };
}

export type StorefrontCatalogController = ReturnType<typeof useStorefrontCatalog>;
