import {
  apiError,
  callGraphqlAsCustomer,
  graphqlErrorToApiStatus,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

type CouponValidationRow = {
  couponId: string;
  code: string;
  discountType: string;
  discountValue: number;
  discountAmountPaise: string;
  finalAmountPaise: string;
  isValid: boolean;
  reason: string;
};

const VALIDATE_COUPON_QUERY = `query ValidateCoupon($input: ValidateCoupon!) {
  validateCoupon(input: $input) {
    couponId
    code
    discountType
    discountValue
    discountAmountPaise
    finalAmountPaise
    isValid
    reason
  }
}`;

/** Immediate, dedicated coupon check for the bag page's "Apply" button — checks only the
 * coupon's own properties (exists, active, date window, usage limit, min order value) against
 * the current item subtotal. Does NOT check per-cart scope or per-customer usage limits; those
 * are still enforced for real at shipping-estimate/place-order once the code is applied. */
export async function POST(request: Request) {
  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const body = (await request.json().catch(() => ({}))) as {
    code?: string;
    orderAmountPaise?: string;
  };
  const code = String(body.code ?? "").trim();
  if (!code) return apiError("code is required", 400, "VALIDATION_ERROR");
  const orderAmountPaise = String(body.orderAmountPaise ?? "0").trim();

  const result = await callGraphqlAsCustomer<{ validateCoupon?: CouponValidationRow[] }>(
    userId,
    VALIDATE_COUPON_QUERY,
    { input: { code, orderAmountPaise } }
  );
  if (result.errors?.length) {
    const { status, message } = graphqlErrorToApiStatus(result.errors, "Failed to check coupon");
    return apiError(message, status, "GRAPHQL_ERROR");
  }

  return Response.json({
    ok: true,
    data: result.data?.validateCoupon?.[0] ?? null,
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
