/**
 * Admin dashboard GraphQL queries. Use gqlAdmin (Bearer / admin key) only.
 * For storefront (guest session) use @/lib/storefront-queries.
 *
 * Product-catalog admin queries (categories, products, sizes, colors, fabrics, weaves,
 * occasions, moods, variants, inventory) live in ./admin-product-queries and are re-exported
 * below — existing `import { fetchProductsList } from "@/lib/admin-queries"` call sites are
 * unaffected. Split out to keep this file under the max-lines lint limit.
 */

import { gqlAdmin } from "./graphql-client";
import { formatInrFromPaise } from "./money";
import { normalizeLimit, normalizeOffset, ORDER_PAGE_SIZE, PRODUCT_PAGE_SIZE } from "./admin-query-utils";

export * from "./admin-product-queries";

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
  paymentMethod: string | null;
}

export interface ProductRow {
  productId: string;
  name: string;
}

const PRODUCT_COUNT_QUERY = `query ProductCountPage($search: SearchProduct!) {
  searchProduct(search: $search) { productId }
}`;

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
        paymentMethod
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

/** One order-line row, flattened out of a nested searchOrder→orderDetails→productDetails query. */
export interface OrderLineRow {
  orderId: string;
  orderDate: string;
  statusId: string;
  productId: string | null;
  productName: string | null;
  categoryId: string | null;
  quantity: number;
  linePricePaise: number;
}

const ORDER_LINES_QUERY = `query SearchOrderLines($search: SearchOrder!) {
  searchOrder(search: $search) {
    orderId
    orderDate
    statusId
    orderDetails {
      quantity
      pricePaise
      productDetails { productId name categoryId }
    }
  }
}`;

/**
 * Fetch order line items (product, category, quantity, price) across many orders in one date range.
 * Nested under searchOrder (no flat bulk order-details endpoint exists) — bound the date range,
 * this resolves order-details and product-details per order server-side.
 */
export async function fetchOrderLinesByDateRange(params: {
  orderDateStart?: string;
  orderDateEnd?: string;
}): Promise<OrderLineRow[]> {
  const out: OrderLineRow[] = [];
  let offset = 0;
  for (;;) {
    const search: Record<string, string> = {
      userId: "",
      limit: String(ORDER_PAGE_SIZE),
      offset: String(offset),
    };
    if (params.orderDateStart) search.orderDateStart = params.orderDateStart;
    if (params.orderDateEnd) search.orderDateEnd = params.orderDateEnd;

    const data = await gqlAdmin<{
      searchOrder?: Array<{
        orderId: string;
        orderDate: string;
        statusId: string;
        orderDetails?: Array<{
          quantity: string;
          pricePaise: string;
          productDetails?: Array<{ productId: string; name: string; categoryId: string | null }>;
        }>;
      }>;
    }>(ORDER_LINES_QUERY, { search });

    const page = data?.searchOrder ?? [];
    for (const order of page) {
      for (const line of order.orderDetails ?? []) {
        const product = line.productDetails?.[0];
        out.push({
          orderId: order.orderId,
          orderDate: order.orderDate,
          statusId: order.statusId,
          productId: product?.productId ?? null,
          productName: product?.name ?? null,
          categoryId: product?.categoryId ?? null,
          quantity: Number.parseInt(line.quantity, 10) || 0,
          linePricePaise: Number.parseInt(line.pricePaise, 10) || 0,
        });
      }
    }
    if (page.length < ORDER_PAGE_SIZE) break;
    offset += ORDER_PAGE_SIZE;
    if (offset > 100_000) break;
  }
  return out;
}

const PRODUCT_SUMMARY_QUERY = `query SearchProductSummary($search: SearchProduct!) {
  searchProduct(search: $search) { productId name categoryId stockQuantity productStatusId }
}`;

/** Lightweight product row for dashboard aggregation (category mix, low-stock count/list). */
export interface ProductSummaryRow {
  productId: string;
  name: string;
  categoryId: string | null;
  stockQuantity: number | null;
  productStatusId: string | null;
}

/** Fetch productId/categoryId/stockQuantity/productStatusId for every product (paginated). Used by dashboard charts, not the products list page. */
export async function fetchAllProductsSummary(): Promise<ProductSummaryRow[]> {
  const out: ProductSummaryRow[] = [];
  let offset = 0;
  for (;;) {
    const data = await gqlAdmin<{ searchProduct?: ProductSummaryRow[] }>(PRODUCT_SUMMARY_QUERY, {
      search: { limit: String(PRODUCT_PAGE_SIZE), offset: String(offset) },
    });
    const page = data?.searchProduct ?? [];
    out.push(...page);
    if (page.length < PRODUCT_PAGE_SIZE) break;
    offset += PRODUCT_PAGE_SIZE;
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
  /** "active" | "inactive" | "suspended"; null means never explicitly set (treated as active). */
  userStatus: string | null;
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
        userStatus
      }
    }`,
    { input: { userId: "0", limit, offset } }
  );
  return data?.searchUser ?? [];
}

/**
 * Admin: activate/deactivate/suspend a customer account. Not cosmetic — the backend auth gate
 * rejects every future JWT-authenticated request from a deactivated/suspended account (login,
 * checkout, everything), subject to a short cache TTL server-side. Deliberately a separate
 * mutation from updateCustomerAdmin, which intentionally never touches this field.
 */
export async function setCustomerStatus(
  userId: string,
  status: "active" | "inactive" | "suspended"
): Promise<CustomerListRow | null> {
  const data = await gqlAdmin<{ setUserStatus?: CustomerListRow[] }>(
    `mutation AdminSetCustomerStatus($input: SetUserStatusInput!) {
      setUserStatus(input: $input) {
        userId
        username
        email
        authProvider
        fullName
        address
        phone
        createDate
        userStatus
      }
    }`,
    { input: { userId, status } }
  );
  return data?.setUserStatus?.[0] ?? null;
}

/** Fields `exportMyPii` doesn't already put in front of an admin via the customer list/profile
 * dialog (gender, dateOfBirth, firstName/lastName) plus the rest, for a complete, exportable
 * record — the admin-facing counterpart to a customer's own "Download my data" button. */
export interface AdminUserPiiExport {
  userId: string;
  email: string;
  fullName: string | null;
  address: string | null;
  phone: string | null;
  createDate: string;
  firstName: string | null;
  lastName: string | null;
  gender: string | null;
  dateOfBirth: string | null;
}

/** Admin lookup of another customer's full PII record by id, for support/data-request purposes. */
export async function fetchCustomerPiiExport(userId: string): Promise<AdminUserPiiExport | null> {
  const data = await gqlAdmin<{ adminExportUserPii?: AdminUserPiiExport }>(
    `query AdminExportUserPii($userId: String!) {
      adminExportUserPii(userId: $userId) {
        userId
        email
        fullName
        address
        phone
        createDate
        firstName
        lastName
        gender
        dateOfBirth
      }
    }`,
    { userId }
  );
  return data?.adminExportUserPii ?? null;
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

/** Admin: edit a customer's non-status profile fields (name/address/phone). Deliberately
 * excludes email/username/role/status — those are identity- or access-critical and out of scope
 * for a quick profile edit; use dedicated flows for those if/when built. */
export async function updateCustomerAdmin(params: {
  userId: string;
  fullName?: string;
  address?: string;
  phone?: string;
}): Promise<CustomerListRow | null> {
  const data = await gqlAdmin<{ updateUser?: CustomerListRow[] }>(
    `mutation AdminUpdateCustomer($input: UpdateUserInput!) {
      updateUser(input: $input) {
        userId
        username
        email
        authProvider
        fullName
        address
        phone
        createDate
        userStatus
      }
    }`,
    {
      input: {
        userId: params.userId,
        fullName: params.fullName,
        address: params.address,
        phone: params.phone,
      },
    }
  );
  return data?.updateUser?.[0] ?? null;
}

/** Order counts by status for dashboard (total, pending, delivered, cancelled, in transit). */
export interface OrderCountsByStatus {
  total: number;
  pending: number;
  delivered: number;
  cancelled: number;
  inTransit: number;
}

const ORDER_STATS_QUERY = `query AdminOrderStats($input: GetOrderStatsInput!) {
  orderStats(input: $input) {
    totalOrders
    totalRevenuePaise
    byStatus { statusId statusName count }
    customerCount
  }
}`;

interface OrderStatsGqlResult {
  totalOrders: string;
  totalRevenuePaise: string;
  byStatus: Array<{ statusId: string; statusName: string; count: string }>;
  customerCount: string;
}

/**
 * Fetch aggregated order/revenue/customer stats via a single GraphQL query computed server-side
 * with SQL COUNT/SUM/GROUP BY, instead of paginating the entire Orders/Users tables into the
 * browser and counting/summing client-side (previously up to ~2000 requests per dashboard load).
 */
async function fetchOrderStats(
  params: { orderDateStart?: string; orderDateEnd?: string } = {}
): Promise<OrderStatsGqlResult> {
  const data = await gqlAdmin<{ orderStats?: OrderStatsGqlResult }>(ORDER_STATS_QUERY, {
    input: {
      orderDateStart: params.orderDateStart ?? null,
      orderDateEnd: params.orderDateEnd ?? null,
    },
  });
  return (
    data?.orderStats ?? {
      totalOrders: "0",
      totalRevenuePaise: "0",
      byStatus: [],
      customerCount: "0",
    }
  );
}

/** Fetch order counts: total and by status (pending, delivered, cancelled, in transit = shipped). */
export async function fetchOrderCountsByStatus(): Promise<OrderCountsByStatus> {
  const stats = await fetchOrderStats();
  const byName = new Map<string, number>(
    stats.byStatus.map((s) => [s.statusName.toLowerCase(), Number.parseInt(s.count, 10) || 0])
  );
  return {
    total: Number.parseInt(stats.totalOrders, 10) || 0,
    pending: byName.get("pending") ?? 0,
    delivered: byName.get("delivered") ?? 0,
    cancelled: byName.get("cancelled") ?? 0,
    inTransit: byName.get("shipped") ?? 0,
  };
}

/**
 * Fetch all metrics for the admin dashboard.
 * Orders/revenue/customers come from the orderStats aggregation query per date range instead of
 * paginating the full Orders/Users tables.
 */
export async function fetchDashboardStats(): Promise<DashboardStats> {
  const todayStart = String(startOfTodaySeconds());
  const todayEnd = String(endOfTodaySeconds());
  const monthStart = String(startOfMonthSeconds());
  const monthEnd = String(endOfMonthSeconds());

  const [statsToday, statsMtd, productsCount] = await Promise.all([
    fetchOrderStats({ orderDateStart: todayStart, orderDateEnd: todayEnd }),
    fetchOrderStats({ orderDateStart: monthStart, orderDateEnd: monthEnd }),
    fetchProductCount(),
  ]);

  const revenueMtdPaise = Number.parseInt(statsMtd.totalRevenuePaise, 10) || 0;
  const revenueMtdFormatted = revenueMtdPaise >= 0 ? formatInrFromPaise(revenueMtdPaise) : "-";

  return {
    ordersToday: Number.parseInt(statsToday.totalOrders, 10) || 0,
    revenueMtdPaise,
    revenueMtdFormatted,
    productsCount,
    customersCount: Number.parseInt(statsMtd.customerCount, 10) || 0,
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
  const [stats, allTimeStats] = await Promise.all([
    fetchDashboardStats(),
    fetchOrderStats(),
  ]);
  const totalPaise = Number.parseInt(allTimeStats.totalRevenuePaise, 10) || 0;
  const revenueTotalFormatted = totalPaise >= 0 ? formatInrFromPaise(totalPaise) : "-";
  return {
    revenueMtdFormatted: stats.revenueMtdFormatted,
    revenueTotalFormatted,
    customersCount: stats.customersCount ?? 0,
  };
}

export interface TelemetryRatio {
  numerator: number;
  denominator: number;
  percent: number;
}

export interface TelemetrySummary {
  windowHours: number;
  loginFailureRate: TelemetryRatio;
  cartConversionDropoff: TelemetryRatio;
  checkoutFailureRate: TelemetryRatio;
  paymentMismatchRate: TelemetryRatio;
  adminActionFailureRate: TelemetryRatio;
  releaseConfidence: { score: number; scale: string };
  webhookProcessingLatency: { available: boolean; averageMs?: number | null; message?: string | null };
  backendSignals?: {
    available: boolean;
    authUnauthorizedCount?: number;
    paymentFailureCount?: { value: number; unit: string };
    webhookFailureCount?: { value: number; unit: string };
    refundFailureCount?: { value: number; unit: string };
    shiprocketBookingFailureCount?: { value: number; unit: string };
    staleExpiryFailureCount?: { value: number; unit: string };
    cancelPendingLogisticsBacklog?: { value: number; unit: string };
    outboxBacklog?: { value: number; unit: string };
    stuckPendingOrders?: { value: number; unit: string };
    stuckPaymentIntents?: { value: number; unit: string };
  };
}

export async function fetchTelemetrySummary(): Promise<TelemetrySummary> {
  const res = await fetch("/api/telemetry/summary", { cache: "no-store" });
  const text = await res.text();
  if (!res.ok) {
    throw new Error(text || `HTTP ${res.status}`);
  }
  const parsed = JSON.parse(text) as {
    ok?: boolean;
    data?: TelemetrySummary;
    message?: string | null;
  };
  if (!parsed.ok || !parsed.data) {
    throw new Error(parsed.message || "Failed to load telemetry summary");
  }
  return parsed.data;
}
