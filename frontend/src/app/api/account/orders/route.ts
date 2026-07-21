import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";
import { canonicalOrderStatusName, statusNameFromId } from "@/lib/order-state";

type OrderRow = {
  orderId: string;
  userId: string;
  orderDate: string;
  cancelWindowEndsAt?: string | null;
  paymentMethod?: string | null;
  totalAmountPaise: string;
  totalAmountFormatted: string;
  statusId: string;
};

const ORDERS_QUERY = `query AccountOrders($search: SearchOrder!) {
  searchOrder(search: $search) {
    orderId
    userId
    orderDate
    cancelWindowEndsAt
    paymentMethod
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

  const cancelWindowHours = Number.parseInt(
    (process.env.CANCEL_WINDOW_HOURS ?? "12").trim(),
    10
  );
  const normalizedCancelWindowHours =
    Number.isFinite(cancelWindowHours) && cancelWindowHours > 0
      ? cancelWindowHours
      : 12;
  const returnWindowDays = Number.parseInt(
    (process.env.RETURN_WINDOW_DAYS ?? "7").trim(),
    10
  );
  const normalizedReturnWindowDays =
    Number.isFinite(returnWindowDays) && returnWindowDays > 0
      ? returnWindowDays
      : 7;

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
    (statusesResult.data?.searchOrderStatus ?? []).map((s) => [
      s.statusId,
      canonicalOrderStatusName(s.statusName),
    ])
  );

  const orders = (ordersResult.data?.searchOrder ?? []).map((order) => ({
    ...order,
    statusName: statusNameFromId(order.statusId, statusNameById),
    cancelWindowHours: normalizedCancelWindowHours,
    returnWindowDays: normalizedReturnWindowDays,
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
