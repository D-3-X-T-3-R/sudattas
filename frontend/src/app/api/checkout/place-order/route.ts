import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

type PlaceOrderRow = {
  orderId: string;
  totalAmountPaise: string;
  totalAmountFormatted: string;
  statusId: string;
  paymentMethod?: string | null;
};

type PaymentIntentRow = {
  intentId?: string;
  razorpayOrderId: string;
  razorpayKeyId?: string | null;
  orderId: string;
  amountPaise: string;
  currency?: string | null;
  status: string;
};

const ACTIVE_PAYMENT_INTENT_STATUSES = new Set([
  "pending",
  "client_verified",
  "needs_review",
]);

function isValidRazorpayOrderId(value: string | null | undefined): value is string {
  return typeof value === "string" && value.trim().startsWith("order_");
}

function parseIntentId(value: string | undefined): number {
  if (!value) return 0;
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) ? parsed : 0;
}

function pickCheckoutPaymentIntent(
  orderId: string,
  intents: PaymentIntentRow[] | undefined
): PaymentIntentRow | null {
  if (!intents?.length) return null;
  const sorted = intents
    .filter((row) => row?.orderId === orderId)
    .slice()
    .sort((a, b) => parseIntentId(b.intentId) - parseIntentId(a.intentId));
  if (!sorted.length) return null;

  const active = sorted.find(
    (row) =>
      ACTIVE_PAYMENT_INTENT_STATUSES.has((row.status ?? "").toLowerCase()) &&
      isValidRazorpayOrderId(row.razorpayOrderId)
  );
  if (active) return active;

  return sorted.find((row) => isValidRazorpayOrderId(row.razorpayOrderId)) ?? null;
}

const PLACE_ORDER_MUTATION = `mutation PlaceOrder($order: NewOrder!) {
  placeOrder(order: $order) {
    orderId
    totalAmountPaise
    totalAmountFormatted
    statusId
    paymentMethod
  }
}`;

const GET_PAYMENT_INTENT_QUERY = `query GetPaymentIntent($input: GetPaymentIntent!) {
  getPaymentIntent(input: $input) {
    intentId
    razorpayOrderId
    razorpayKeyId
    orderId
    amountPaise
    currency
    status
  }
}`;

export async function POST(request: Request) {
  const body = (await request.json().catch(() => ({}))) as {
    shippingAddressId?: string;
    couponCode?: string;
    idempotencyKey?: string;
    selectedCartLineIds?: unknown;
    paymentMode?: string;
  };

  const shippingAddressId = String(body.shippingAddressId ?? "").trim();
  if (!shippingAddressId) {
    return apiError("shippingAddressId is required", 400, "VALIDATION_ERROR");
  }
  const selectedCartLineIds = Array.isArray(body.selectedCartLineIds)
    ? body.selectedCartLineIds.map((value) => String(value ?? "").trim()).filter(Boolean)
    : [];
  if (selectedCartLineIds.length === 0) {
    return apiError("selectedCartLineIds must contain at least one cart line id", 400, "VALIDATION_ERROR");
  }
  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) {
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
  }

  const normalizedPaymentMode = (body.paymentMode ?? "prepaid").trim().toLowerCase();
  if (normalizedPaymentMode !== "prepaid" && normalizedPaymentMode !== "cod") {
    return apiError("paymentMode must be prepaid or cod", 400, "VALIDATION_ERROR");
  }

  const placeOrderKey =
    body.idempotencyKey?.trim() || `checkout-place-${crypto.randomUUID()}`;
  const verifyKey = `checkout-verify-${crypto.randomUUID()}`;

  const placeOrderResult = await callGraphqlAsCustomer<{ placeOrder?: PlaceOrderRow[] }>(
    userId,
    PLACE_ORDER_MUTATION,
    {
      order: {
        shippingAddressId,
        couponCode: body.couponCode?.trim() || null,
        selectedCartIds: selectedCartLineIds,
        paymentMode: normalizedPaymentMode,
      },
    },
    { "Idempotency-Key": placeOrderKey }
  );
  if (placeOrderResult.errors?.length) {
    return apiError(
      placeOrderResult.errors[0]?.message ?? "Failed to place order",
      400,
      "GRAPHQL_ERROR"
    );
  }
  const order = placeOrderResult.data?.placeOrder?.[0];
  if (!order?.orderId) {
    return apiError("Order was not created", 400, "GRAPHQL_ERROR");
  }

  const resolvedPaymentMode = (order.paymentMethod ?? normalizedPaymentMode).toLowerCase();
  if (resolvedPaymentMode === "cod") {
    return Response.json({
      ok: true,
      data: {
        order,
        checkoutMode: "cod",
        paymentIntent: null,
        idempotency: {
          placeOrderKey,
          verifyKey: null,
        },
      },
      errorCode: null,
      message: null,
      fieldErrors: null,
      retryable: false,
    });
  }

  const paymentResult = await callGraphqlAsCustomer<{ getPaymentIntent?: PaymentIntentRow[] }>(
    userId,
    GET_PAYMENT_INTENT_QUERY,
    {
      input: { orderId: order.orderId },
    }
  );
  if (paymentResult.errors?.length) {
    return apiError(
      paymentResult.errors[0]?.message ?? "Failed to load payment intent",
      400,
      "GRAPHQL_ERROR"
    );
  }

  const paymentIntent = pickCheckoutPaymentIntent(
    order.orderId,
    paymentResult.data?.getPaymentIntent
  );
  if (!paymentIntent?.razorpayKeyId || !isValidRazorpayOrderId(paymentIntent.razorpayOrderId)) {
    return apiError(
      "Payment intent is missing a valid Razorpay order ID",
      502,
      "PAYMENT_PROVIDER_ERROR"
    );
  }
  const normalizedIntent = {
    ...paymentIntent,
    razorpayOrderId: paymentIntent.razorpayOrderId.trim(),
    razorpayKeyId: paymentIntent.razorpayKeyId,
    currency: paymentIntent.currency ?? "INR",
  };

  return Response.json({
    ok: true,
    data: {
      order,
      checkoutMode: "prepaid",
      paymentIntent: normalizedIntent,
      idempotency: {
        placeOrderKey,
        verifyKey,
      },
    },
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
