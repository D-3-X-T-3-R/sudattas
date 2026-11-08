import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

type ExchangeRequestRow = {
  exchangeId: string;
  orderId: string;
  orderDetailId: string;
  desiredVariantId: string;
  quantity: string;
  status: string;
  reason: string;
};

type RequestExchangeBody = {
  orderDetailId?: string;
  desiredVariantId?: string;
  quantity?: string;
  reason?: string;
};

const REQUEST_EXCHANGE_MUTATION = `mutation AccountRequestExchange($input: RequestExchangeInput!) {
  requestExchange(input: $input) {
    exchangeId
    orderId
    orderDetailId
    desiredVariantId
    quantity
    status
    reason
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

  let body: RequestExchangeBody = {};
  try {
    body = (await request.json()) as RequestExchangeBody;
  } catch {
    return apiError("Invalid request body", 400, "VALIDATION_ERROR");
  }

  const orderDetailId = String(body.orderDetailId ?? "").trim();
  const desiredVariantId = String(body.desiredVariantId ?? "").trim();
  const reason = String(body.reason ?? "").trim();
  if (!orderDetailId) {
    return apiError("orderDetailId is required", 400, "VALIDATION_ERROR");
  }
  if (!desiredVariantId) {
    return apiError("desiredVariantId is required", 400, "VALIDATION_ERROR");
  }
  if (!reason) {
    return apiError("Exchange reason is required", 400, "VALIDATION_ERROR");
  }
  const quantity = String(body.quantity ?? "").trim();

  const result = await callGraphqlAsCustomer<{ requestExchange?: ExchangeRequestRow[] }>(
    userId,
    REQUEST_EXCHANGE_MUTATION,
    {
      input: {
        orderId: trimmedOrderId,
        orderDetailId,
        desiredVariantId,
        reason,
        ...(quantity ? { quantity } : {}),
      },
    }
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

  const row = result.data?.requestExchange?.[0];
  if (!row) {
    return apiError("Exchange request did not return a result", 502, "GRAPHQL_ERROR");
  }

  return Response.json({
    ok: true,
    data: row,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
