import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

type CancelOrderGqlRow = {
  orderId: string;
  statusId: string;
};

const DELETE_ORDER_CANCEL = `mutation AccountCancelOrder($orderId: String!) {
  deleteOrder(orderId: $orderId) {
    orderId
    statusId
  }
}`;

export async function POST(
  _request: Request,
  context: { params: Promise<{ orderId: string }> }
) {
  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) {
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
  }

  const { orderId } = await context.params;
  const trimmed = orderId.trim();
  if (!trimmed) {
    return apiError("Order ID is required", 400, "VALIDATION_ERROR");
  }

  const result = await callGraphqlAsCustomer<{ deleteOrder?: CancelOrderGqlRow[] }>(
    userId,
    DELETE_ORDER_CANCEL,
    { orderId: trimmed }
  );

  const gqlMessage = result.errors?.[0]?.message;
  if (gqlMessage) {
    const lower = gqlMessage.toLowerCase();
    const status =
      lower.includes("not found") || lower.includes("order not found")
        ? 404
        : lower.includes("failed_precondition")
          ? 409
        : lower.includes("illegal") || lower.includes("invalid")
          ? 400
          : 400;
    return apiError(gqlMessage, status, "GRAPHQL_ERROR");
  }

  const rows = result.data?.deleteOrder;
  const first = rows?.[0];
  if (!first) {
    return apiError("Cancel did not return an order", 502, "GRAPHQL_ERROR");
  }

  return Response.json({
    ok: true,
    data: { orderId: first.orderId, statusId: first.statusId },
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
