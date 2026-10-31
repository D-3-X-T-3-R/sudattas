import {
  apiError,
  callGraphqlAsCustomer,
  graphqlErrorToApiStatus,
  requireAuthenticatedCustomerUserId,
} from "@/lib/server-session-auth";

type PublicCouponRow = {
  couponId: string;
  code: string;
  discountType: string;
  discountValue: number;
  minOrderValuePaise?: number | null;
  endsAt?: string | null;
};

const ACTIVE_COUPONS_QUERY = `query ListActiveCoupons {
  listActiveCoupons {
    couponId
    code
    discountType
    discountValue
    minOrderValuePaise
    endsAt
  }
}`;

export async function GET() {
  const userId = await requireAuthenticatedCustomerUserId();
  if (!userId) return apiError("Unauthorized", 401, "UNAUTHORIZED");

  const result = await callGraphqlAsCustomer<{ listActiveCoupons?: PublicCouponRow[] }>(
    userId,
    ACTIVE_COUPONS_QUERY
  );
  if (result.errors?.length) {
    const { status, message } = graphqlErrorToApiStatus(result.errors, "Failed to load offers");
    return apiError(message, status, "GRAPHQL_ERROR");
  }

  return Response.json({
    ok: true,
    data: result.data?.listActiveCoupons ?? [],
    errorCode: null,
    message: null,
    fieldErrors: null,
    retryable: false,
  });
}
