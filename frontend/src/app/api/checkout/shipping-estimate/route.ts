import {
  apiError,
  callGraphqlAsCustomer,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

type ShippingEstimateRow = {
  shippingAmountPaise: string;
  courierName?: string | null;
  estimatedDeliveryDays?: number | null;
  itemSubtotalPaise: string;
  orderTotalPaise: string;
  quoteAvailable: boolean;
  note?: string | null;
};

const ESTIMATE_SHIPPING_QUERY = `query EstimateCheckoutShipping($input: EstimateCheckoutShippingInput!) {
  estimateCheckoutShipping(input: $input) {
    shippingAmountPaise
    courierName
    estimatedDeliveryDays
    itemSubtotalPaise
    orderTotalPaise
    quoteAvailable
    note
  }
}`;

export async function POST(request: Request) {
  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const body = (await request.json().catch(() => ({}))) as {
    shippingAddressId?: string;
    couponCode?: string;
  };
  const shippingAddressId = String(body.shippingAddressId ?? "").trim();
  if (!shippingAddressId) {
    return apiError("shippingAddressId is required", 400, "VALIDATION_ERROR");
  }

  const result = await callGraphqlAsCustomer<{
    estimateCheckoutShipping?: ShippingEstimateRow | null;
  }>(userId, ESTIMATE_SHIPPING_QUERY, {
    input: {
      shippingAddressId,
      couponCode: body.couponCode?.trim() || null,
    },
  });

  if (result.errors?.length) {
    return apiError(
      result.errors[0]?.message ?? "Failed to estimate shipping",
      400,
      "GRAPHQL_ERROR"
    );
  }

  return Response.json({
    ok: true,
    data: result.data?.estimateCheckoutShipping ?? null,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
