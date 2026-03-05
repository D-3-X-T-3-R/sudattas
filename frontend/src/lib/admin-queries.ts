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
