import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";
import {
  canonicalOrderStatusName,
  deriveOrderUiState,
  normalizePaymentState,
  statusNameFromId,
} from "@/lib/order-state";

type VerifyRow = {
  verified: boolean;
  paymentIntent?: {
    intentId?: string;
    status?: string;
    razorpayPaymentId?: string | null;
  } | null;
};

type OrderRow = {
  orderId: string;
  statusId: string;
};

type OrderStatusRow = {
  statusId: string;
  statusName: string;
};

const VERIFY_MUTATION = `mutation VerifyRazorpayPayment($input: VerifyRazorpayPaymentInput!) {
  verifyRazorpayPayment(input: $input) {
    verified
    paymentIntent {
      intentId
      status
      razorpayPaymentId
    }
  }
}`;

const ORDER_STATUS_QUERY = `query CheckoutOrderStatus($search: SearchOrder!) {
  searchOrder(search: $search) {
    orderId
    statusId
  }
}`;
const STATUS_LOOKUP_QUERY = `query CheckoutStatusLookup {
  searchOrderStatus {
    statusId
    statusName
  }
}`;

export async function POST(request: Request) {
  const customerUserId = await requireAuthenticatedCustomerUserId();
  if (!customerUserId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const body = (await request.json().catch(() => ({}))) as {
    orderId?: string;
    razorpayPaymentId?: string;
    razorpayOrderId?: string;
    razorpaySignature?: string;
    idempotencyKey?: string;
  };

  const orderId = String(body.orderId ?? "").trim();
  const razorpayPaymentId = String(body.razorpayPaymentId ?? "").trim();
  const razorpayOrderId = String(body.razorpayOrderId ?? "").trim();
  const razorpaySignature = String(body.razorpaySignature ?? "").trim();
  if (!orderId || !razorpayPaymentId || !razorpayOrderId || !razorpaySignature) {
    return apiError("Missing payment verification fields", 400, "VALIDATION_ERROR");
  }

  const verifyKey =
    body.idempotencyKey?.trim() || `checkout-verify-${crypto.randomUUID()}`;

  const verifyResult = await callGraphqlAsCustomer<{ verifyRazorpayPayment?: VerifyRow }>(
    customerUserId,
    VERIFY_MUTATION,
    {
      input: {
        orderId,
        razorpayPaymentId,
        razorpayOrderId,
        razorpaySignature,
      },
    },
    { "Idempotency-Key": verifyKey }
  );
  if (verifyResult.errors?.length) {
    return apiError(
      verifyResult.errors[0]?.message ?? "Payment verification failed",
      400,
      "GRAPHQL_ERROR"
    );
  }

  const verified = verifyResult.data?.verifyRazorpayPayment;
  if (!verified) {
    return apiError("Payment verification result missing", 400, "GRAPHQL_ERROR");
  }

  const [orderResult, statusesResult] = await Promise.all([
    callGraphqlAsCustomer<{ searchOrder?: OrderRow[] }>(customerUserId, ORDER_STATUS_QUERY, {
      search: { orderId, limit: "1", offset: "0", userId: "" },
    }),
    callGraphqlAsCustomer<{ searchOrderStatus?: OrderStatusRow[] }>(
      customerUserId,
      STATUS_LOOKUP_QUERY
    ),
  ]);
  if (orderResult.errors?.length) {
    return apiError(
      orderResult.errors[0]?.message ?? "Could not load updated order state",
      400,
      "GRAPHQL_ERROR"
    );
  }
  if (statusesResult.errors?.length) {
    return apiError(
      statusesResult.errors[0]?.message ?? "Could not resolve order status labels",
      400,
      "GRAPHQL_ERROR"
    );
  }
  const order = orderResult.data?.searchOrder?.[0] ?? null;
  const statusNameById = new Map(
    (statusesResult.data?.searchOrderStatus ?? []).map((status) => [
      status.statusId,
      canonicalOrderStatusName(status.statusName),
    ])
  );
  const orderStatusName = statusNameFromId(order?.statusId, statusNameById);

  return Response.json({
    ok: true,
    data: {
      verified: verified.verified,
      paymentState: normalizePaymentState(verified.paymentIntent?.status),
      orderStatusId: order?.statusId ?? null,
      orderStatusName,
      orderUiState: deriveOrderUiState(orderStatusName),
      verifyKey,
    },
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
