import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

type CancelOrderItemsGqlRow = {
  orderId: string;
  statusId: string;
};

type CancelOrderItemsBody = {
  orderDetailIds?: string[];
};

const CANCEL_ORDER_ITEMS_MUTATION = `mutation AccountCancelOrderItems($input: CancelOrderItemsInput!) {
  cancelOrderItems(input: $input) {
    orderId
    statusId
  }
}`;

export async function POST(
  request: Request,
  context: { params: Promise<{ orderId: string }> }
) {
  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) {
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
  }

  const { orderId } = await context.params;
  const trimmedOrderId = orderId.trim();
  if (!trimmedOrderId) {
    return apiError("Order ID is required", 400, "VALIDATION_ERROR");
  }

  let body: CancelOrderItemsBody = {};
  try {
    body = (await request.json()) as CancelOrderItemsBody;
  } catch {
    return apiError("Invalid request body", 400, "VALIDATION_ERROR");
  }
  const orderDetailIds = (body.orderDetailIds ?? []).map((v) => String(v).trim()).filter(Boolean);
  if (orderDetailIds.length === 0) {
    return apiError("At least one orderDetailId is required", 400, "VALIDATION_ERROR");
  }

  const result = await callGraphqlAsCustomer<{ cancelOrderItems?: CancelOrderItemsGqlRow[] }>(
    userId,
    CANCEL_ORDER_ITEMS_MUTATION,
    { input: { orderId: trimmedOrderId, orderDetailIds } }
  );
  const firstError = result.errors?.[0];
  const gqlMessage = firstError?.message;
  if (gqlMessage) {
    const code = firstError?.extensions?.code;
    const lower = gqlMessage.toLowerCase();
    const status =
      code === "NotFound" || lower.includes("not found") || lower.includes("order not found")
        ? 404
        : code === "FailedPrecondition" || lower.includes("failed_precondition")
          ? 409
          : 400;
    return apiError(gqlMessage, status, "GRAPHQL_ERROR");
  }

  const row = result.data?.cancelOrderItems?.[0];
  if (!row) {
    return apiError("Cancel did not return an order", 502, "GRAPHQL_ERROR");
  }

  return Response.json({
    ok: true,
    data: { orderId: row.orderId, statusId: row.statusId },
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
