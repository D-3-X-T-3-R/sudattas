import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

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

function derivePaymentState(rawStatus?: string | null): string {
  const status = (rawStatus ?? "").toLowerCase();
  if (!status) return "pending";
  if (status.includes("needs_review")) return "needs_review";
  if (status.includes("captured") || status.includes("paid")) return "paid";
  if (status.includes("verified")) return "verified";
  if (status.includes("failed")) return "failed";
  if (status.includes("refunded")) return "refunded";
  return status;
}

function deriveOrderUiState(statusId?: string): string {
  if (!statusId) return "unknown";
  if (statusId === "1") return "pending";
  if (statusId === "2") return "processing";
  if (statusId === "3") return "shipped";
  if (statusId === "4") return "delivered";
  if (statusId === "5") return "cancelled";
  return statusId;
}

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

  const orderResult = await callGraphqlAsCustomer<{ searchOrder?: OrderRow[] }>(
    customerUserId,
    ORDER_STATUS_QUERY,
    { search: { orderId, limit: "1", offset: "0", userId: "" } }
  );
  if (orderResult.errors?.length) {
    return apiError(
      orderResult.errors[0]?.message ?? "Could not load updated order state",
      400,
      "GRAPHQL_ERROR"
    );
  }
  const order = orderResult.data?.searchOrder?.[0] ?? null;

  return Response.json({
    ok: true,
    data: {
      verified: verified.verified,
      paymentState: derivePaymentState(verified.paymentIntent?.status),
      orderStatusId: order?.statusId ?? null,
      orderUiState: deriveOrderUiState(order?.statusId),
      verifyKey,
    },
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
