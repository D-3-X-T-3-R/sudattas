/**
 * Admin dashboard GraphQL queries. Use gqlAdmin (Bearer / admin key) only.
 * For storefront (guest session) use @/lib/storefront-queries.
 */

import { gqlAdmin } from "./graphql-client";
import { formatInrFromPaise } from "./money";
import type {
  CategoryRow,
  OccasionRow,
  ProductListRow,
  ProductListRowWithVariantStock,
  ProductImageListItem,
} from "./graphql-types";

export type { CategoryRow, OccasionRow, ProductListRow, ProductListRowWithVariantStock, ProductImageListItem };

export interface OrderRow {
  orderId: string;
  totalAmountPaise: string;
  totalAmountFormatted: string;
  orderDate: string;
}

/** Order row for admin list: includes userId and statusId for filters and display */
export interface OrderListRow extends OrderRow {
  userId: string;
  statusId: string;
}

export interface ProductRow {
  productId: string;
  name: string;
}

const CATEGORIES_QUERY = `query Categories { searchCategory(search: {}) { categoryId name exchangeEligible } }`;
const PRODUCTS_QUERY = `query SearchProductsList($search: SearchProduct!) {
  searchProduct(search: $search) {
    productId name description amountPaise formatted stockQuantity categoryId
    sku slug fabric weave occasion hasBlousePiece careInstructions productStatusId metaTitle metaDescription
    images { imageId thumbnailUrl url }
    variantStock { variantId sizeId sizeName quantity }
  }
}`;

const PRODUCT_BY_ID_QUERY = `query ProductById($search: SearchProduct!) {
  searchProduct(search: $search) {
    productId name description amountPaise formatted stockQuantity categoryId
    sku slug fabric weave occasion hasBlousePiece careInstructions productStatusId metaTitle metaDescription
    images { imageId thumbnailUrl url }
    variantStock { sizeId sizeName quantity }
  }
}`;

const ORDER_PAGE_SIZE = 50;
const PRODUCT_PAGE_SIZE = 50;

const PRODUCT_COUNT_QUERY = `query ProductCountPage($search: SearchProduct!) {
  searchProduct(search: $search) { productId }
}`;

function toPositiveInt(value: string | undefined, fallback: number): number {
  if (!value) return fallback;
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed < 1) return fallback;
  return parsed;
}

function normalizeLimit(limit: string | undefined): string {
  const n = toPositiveInt(limit, ORDER_PAGE_SIZE);
  return String(Math.min(n, ORDER_PAGE_SIZE));
}

function normalizeOffset(offset: string | undefined): string | undefined {
  if (!offset) return undefined;
  const parsed = Number.parseInt(offset, 10);
  if (!Number.isFinite(parsed) || parsed < 0) return "0";
  return String(parsed);
}

/** Fetch all categories for admin dropdowns. */
export async function fetchCategories(): Promise<CategoryRow[]> {
  const data = await gqlAdmin<{ searchCategory?: CategoryRow[] }>(CATEGORIES_QUERY);
  return data?.searchCategory ?? [];
}

/** Fetch products for admin list with optional filters. */
export async function fetchProductsList(params: {
  name?: string;
  categoryId?: string;
  fabric?: string;
  weave?: string;
  occasion?: string;
  limit?: string;
  productStatusId?: string;
  startingPricePaise?: string;
  endingPricePaise?: string;
  moodId?: string;
}): Promise<ProductListRowWithVariantStock[]> {
  const search: Record<string, string> = {};
  if (params.name) search.name = params.name;
  if (params.categoryId) search.categoryId = params.categoryId;
  if (params.fabric) search.fabric = params.fabric;
  if (params.weave) search.weave = params.weave;
  if (params.occasion) search.occasion = params.occasion;
  if (params.limit) search.limit = normalizeLimit(params.limit);
  if (params.productStatusId) search.productStatusId = params.productStatusId;
  if (params.startingPricePaise) search.startingPricePaise = params.startingPricePaise;
  if (params.endingPricePaise) search.endingPricePaise = params.endingPricePaise;
  if (params.moodId) search.moodId = params.moodId;

  const variables = { search: Object.keys(search).length ? search : { limit: "50" } };
  const data = await gqlAdmin<{ searchProduct?: ProductListRowWithVariantStock[] }>(
    PRODUCTS_QUERY,
    variables
  );
  const raw = data?.searchProduct ?? [];
  return raw.map(normalizeProductImages);
}

/** Ensure each product's images have url/imageId/thumbnailUrl in a consistent shape (camelCase). */
function normalizeProductImages(p: ProductListRowWithVariantStock): ProductListRowWithVariantStock {
  const rawImages = p.images;
  if (!rawImages?.length) return p;
  const images: ProductImageListItem[] = rawImages.map((img: unknown) => {
    const o = img as Record<string, unknown>;
    const url =
      (o.url as string) ?? (o.thumbnailUrl as string) ?? (o.thumbnail_url as string) ?? "";
    const imageId = (o.imageId as string) ?? (o.image_id as string) ?? "";
    const thumbnailUrl = (o.thumbnailUrl as string) ?? (o.thumbnail_url as string) ?? url;
    return { imageId: imageId || undefined, thumbnailUrl: thumbnailUrl || null, url: url || null };
  });
  return { ...p, images };
}

/** Fetch a single product by id with variant stock (for storefront detail when no session). */
export async function fetchProductById(
  productId: string
): Promise<ProductListRowWithVariantStock | null> {
  const data = await gqlAdmin<{ searchProduct?: ProductListRowWithVariantStock[] }>(
    PRODUCT_BY_ID_QUERY,
    { search: { productId, limit: "1" } }
  );
  const list = data?.searchProduct ?? [];
  const first = list[0];
  return first ? normalizeProductImages(first) as ProductListRowWithVariantStock : null;
}

/** Delete a product by ID. Returns remaining products (backend returns deleted product in list). */
export async function deleteProduct(productId: string): Promise<void> {
  await gqlAdmin<{ deleteProduct?: unknown[] }>(
    `mutation DeleteProduct($productId: String!) {
      deleteProduct(productId: $productId) { productId }
    }`,
    { productId }
  );
}

/** Search product images (one row per image). Filter by productId to load images for a product. */
export async function searchProductImage(params: {
  productId?: string;
}): Promise<{ imageId: string; productId: string; url?: string | null }[]> {
  const data = await gqlAdmin<{
    searchProductImage?: Array<{ imageId: string; productId: string; url?: string | null }>;
  }>(
    `query SearchProductImage($search: SearchProductImage!) {
      searchProductImage(search: $search) { imageId productId url }
    }`,
    { search: { productId: params.productId ?? undefined } }
  );
  return data?.searchProductImage ?? [];
}

/** Delete a product image by id. */
export async function deleteProductImage(imageId: string): Promise<void> {
  await gqlAdmin<{ deleteProductImage?: unknown[] }>(
    `mutation DeleteProductImage($imageId: String!) {
      deleteProductImage(imageId: $imageId) { imageId productId }
    }`,
    { imageId }
  );
}

/** Size for variant dropdowns (id "0" returns all) */
export interface SizeRow {
  sizeId: string;
  sizeName: string;
}

export async function fetchSizes(): Promise<SizeRow[]> {
  const data = await gqlAdmin<{ searchSize?: SizeRow[] }>(
    `mutation Sizes { searchSize(input: { sizeId: "0" }) { sizeId sizeName } }`
  );
  return data?.searchSize ?? [];
}

/** Color for variant dropdowns (id "0" returns all) */
export interface ColorRow {
  colorId: string;
  colorName: string;
}

export async function fetchColors(): Promise<ColorRow[]> {
  const data = await gqlAdmin<{ searchColor?: ColorRow[] }>(
    `mutation Colors { searchColor(input: { colorId: "0" }) { colorId colorName } }`
  );
  return data?.searchColor ?? [];
}

/** Fabric options for products (id "0" returns all) */
export interface FabricRow {
  fabricId: string;
  fabricName: string;
}

export async function fetchFabrics(): Promise<FabricRow[]> {
  const data = await gqlAdmin<{ searchFabric?: FabricRow[] }>(
    `mutation Fabrics { searchFabric(input: { fabricId: "0" }) { fabricId fabricName } }`
  );
  return data?.searchFabric ?? [];
}

/** Weave options for products (id "0" returns all) */
export interface WeaveRow {
  weaveId: string;
  weaveName: string;
}

export async function fetchWeaves(): Promise<WeaveRow[]> {
  const data = await gqlAdmin<{ searchWeave?: WeaveRow[] }>(
    `mutation Weaves { searchWeave(input: { weaveId: "0" }) { weaveId weaveName } }`
  );
  return data?.searchWeave ?? [];
}

const OCCASIONS_MUTATION = `mutation Occasions { searchOccasion(input: { occasionId: "0" }) { occasionId occasionName } }`;

export async function fetchOccasions(): Promise<OccasionRow[]> {
  const data = await gqlAdmin<{ searchOccasion?: OccasionRow[] }>(OCCASIONS_MUTATION);
  return data?.searchOccasion ?? [];
}

/** Product mood (id + name) for linking to products */
export interface ProductMoodRow {
  moodId: string;
  moodName: string;
}

export async function searchProductMoods(params?: {
  moodId?: string;
  moodName?: string;
}): Promise<ProductMoodRow[]> {
  const input: Record<string, string> = {};
  if (params?.moodId) input.moodId = params.moodId;
  if (params?.moodName) input.moodName = params.moodName;
  const data = await gqlAdmin<{ searchProductMood?: ProductMoodRow[] }>(
    `query SearchProductMoods($input: SearchProductMoodInput!) {
      searchProductMood(input: $input) { moodId moodName }
    }`,
    { input: Object.keys(input).length ? input : {} }
  );
  return data?.searchProductMood ?? [];
}

/** Moods from newest products (storefront; backend walks recent products for distinct mood ids). */
export async function fetchShopHighlightMoods(opts?: {
  recentProductLimit?: number;
  maxMoods?: number;
}): Promise<ProductMoodRow[]> {
  const data = await gqlAdmin<{ shopHighlightMoods?: ProductMoodRow[] }>(
    `query ShopHighlightMoods($recentProductLimit: Int, $maxMoods: Int) {
      shopHighlightMoods(recentProductLimit: $recentProductLimit, maxMoods: $maxMoods) {
        moodId moodName
      }
    }`,
    {
      recentProductLimit: opts?.recentProductLimit ?? 100,
      maxMoods: opts?.maxMoods ?? 12,
    }
  );
  return data?.shopHighlightMoods ?? [];
}

/** Create a new product mood. Returns the created mood (with moodId and moodName). */
export async function createProductMood(moodName: string): Promise<ProductMoodRow | null> {
  const name = moodName?.trim();
  if (!name) return null;
  const data = await gqlAdmin<{ createProductMood?: ProductMoodRow[] }>(
    `mutation CreateProductMood($input: NewProductMood!) {
      createProductMood(input: $input) { moodId moodName }
    }`,
    { input: { moodName: name } }
  );
  const list = data?.createProductMood ?? [];
  return list.length > 0 ? list[0] : null;
}

/** List mood mappings for a product (for edit form). Returns array of { productId, moodId }. */
export async function searchProductMoodMappingsByProduct(
  productId: string
): Promise<{ productId: string; moodId: string }[]> {
  const data = await gqlAdmin<{ searchProductMoodMapping?: { productId: string; moodId: string }[] }>(
    `mutation SearchProductMoodMappings($input: SearchProductMoodMappingInput!) {
      searchProductMoodMapping(input: $input) { productId moodId }
    }`,
    { input: { productId } }
  );
  return data?.searchProductMoodMapping ?? [];
}

/** Link a product to a mood. */
export async function createProductMoodMapping(
  productId: string,
  moodId: string
): Promise<void> {
  await gqlAdmin<{ createProductMoodMapping?: unknown[] }>(
    `mutation CreateProductMoodMapping($input: NewProductMoodMapping!) {
      createProductMoodMapping(input: $input) { productId moodId }
    }`,
    { input: { productId, moodId } }
  );
}

/** Unlink a product from a mood. */
export async function deleteProductMoodMapping(
  productId: string,
  moodId: string
): Promise<void> {
  await gqlAdmin<{ deleteProductMoodMapping?: unknown[] }>(
    `mutation DeleteProductMoodMapping($input: DeleteProductMoodMappingInput!) {
      deleteProductMoodMapping(input: $input) { productId moodId }
    }`,
    { input: { productId, moodId } }
  );
}

/** Create a product variant. Returns created variant with variantId. */
export interface ProductVariantRow {
  variantId: string;
  productId: string;
  sizeId?: string | null;
  colorId?: string | null;
  additionalPricePaise?: string | null;
}

export async function createProductVariant(params: {
  productId: string;
  sizeId?: string;
  colorId?: string;
  additionalPricePaise?: string;
}): Promise<ProductVariantRow | null> {
  const input: Record<string, string> = { productId: params.productId };
  if (params.sizeId) input.sizeId = params.sizeId;
  if (params.colorId) input.colorId = params.colorId;
  if (params.additionalPricePaise != null) input.additionalPricePaise = params.additionalPricePaise;
  const data = await gqlAdmin<{ createProductVariant?: ProductVariantRow[] }>(
    `mutation CreateProductVariant($input: NewProductVariant!) {
      createProductVariant(input: $input) { variantId productId sizeId colorId additionalPricePaise }
    }`,
    { input }
  );
  return data?.createProductVariant?.[0] ?? null;
}

/** Update a product variant (size/color/additional price). */
export async function updateProductVariant(params: {
  variantId: string;
  productId?: string;
  sizeId?: string;
  colorId?: string;
  additionalPricePaise?: string;
}): Promise<void> {
  const input: Record<string, string> = { variantId: params.variantId };
  if (params.productId) input.productId = params.productId;
  if (params.sizeId !== undefined) input.sizeId = params.sizeId;
  if (params.colorId !== undefined) input.colorId = params.colorId;
  if (params.additionalPricePaise !== undefined) {
    input.additionalPricePaise = params.additionalPricePaise;
  }
  await gqlAdmin<{ updateProductVariant?: unknown[] }>(
    `mutation UpdateProductVariant($input: ProductVariantMutation!) {
      updateProductVariant(input: $input) {
        variantId
        productId
        sizeId
        colorId
        additionalPricePaise
      }
    }`,
    { input }
  );
}

/** Delete a variant by id. */
export async function deleteProductVariant(variantId: string): Promise<void> {
  await gqlAdmin<{ deleteProductVariant?: unknown[] }>(
    `mutation DeleteProductVariant($input: DeleteProductVariantInput!) {
      deleteProductVariant(input: $input) { variantId }
    }`,
    { input: { variantId } }
  );
}

/** Create inventory for a variant (quantity and optional reorder level). */
export async function createInventoryItem(params: {
  variantId: string;
  quantityAvailable: string;
  reorderLevel?: string;
}): Promise<void> {
  const input: Record<string, string> = {
    variantId: params.variantId,
    quantityAvailable: params.quantityAvailable,
  };
  if (params.reorderLevel != null) input.reorderLevel = params.reorderLevel;
  await gqlAdmin<{ createInventoryItem?: unknown[] }>(
    `mutation CreateInventoryItem($input: NewInventoryItem!) {
      createInventoryItem(input: $input) { inventoryId variantId quantityAvailable reorderLevel }
    }`,
    { input }
  );
}

export interface InventoryItemRow {
  inventoryId: string;
  variantId: string;
  quantityAvailable: string;
  reorderLevel: string;
}

/** Find inventory rows by variant id (usually 0 or 1 row). */
export async function searchInventoryByVariantId(
  variantId: string
): Promise<InventoryItemRow[]> {
  const data = await gqlAdmin<{ searchInventoryItem?: InventoryItemRow[] }>(
    `query SearchInventoryByVariant($input: SearchInventoryItem!) {
      searchInventoryItem(input: $input) {
        inventoryId
        variantId
        quantityAvailable
        reorderLevel
      }
    }`,
    { input: { variantId } }
  );
  return data?.searchInventoryItem ?? [];
}

/** Update inventory row quantity/reorder level. */
export async function updateInventoryItem(params: {
  inventoryId: string;
  quantityAvailable?: string;
  reorderLevel?: string;
}): Promise<void> {
  const input: Record<string, string> = { inventoryId: params.inventoryId };
  if (params.quantityAvailable != null) input.quantityAvailable = params.quantityAvailable;
  if (params.reorderLevel != null) input.reorderLevel = params.reorderLevel;
  await gqlAdmin<{ updateInventoryItem?: unknown[] }>(
    `mutation UpdateInventoryItem($input: InventoryItemMutation!) {
      updateInventoryItem(input: $input) {
        inventoryId
        variantId
        quantityAvailable
        reorderLevel
      }
    }`,
    { input }
  );
}

/** Unix timestamp in seconds for start/end of day or month */
function startOfTodaySeconds(): number {
  const d = new Date();
  d.setHours(0, 0, 0, 0);
  return Math.floor(d.getTime() / 1000);
}

function endOfTodaySeconds(): number {
  const d = new Date();
  d.setHours(23, 59, 59, 999);
  return Math.floor(d.getTime() / 1000);
}

function startOfMonthSeconds(): number {
  const d = new Date();
  d.setDate(1);
  d.setHours(0, 0, 0, 0);
  return Math.floor(d.getTime() / 1000);
}

function endOfMonthSeconds(): number {
  const d = new Date();
  d.setMonth(d.getMonth() + 1, 0);
  d.setHours(23, 59, 59, 999);
  return Math.floor(d.getTime() / 1000);
}

/** Last N days: start of first day (00:00) to end of today (23:59), as unix seconds strings. */
export function lastNDaysRange(n: number): { orderDateStart: string; orderDateEnd: string } {
  const end = new Date();
  end.setHours(23, 59, 59, 999);
  const start = new Date();
  start.setDate(start.getDate() - n);
  start.setHours(0, 0, 0, 0);
  return {
    orderDateStart: String(Math.floor(start.getTime() / 1000)),
    orderDateEnd: String(Math.floor(end.getTime() / 1000)),
  };
}

/** Last N months: start of month N months ago to end of current month, as unix seconds strings. */
export function lastNMonthsRange(n: number): { orderDateStart: string; orderDateEnd: string } {
  const end = new Date();
  end.setMonth(end.getMonth() + 1, 0);
  end.setHours(23, 59, 59, 999);
  const start = new Date();
  start.setMonth(start.getMonth() - n, 1);
  start.setHours(0, 0, 0, 0);
  return {
    orderDateStart: String(Math.floor(start.getTime() / 1000)),
    orderDateEnd: String(Math.floor(end.getTime() / 1000)),
  };
}

/** Last N weeks: start of week N weeks ago (Monday) to end of today, as unix seconds strings. */
export function lastNWeeksRange(n: number): { orderDateStart: string; orderDateEnd: string } {
  const end = new Date();
  end.setHours(23, 59, 59, 999);
  const start = new Date();
  start.setDate(start.getDate() - n * 7);
  const day = start.getDay();
  const toMonday = day === 0 ? -6 : 1 - day;
  start.setDate(start.getDate() + toMonday);
  start.setHours(0, 0, 0, 0);
  return {
    orderDateStart: String(Math.floor(start.getTime() / 1000)),
    orderDateEnd: String(Math.floor(end.getTime() / 1000)),
  };
}

/** Last N years: start of year N years ago to end of current year, as unix seconds strings. */
export function lastNYearsRange(n: number): { orderDateStart: string; orderDateEnd: string } {
  const end = new Date();
  end.setMonth(11, 31);
  end.setHours(23, 59, 59, 999);
  const start = new Date();
  start.setFullYear(start.getFullYear() - n, 0, 1);
  start.setHours(0, 0, 0, 0);
  return {
    orderDateStart: String(Math.floor(start.getTime() / 1000)),
    orderDateEnd: String(Math.floor(end.getTime() / 1000)),
  };
}

export interface DashboardStats {
  ordersToday: number;
  revenueMtdPaise: number;
  revenueMtdFormatted: string;
  productsCount: number;
  customersCount: number | null;
}

/** Fetch orders with optional date range (unix seconds as string for GraphQL â†’ backend i64). Omit userId for admin (all orders). */
export async function fetchOrdersByDateRange(params: {
  orderDateStart?: string;
  orderDateEnd?: string;
  limit?: string;
}): Promise<OrderRow[]> {
  const search: Record<string, string> = {
    userId: "",
    limit: normalizeLimit(params.limit),
  };
  if (params.orderDateStart) search.orderDateStart = params.orderDateStart;
  if (params.orderDateEnd) search.orderDateEnd = params.orderDateEnd;

  const data = await gqlAdmin<{ searchOrder?: OrderRow[] }>(
    `query SearchOrders($search: SearchOrder!) {
      searchOrder(search: $search) {
        orderId
        totalAmountPaise
        totalAmountFormatted
        orderDate
      }
    }`,
    { search }
  );
  return data?.searchOrder ?? [];
}

export async function fetchAllOrdersByDateRange(params: {
  orderDateStart?: string;
  orderDateEnd?: string;
}): Promise<OrderRow[]> {
  const rows = await fetchAllOrdersList({
    orderDateStart: params.orderDateStart,
    orderDateEnd: params.orderDateEnd,
  });
  return rows.map((o) => ({
    orderId: o.orderId,
    totalAmountPaise: o.totalAmountPaise,
    totalAmountFormatted: o.totalAmountFormatted,
    orderDate: o.orderDate,
  }));
}

/** Order status row for dropdown (from OrderStatus table). */
export interface OrderStatusRow {
  statusId: string;
  statusName: string;
}

/** Fetch all order statuses for admin dropdown. */
export async function fetchOrderStatuses(): Promise<OrderStatusRow[]> {
  const data = await gqlAdmin<{ searchOrderStatus?: OrderStatusRow[] }>(
    `query OrderStatuses {
      searchOrderStatus {
        statusId
        statusName
      }
    }`
  );
  return data?.searchOrderStatus ?? [];
}

/** Fetch orders for admin list with optional date range, status, userId, and pagination. */
export async function fetchOrdersList(params: {
  orderDateStart?: string;
  orderDateEnd?: string;
  statusId?: string;
  userId?: string;
  limit?: string;
  offset?: string;
}): Promise<OrderListRow[]> {
  const search: Record<string, string> = {
    userId: "",
    limit: normalizeLimit(params.limit),
  };
  if (params.userId) search.userId = params.userId;
  if (params.orderDateStart) search.orderDateStart = params.orderDateStart;
  if (params.orderDateEnd) search.orderDateEnd = params.orderDateEnd;
  if (params.statusId) search.statusId = params.statusId;
  const normalizedOffset = normalizeOffset(params.offset);
  if (normalizedOffset != null) search.offset = normalizedOffset;

  const data = await gqlAdmin<{ searchOrder?: OrderListRow[] }>(
    `query SearchOrdersList($search: SearchOrder!) {
      searchOrder(search: $search) {
        orderId
        userId
        totalAmountPaise
        totalAmountFormatted
        orderDate
        statusId
      }
    }`,
    { search }
  );
  return data?.searchOrder ?? [];
}

export async function fetchAllOrdersList(params: {
  orderDateStart?: string;
  orderDateEnd?: string;
  statusId?: string;
  userId?: string;
} = {}): Promise<OrderListRow[]> {
  const out: OrderListRow[] = [];
  let offset = 0;
  for (;;) {
    const page = await fetchOrdersList({
      ...params,
      limit: String(ORDER_PAGE_SIZE),
      offset: String(offset),
    });
    out.push(...page);
    if (page.length < ORDER_PAGE_SIZE) break;
    offset += ORDER_PAGE_SIZE;
    if (offset > 100_000) break;
  }
  return out;
}

/** Fetch product count via paginated requests to respect GraphQL limits. */
async function fetchProductCount(): Promise<number> {
  let total = 0;
  let offset = 0;
  for (;;) {
    const data = await gqlAdmin<{ searchProduct?: Array<{ productId: string }> }>(
      PRODUCT_COUNT_QUERY,
      { search: { limit: String(PRODUCT_PAGE_SIZE), offset: String(offset) } }
    );
    const list = data?.searchProduct ?? [];
    total += list.length;
    if (list.length < PRODUCT_PAGE_SIZE) break;
    offset += PRODUCT_PAGE_SIZE;
    if (offset > 100_000) break;
  }
  return total;
}

/** Customer row for admin list (from searchUser). */
export interface CustomerListRow {
  userId: string;
  username: string;
  email: string;
  authProvider: string;
  fullName: string | null;
  address: string | null;
  phone: string | null;
  createDate: string;
}

/** Fetch one customers page (bounded). */
export async function fetchCustomersList(params?: {
  limit?: string;
  offset?: string;
}): Promise<CustomerListRow[]> {
  const limit = normalizeLimit(params?.limit);
  const offset = normalizeOffset(params?.offset) ?? "0";
  const data = await gqlAdmin<{ searchUser?: CustomerListRow[] }>(
    `query SearchUsersList($input: SearchUserInput!) {
      searchUser(input: $input) {
        userId
        username
        email
        authProvider
        fullName
        address
        phone
        createDate
      }
    }`,
    { input: { userId: "0", limit, offset } }
  );
  return data?.searchUser ?? [];
}

export async function fetchAllCustomersList(): Promise<CustomerListRow[]> {
  const out: CustomerListRow[] = [];
  let offset = 0;
  for (;;) {
    const page = await fetchCustomersList({
      limit: String(ORDER_PAGE_SIZE),
      offset: String(offset),
    });
    out.push(...page);
    if (page.length < ORDER_PAGE_SIZE) break;
    offset += ORDER_PAGE_SIZE;
    if (offset > 100_000) break;
  }
  return out;
}

/** Order counts by status for dashboard (total, pending, delivered, cancelled, in transit). */
export interface OrderCountsByStatus {
  total: number;
  pending: number;
  delivered: number;
  cancelled: number;
  inTransit: number;
}

/** Fetch order counts: total and by status (pending, delivered, cancelled, in transit = shipped). */
export async function fetchOrderCountsByStatus(): Promise<OrderCountsByStatus> {
  const [orders, statuses] = await Promise.all([
    fetchAllOrdersList(),
    fetchOrderStatuses(),
  ]);
  const idToName = new Map<string, string>(
    statuses.map((s) => [s.statusId, s.statusName.toLowerCase()])
  );
  let pending = 0;
  let delivered = 0;
  let cancelled = 0;
  let inTransit = 0;
  for (const o of orders) {
    const name = idToName.get(o.statusId) ?? "";
    if (name === "pending") pending++;
    else if (name === "delivered") delivered++;
    else if (name === "cancelled") cancelled++;
    else if (name === "shipped") inTransit++;
  }
  return {
    total: orders.length,
    pending,
    delivered,
    cancelled,
    inTransit,
  };
}

/** Fetch customer count via searchUser(userId: "0") which returns all users; we count the list. */
async function fetchCustomerCount(): Promise<number> {
  const list = await fetchAllCustomersList();
  return list.length;
}

/**
 * Fetch all metrics for the admin dashboard.
 * Customers: searchUser(userId: "0") returns all users; we use list length as count.
 */
export async function fetchDashboardStats(): Promise<DashboardStats> {
  const todayStart = String(startOfTodaySeconds());
  const todayEnd = String(endOfTodaySeconds());
  const monthStart = String(startOfMonthSeconds());
  const monthEnd = String(endOfMonthSeconds());

  const [ordersToday, ordersMtd, productsCount, customersCount] = await Promise.all([
    fetchAllOrdersList({ orderDateStart: todayStart, orderDateEnd: todayEnd }),
    fetchAllOrdersList({ orderDateStart: monthStart, orderDateEnd: monthEnd }),
    fetchProductCount(),
    fetchCustomerCount(),
  ]);

  const revenueMtdPaise = ordersMtd.reduce(
    (sum, o) => sum + parseInt(o.totalAmountPaise, 10) || 0,
    0
  );
  const revenueMtdFormatted = revenueMtdPaise >= 0 ? formatInrFromPaise(revenueMtdPaise) : "-";

  return {
    ordersToday: ordersToday.length,
    revenueMtdPaise,
    revenueMtdFormatted,
    productsCount,
    customersCount,
  };
}

/** Revenue (MTD), Revenue (Total), and customers count for dashboard extra cards. */
export interface DashboardExtras {
  revenueMtdFormatted: string;
  revenueTotalFormatted: string;
  customersCount: number;
}

/** Fetch revenue MTD, revenue total (all orders), and customer count for dashboard. */
export async function fetchDashboardExtras(): Promise<DashboardExtras> {
  const [stats, allOrders] = await Promise.all([
    fetchDashboardStats(),
    fetchAllOrdersList(),
  ]);
  const totalPaise = allOrders.reduce(
    (sum, o) => sum + (parseInt(o.totalAmountPaise, 10) || 0),
    0
  );
  const revenueTotalFormatted = totalPaise >= 0 ? formatInrFromPaise(totalPaise) : "-";
  return {
    revenueMtdFormatted: stats.revenueMtdFormatted,
    revenueTotalFormatted,
    customersCount: stats.customersCount ?? 0,
  };
}
