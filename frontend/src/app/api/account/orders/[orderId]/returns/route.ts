import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

type ReturnRequestRow = {
  returnId: string;
  orderId: string;
  status: string;
  reason: string;
  items: Array<{
    orderDetailId: string;
    quantity: string;
    status: string;
    refundAmountMinor: string;
  }>;
};

type RequestReturnBody = {
  orderDetailIds?: string[];
  reason?: string;
};

const REQUEST_RETURN_MUTATION = `mutation AccountRequestReturn($input: RequestReturnInput!) {
  requestReturn(input: $input) {
    returnId
    orderId
    status
    reason
    items {
      orderDetailId
      quantity
      status
      refundAmountMinor
    }
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

  let body: RequestReturnBody = {};
  try {
    body = (await request.json()) as RequestReturnBody;
  } catch {
    return apiError("Invalid request body", 400, "VALIDATION_ERROR");
  }

  const reason = String(body.reason ?? "").trim();
  if (!reason) {
    return apiError("Return reason is required", 400, "VALIDATION_ERROR");
  }
  const orderDetailIds = (body.orderDetailIds ?? [])
    .map((v) => String(v).trim())
    .filter(Boolean);
  if (orderDetailIds.length === 0) {
    return apiError("At least one orderDetailId is required", 400, "VALIDATION_ERROR");
  }

  const result = await callGraphqlAsCustomer<{ requestReturn?: ReturnRequestRow[] }>(
    userId,
    REQUEST_RETURN_MUTATION,
    {
      input: {
        orderId: trimmedOrderId,
        reason,
        items: orderDetailIds.map((orderDetailId) => ({ orderDetailId })),
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

  const row = result.data?.requestReturn?.[0];
  if (!row) {
    return apiError("Return request did not return a result", 502, "GRAPHQL_ERROR");
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
