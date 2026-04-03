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

const PLACE_ORDER_MUTATION = `mutation PlaceOrder($order: NewOrder!) {
  placeOrder(order: $order) {
    orderId
    totalAmountPaise
    totalAmountFormatted
    statusId
  }
}`;

const CREATE_PAYMENT_INTENT_MUTATION = `mutation CreatePaymentIntent($input: NewPaymentIntent!) {
  createPaymentIntent(input: $input) {
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
  };

  const shippingAddressId = String(body.shippingAddressId ?? "").trim();
  if (!shippingAddressId) {
    return apiError("shippingAddressId is required", 400, "VALIDATION_ERROR");
  }
  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) {
    return apiError("Unable to resolve customer identity", 401, "UNAUTHORIZED");
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

  const amountPaiseNum = Number.parseInt(order.totalAmountPaise, 10);
  if (!Number.isFinite(amountPaiseNum) || amountPaiseNum <= 0) {
    return apiError("Invalid order amount for payment", 400, "VALIDATION_ERROR");
  }

  const paymentResult = await callGraphqlAsCustomer<{ createPaymentIntent?: PaymentIntentRow[] }>(
    userId,
    CREATE_PAYMENT_INTENT_MUTATION,
    {
      input: {
        orderId: order.orderId,
        userId,
        amountPaise: String(amountPaiseNum),
        currency: "INR",
      },
    }
  );
  if (paymentResult.errors?.length) {
    return apiError(
      paymentResult.errors[0]?.message ?? "Failed to create payment intent",
      400,
      "GRAPHQL_ERROR"
    );
  }

  const paymentIntent = paymentResult.data?.createPaymentIntent?.[0];
  if (!paymentIntent?.razorpayOrderId || !paymentIntent?.razorpayKeyId) {
    return apiError(
      "Payment intent is missing Razorpay details",
      502,
      "PAYMENT_PROVIDER_ERROR"
    );
  }
  const normalizedIntent = {
    ...paymentIntent,
    razorpayKeyId: paymentIntent.razorpayKeyId,
    currency: paymentIntent.currency ?? "INR",
  };

  return Response.json({
    ok: true,
    data: {
      order,
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
