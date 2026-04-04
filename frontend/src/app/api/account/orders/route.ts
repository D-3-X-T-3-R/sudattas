import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

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

export async function GET() {
  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) {
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
  }

  const [ordersResult, statusesResult] = await Promise.all([
    callGraphqlAsCustomer<{ searchOrder?: OrderRow[] }>(userId, ORDERS_QUERY, {
      search: { userId, limit: "100" },
    }),
    callGraphqlAsCustomer<{
      searchOrderStatus?: Array<{ statusId: string; statusName: string }>;
    }>(userId, ORDER_STATUS_QUERY),
  ]);

  if (ordersResult.errors?.length) {
    return apiError(
      ordersResult.errors[0]?.message ?? "Failed to load orders",
      400,
      "GRAPHQL_ERROR"
    );
  }
  if (statusesResult.errors?.length) {
    return apiError(
      statusesResult.errors[0]?.message ?? "Failed to load order statuses",
      400,
      "GRAPHQL_ERROR"
    );
  }

  const statusNameById = new Map(
    (statusesResult.data?.searchOrderStatus ?? []).map((s) => [s.statusId, s.statusName])
  );

  const orders = (ordersResult.data?.searchOrder ?? []).map((order) => ({
    ...order,
    statusName: statusNameById.get(order.statusId) ?? order.statusId,
  }));
  const mismatchedOrder = orders.find((order) => order.userId !== userId);
  if (mismatchedOrder) {
    return apiError(
      "Order identity mismatch for authenticated customer",
      403,
      "FORBIDDEN"
    );
  }

  return Response.json({
    ok: true,
    data: orders,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
