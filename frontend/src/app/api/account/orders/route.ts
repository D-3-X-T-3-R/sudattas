import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

function flowLog(message: string, meta?: Record<string, unknown>) {
  if (meta) {
    console.info(`[orders-flow][customer-api] ${message}`, meta);
    return;
  }
  console.info(`[orders-flow][customer-api] ${message}`);
}

type OrderRow = {
  orderId: string;
  userId: string;
  orderDate: string;
  totalAmountPaise: string;
  totalAmountFormatted: string;
  statusId: string;
};

const ORDERS_QUERY = `query AccountOrders($search: SearchOrder!) {
  searchOrder(search: $search) {
    orderId
    userId
    orderDate
    totalAmountPaise
    totalAmountFormatted
    statusId
  }
}`;

const ORDER_STATUS_QUERY = `query AccountOrderStatuses {
  searchOrderStatus {
    statusId
    statusName
  }
}`;

function formatOrderStatusName(statusName: string): string {
  return statusName.trim().toLowerCase() === "processing" ? "processing order" : statusName;
}

export async function GET() {
  flowLog("orders list request received");
  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) {
    flowLog("orders list rejected: unauthenticated");
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
  }
  flowLog("loading orders list", { userId });

  const [ordersResult, statusesResult] = await Promise.all([
    callGraphqlAsCustomer<{ searchOrder?: OrderRow[] }>(userId, ORDERS_QUERY, {
      search: { userId, limit: "100" },
    }),
    callGraphqlAsCustomer<{
      searchOrderStatus?: Array<{ statusId: string; statusName: string }>;
    }>(userId, ORDER_STATUS_QUERY),
  ]);

  if (ordersResult.errors?.length) {
    flowLog("orders list graphql error", {
      userId,
      error: ordersResult.errors[0]?.message ?? "Failed to load orders",
    });
    return apiError(
      ordersResult.errors[0]?.message ?? "Failed to load orders",
      400,
      "GRAPHQL_ERROR"
    );
  }
  if (statusesResult.errors?.length) {
    flowLog("order status lookup graphql error", {
      userId,
      error: statusesResult.errors[0]?.message ?? "Failed to load order statuses",
    });
    return apiError(
      statusesResult.errors[0]?.message ?? "Failed to load order statuses",
      400,
      "GRAPHQL_ERROR"
    );
  }

  const statusNameById = new Map(
    (statusesResult.data?.searchOrderStatus ?? []).map((s) => [
      s.statusId,
      formatOrderStatusName(s.statusName),
    ])
  );

  const orders = (ordersResult.data?.searchOrder ?? []).map((order) => ({
    ...order,
    statusName: statusNameById.get(order.statusId) ?? order.statusId,
  }));
  const mismatchedOrder = orders.find((order) => order.userId !== userId);
  if (mismatchedOrder) {
    flowLog("orders list identity mismatch", {
      userId,
      orderId: mismatchedOrder.orderId,
      ownerUserId: mismatchedOrder.userId,
    });
    return apiError(
      "Order identity mismatch for authenticated customer",
      403,
      "FORBIDDEN"
    );
  }
  flowLog("orders list loaded", {
    userId,
    totalOrders: orders.length,
    latestOrderId: orders[0]?.orderId ?? null,
    statuses: orders.map((o) => o.statusName ?? o.statusId),
  });

  return Response.json({
    ok: true,
    data: orders,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
