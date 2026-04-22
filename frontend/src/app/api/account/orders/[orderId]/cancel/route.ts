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
  flowLog("cancel endpoint request received");
  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) {
    flowLog("cancel endpoint rejected: unauthenticated");
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
  }

  const { orderId } = await context.params;
  const trimmed = orderId.trim();
  if (!trimmed) {
    flowLog("cancel endpoint rejected: missing order id", { userId });
    return apiError("Order ID is required", 400, "VALIDATION_ERROR");
  }
  flowLog("cancel mutation requested", { userId, orderId: trimmed });

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
    flowLog("cancel mutation failed", {
      userId,
      orderId: trimmed,
      error: gqlMessage,
      mappedHttpStatus: status,
    });
    return apiError(gqlMessage, status, "GRAPHQL_ERROR");
  }

  const rows = result.data?.deleteOrder;
  const first = rows?.[0];
  if (!first) {
    flowLog("cancel mutation failed: empty payload", { userId, orderId: trimmed });
    return apiError("Cancel did not return an order", 502, "GRAPHQL_ERROR");
  }
  flowLog("cancel mutation success", {
    userId,
    orderId: first.orderId,
    updatedStatusId: first.statusId,
  });

  return Response.json({
    ok: true,
    data: { orderId: first.orderId, statusId: first.statusId },
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
