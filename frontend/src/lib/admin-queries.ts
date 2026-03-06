/**
 * Admin dashboard GraphQL queries. Use with gqlAdmin (sends Bearer token).
 */

import { gqlAdmin } from "./graphqlAdmin";

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

/** Category for admin dropdowns */
export interface CategoryRow {
  categoryId: string;
  name: string;
}

/** Product row for admin list (includes categoryId for filter and display) */
export interface ProductListRow {
  productId: string;
  name: string;
  formatted: string;
  stockQuantity?: string | null;
  categoryId?: string | null;
}

/** Fetch all categories for admin dropdowns. */
export async function fetchCategories(): Promise<CategoryRow[]> {
  const data = await gqlAdmin<{ searchCategory?: CategoryRow[] }>(
    `query Categories { searchCategory(search: {}) { categoryId name } }`
  );
  return data?.searchCategory ?? [];
}

/** Fetch products for admin list with optional name, category, and limit. */
export async function fetchProductsList(params: {
  name?: string;
  categoryId?: string;
  limit?: string;
}): Promise<ProductListRow[]> {
  const search: Record<string, string> = {};
  if (params.name) search.name = params.name;
  if (params.categoryId) search.categoryId = params.categoryId;
  if (params.limit) search.limit = params.limit;

  const data = await gqlAdmin<{ searchProduct?: ProductListRow[] }>(
    `query SearchProductsList($search: SearchProduct!) {
      searchProduct(search: $search) {
        productId
        name
        formatted
        stockQuantity
        categoryId
      }
    }`,
    { search: Object.keys(search).length ? search : { limit: "50" } }
  );
  return data?.searchProduct ?? [];
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

/** Occasion options for products (id "0" returns all) */
export interface OccasionRow {
  occasionId: string;
  occasionName: string;
}

export async function fetchOccasions(): Promise<OccasionRow[]> {
  const data = await gqlAdmin<{ searchOccasion?: OccasionRow[] }>(
    `mutation Occasions { searchOccasion(input: { occasionId: "0" }) { occasionId occasionName } }`
  );
  return data?.searchOccasion ?? [];
}

/** Product attribute (name/value) for linking to products */
export interface ProductAttributeRow {
  attributeId: string;
  attributeName: string;
  attributeValue: string;
}

export async function searchProductAttributes(params?: {
  attributeName?: string;
  attributeValue?: string;
}): Promise<ProductAttributeRow[]> {
  const input: Record<string, string> = {};
  if (params?.attributeName) input.attributeName = params.attributeName;
  if (params?.attributeValue) input.attributeValue = params.attributeValue;
  const data = await gqlAdmin<{ searchProductAttribute?: ProductAttributeRow[] }>(
    `query SearchProductAttributes($input: SearchProductAttributeInput!) {
      searchProductAttribute(input: $input) { attributeId attributeName attributeValue }
    }`,
    { input: Object.keys(input).length ? input : {} }
  );
  return data?.searchProductAttribute ?? [];
}

/** Create a product attribute (name/value). Returns created attribute with attributeId. */
export async function createProductAttribute(
  attributeName: string,
  attributeValue: string
): Promise<ProductAttributeRow | null> {
  const data = await gqlAdmin<{ createProductAttribute?: ProductAttributeRow[] }>(
    `mutation CreateProductAttribute($input: NewProductAttribute!) {
      createProductAttribute(input: $input) { attributeId attributeName attributeValue }
    }`,
    { input: { attributeName, attributeValue } }
  );
  return data?.createProductAttribute?.[0] ?? null;
}

/** Link a product to an attribute. */
export async function createProductAttributeMapping(
  productId: string,
  attributeId: string
): Promise<void> {
  await gqlAdmin<{ createProductAttributeMapping?: unknown[] }>(
    `mutation CreateProductAttributeMapping($input: NewProductAttributeMapping!) {
      createProductAttributeMapping(input: $input) { productId attributeId }
    }`,
    { input: { productId, attributeId } }
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

export interface DashboardStats {
  ordersToday: number;
  revenueMtdPaise: number;
  revenueMtdFormatted: string;
  productsCount: number;
  customersCount: number | null;
}

/** Fetch orders with optional date range (unix seconds as string for GraphQL → backend i64). Omit userId for admin (all orders). */
async function fetchOrders(params: {
  orderDateStart?: string;
  orderDateEnd?: string;
  limit?: string;
}): Promise<OrderRow[]> {
  const search: Record<string, string> = {
    userId: "",
    limit: params.limit ?? "500",
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

/** Fetch orders for admin list with optional date range, status, and pagination. */
export async function fetchOrdersList(params: {
  orderDateStart?: string;
  orderDateEnd?: string;
  statusId?: string;
  limit?: string;
  offset?: string;
}): Promise<OrderListRow[]> {
  const search: Record<string, string> = {
    userId: "",
    limit: params.limit ?? "100",
  };
  if (params.orderDateStart) search.orderDateStart = params.orderDateStart;
  if (params.orderDateEnd) search.orderDateEnd = params.orderDateEnd;
  if (params.statusId) search.statusId = params.statusId;
  if (params.offset) search.offset = params.offset;

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

/** Fetch product count (search with high limit, then count) */
async function fetchProductCount(): Promise<number> {
  const data = await gqlAdmin<{ searchProduct?: ProductRow[] }>(
    `query SearchProducts($search: SearchProduct!) {
      searchProduct(search: $search) {
        productId
        name
      }
    }`,
    { search: { limit: "5000" } }
  );
  const list = data?.searchProduct ?? [];
  return list.length;
}

/** Fetch customer count via searchUser(userId: "0") which returns all users; we count the list. */
async function fetchCustomerCount(): Promise<number> {
  const data = await gqlAdmin<{ searchUser?: { userId: string }[] }>(
    `query SearchUsers($input: SearchUserInput!) {
      searchUser(input: $input) {
        userId
      }
    }`,
    { input: { userId: "0" } }
  );
  const list = data?.searchUser ?? [];
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
    fetchOrders({ orderDateStart: todayStart, orderDateEnd: todayEnd, limit: "500" }),
    fetchOrders({ orderDateStart: monthStart, orderDateEnd: monthEnd, limit: "2000" }),
    fetchProductCount(),
    fetchCustomerCount(),
  ]);

  const revenueMtdPaise = ordersMtd.reduce(
    (sum, o) => sum + parseInt(o.totalAmountPaise, 10) || 0,
    0
  );
  const revenueMtdFormatted =
    revenueMtdPaise >= 0
      ? new Intl.NumberFormat("en-IN", {
          style: "currency",
          currency: "INR",
          maximumFractionDigits: 0,
        }).format(revenueMtdPaise / 100)
      : "—";

  return {
    ordersToday: ordersToday.length,
    revenueMtdPaise,
    revenueMtdFormatted,
    productsCount,
    customersCount,
  };
}
